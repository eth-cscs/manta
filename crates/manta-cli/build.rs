//! Two build-time tasks:
//!
//! 1. **OpenAPI client codegen** (every build). Reads
//!    `openapi.json` and emits a typed `reqwest` client into
//!    `OUT_DIR/openapi_client.rs`. The generated module is included
//!    from `src/openapi_client.rs`. Refresh the spec after handler /
//!    schema changes:
//!
//!        cargo run -p manta-server -- --emit-openapi > crates/manta-cli/openapi.json
//!
//! 2. **Man pages + shell completions** (env-var-gated). The generated
//!    files at `man/` and `autocomplete_shell_scripts/` are checked
//!    into git so cargo-dist's `include` directive can reference them
//!    at stable, glob-free paths. They write into the source tree,
//!    which `cargo publish` rejects — so by default this regeneration
//!    is a no-op.
//!
//!    To refresh after a CLI definition change:
//!
//!        MANTA_REGENERATE_DOCS=1 cargo build -p manta-cli
//!        git diff -- crates/manta-cli/{man,autocomplete_shell_scripts}/
//!
//!    CI should run that command followed by `git diff --exit-code` so
//!    PRs with stale generated docs fail loudly.

use clap_complete::{Shell, generate_to};
use std::fs;
use std::io::Error;
use std::path::Path;

#[path = "src/build/mod.rs"]
mod cli;

fn main() -> Result<(), Error> {
  generate_openapi_client()?;
  regenerate_docs_if_requested()
}

/// Generate the typed reqwest client from `openapi.json` using
/// progenitor. Writes to `OUT_DIR/openapi_client.rs`, included from
/// `src/openapi_client.rs`.
///
/// utoipa emits OpenAPI 3.1, which uses array-style `"type":
/// ["string", "null"]` for nullable fields. progenitor (via
/// `openapiv3` v2) only understands OpenAPI 3.0, so we down-convert
/// each nullable schema to the 3.0 `"nullable": true` form before
/// handing the spec off.
fn generate_openapi_client() -> Result<(), Error> {
  let spec_path = "openapi.json";
  println!("cargo:rerun-if-changed={spec_path}");

  let spec_content = fs::read_to_string(spec_path)?;
  let mut spec_value: serde_json::Value = serde_json::from_str(&spec_content)
    .map_err(|e| Error::other(format!("openapi.json is not valid JSON: {e}")))?;

  downconvert_to_openapi_30(&mut spec_value);

  let spec: openapiv3::OpenAPI = serde_json::from_value(spec_value)
    .map_err(|e| Error::other(format!("openapi.json is not valid OpenAPI: {e}")))?;

  let mut generator = progenitor::Generator::default();
  let tokens = generator
    .generate_tokens(&spec)
    .map_err(|e| Error::other(format!("progenitor codegen failed: {e}")))?;

  let ast = syn::parse2(tokens)
    .map_err(|e| Error::other(format!("generated tokens did not parse: {e}")))?;
  let pretty = prettyplease::unparse(&ast);

  let out_dir = std::env::var_os("OUT_DIR").ok_or_else(|| {
    Error::other("OUT_DIR not set by cargo (build.rs invariant)")
  })?;
  let out_file = Path::new(&out_dir).join("openapi_client.rs");
  fs::write(out_file, pretty)?;
  Ok(())
}

/// Walk the JSON tree and rewrite OpenAPI 3.1 nullable idioms into
/// 3.0-compatible ones so the `openapiv3` v2 parser used by
/// progenitor accepts the document. Specifically:
///
/// - top-level `"openapi": "3.1.x"` → `"3.0.3"`;
/// - `{"type": ["X", "null"]}` → `{"type": "X", "nullable": true}`;
/// - `oneOf: [{"type": "null"}, X]` (in either order) → X with
///   `nullable: true`. If X is a bare `$ref`, the result wraps it
///   in `allOf` since 3.0 disallows siblings to `$ref`.
fn downconvert_to_openapi_30(value: &mut serde_json::Value) {
  if let serde_json::Value::Object(obj) = value {
    if let Some(serde_json::Value::String(v)) = obj.get("openapi") {
      if v.starts_with("3.1") {
        obj.insert(
          "openapi".to_string(),
          serde_json::Value::String("3.0.3".to_string()),
        );
      }
    }

    if let Some(serde_json::Value::Array(types)) = obj.get("type").cloned() {
      let non_null: Vec<&str> = types
        .iter()
        .filter_map(|v| v.as_str())
        .filter(|s| *s != "null")
        .collect();
      let has_null = types.iter().any(|v| v.as_str() == Some("null"));
      if non_null.len() == 1 {
        obj.insert(
          "type".to_string(),
          serde_json::Value::String(non_null[0].to_string()),
        );
        if has_null {
          obj.insert("nullable".to_string(), serde_json::Value::Bool(true));
        }
      }
    }

    if let Some(serde_json::Value::Array(arr)) = obj.get("oneOf").cloned() {
      let has_null = arr
        .iter()
        .any(|e| e.as_object().and_then(|o| o.get("type")).and_then(|v| v.as_str()) == Some("null"));
      let non_null: Vec<serde_json::Value> = arr
        .iter()
        .filter(|e| {
          e.as_object().and_then(|o| o.get("type")).and_then(|v| v.as_str()) != Some("null")
        })
        .cloned()
        .collect();
      if has_null && non_null.len() == 1 {
        obj.remove("oneOf");
        let only = non_null.into_iter().next().expect("checked len == 1");
        if only.as_object().is_some_and(|o| o.contains_key("$ref")) {
          obj.insert(
            "allOf".to_string(),
            serde_json::Value::Array(vec![only]),
          );
        } else if let serde_json::Value::Object(inner) = only {
          for (k, v) in inner {
            obj.insert(k, v);
          }
        }
        obj.insert("nullable".to_string(), serde_json::Value::Bool(true));
      }
    }

    for (_, child) in obj.iter_mut() {
      downconvert_to_openapi_30(child);
    }
  } else if let serde_json::Value::Array(arr) = value {
    for child in arr.iter_mut() {
      downconvert_to_openapi_30(child);
    }
  }
}

fn regenerate_docs_if_requested() -> Result<(), Error> {
  // Re-run when the env-var flips, otherwise cargo treats a stale
  // build script as up-to-date and the regeneration silently no-ops.
  println!("cargo:rerun-if-env-changed=MANTA_REGENERATE_DOCS");
  if std::env::var("MANTA_REGENERATE_DOCS").is_err() {
    return Ok(());
  }

  // ── Man page ──────────────────────────────────────────────────────────────
  // One consolidated `manta.1` covers the top-level page + every
  // (sub)subcommand inline. See `src/build/manpage.rs`.
  let man_dir = "man";
  fs::create_dir_all(man_dir)?;
  let man_path = Path::new(man_dir).join("manta.1");
  let mut f = fs::File::create(&man_path)?;
  cli::manpage::render_consolidated(cli::build_cli(), &mut f)?;
  println!(
    "cargo:warning=man page regenerated at {}",
    man_path.display()
  );

  // ── Shell completions ──────────────────────────────────────────────────────
  let completion_dir = "autocomplete_shell_scripts";
  fs::create_dir_all(completion_dir)?;
  for shell in [Shell::Bash, Shell::Zsh, Shell::Fish, Shell::Elvish] {
    let path = generate_to(
      shell,
      &mut cli::build_cli(),
      // Binary name is `manta`, not the package name `manta-cli` —
      // the `[[bin]]` block in Cargo.toml renames the produced binary.
      "manta",
      completion_dir,
    )?;
    println!("cargo:warning=completion regenerated: {path:?}");
  }

  Ok(())
}
