//! CI safeguard: every method declared on a backend trait in
//! `manta-backend-dispatcher::interfaces` must be overridden in some
//! `impl X for StaticBackendDispatcher` block under
//! `crates/manta-server/src/backend_dispatcher/`.
//!
//! # Why this exists
//!
//! Rust's trait dispatch silently falls back to the default impl when
//! `impl X for Y` doesn't override a method. If a trait grows (e.g. a
//! new method gets added upstream in `manta-backend-dispatcher`) and
//! the dispatcher wrapper isn't updated, every call to the new method
//! on `StaticBackendDispatcher` hits the trait's default — which by
//! convention returns `Err(Error::Message("... not implemented for
//! this backend"))`. The error is misleading: it implies the *backend*
//! doesn't implement the method, but the actual backend (csm-rs /
//! ochami-rs) may have a perfectly good impl. The bug is in the
//! dispatcher's forwarding layer, and it's invisible until a handler
//! tries to call the missing method.
//!
//! This caught two real instances in production (the SAT-image
//! "create-session" / "stamp-from-session" pair) and three latent
//! gaps (`CfsTrait::get_cfs_health`, `PCSTrait::power_status`, and
//! the entire `ComponentEthernetInterfaceTrait`). All five were
//! fixed; this test guards against the next one.
//!
//! # How it works
//!
//! 1. Locate the trait crate's source root via `cargo metadata`.
//! 2. Recursively read every `.rs` under
//!    `<trait crate>/src/interfaces/`, parse with `syn`, and collect
//!    every `(trait_name, method_name)` pair for `pub trait` items.
//! 3. Recursively read every `.rs` under
//!    `crates/manta-server/src/backend_dispatcher/`, parse with `syn`,
//!    and collect every `(trait_name, method_name)` pair for
//!    `impl <trait_name> for StaticBackendDispatcher` blocks.
//! 4. Set-difference: each trait pair must appear in the impl set.
//! 5. On a non-empty difference, panic with the full gap report.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::Command;

use syn::{ImplItem, Item, TraitItem};

#[test]
fn dispatcher_covers_every_trait_method() {
  let trait_src = trait_crate_interfaces_dir();
  let wrapper_src = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    .join("src/backend_dispatcher");

  let mut trait_pairs = HashSet::new();
  for file in rs_files(&trait_src) {
    collect_trait_methods(&file, &mut trait_pairs);
  }

  let mut impl_pairs = HashSet::new();
  for file in rs_files(&wrapper_src) {
    collect_impl_methods(&file, &mut impl_pairs);
  }

  let missing: Vec<&(String, String)> =
    trait_pairs.difference(&impl_pairs).collect();

  if !missing.is_empty() {
    let mut sorted = missing.clone();
    sorted.sort();
    let report: String = sorted
      .iter()
      .map(|(t, m)| format!("  - {t}::{m}"))
      .collect::<Vec<_>>()
      .join("\n");

    panic!(
      "StaticBackendDispatcher is missing impl overrides for the following \
       trait methods:\n\n{report}\n\n\
       Add a `dispatch!(...)`-wrapped impl method for each, in the matching \
       file under `crates/manta-server/src/backend_dispatcher/`. Without an \
       override, calls fall through to the trait crate's default, which \
       returns the misleading \"not implemented for this backend\" error \
       even when the backend (csm-rs / ochami-rs) does implement it.\n\n\
       If a trait method genuinely shouldn't be exposed via the dispatcher, \
       update this test's allow-list (currently empty) with a comment \
       explaining why."
    );
  }
}

/// Resolve `manta-backend-dispatcher/src/interfaces/` via `cargo
/// metadata`. Works both with local path overrides (the trait crate
/// is checked out at a sibling directory) and on CI (the trait crate
/// comes from the registry cache).
fn trait_crate_interfaces_dir() -> PathBuf {
  let out = Command::new(env!("CARGO"))
    .args(["metadata", "--format-version=1"])
    .current_dir(env!("CARGO_MANIFEST_DIR"))
    .output()
    .expect("cargo metadata failed");
  assert!(
    out.status.success(),
    "cargo metadata exited non-zero: {}",
    String::from_utf8_lossy(&out.stderr)
  );

  // `cargo metadata` returns each package as an object with both
  // a top-level `name` AND a top-level `manifest_path`. Transitive
  // packages (csm-rs, ochami-rs, …) also have `dependencies` arrays
  // whose entries have a `name` field that matches our search term
  // but lack `manifest_path` — a naive string scan would match the
  // wrong package. Use a real JSON parse.
  let meta: serde_json::Value = serde_json::from_slice(&out.stdout)
    .expect("cargo metadata is not valid JSON");
  let packages = meta["packages"]
    .as_array()
    .expect("cargo metadata has no `packages` array");
  let dispatcher = packages
    .iter()
    .find(|p| p["name"].as_str() == Some("manta-backend-dispatcher"))
    .expect("manta-backend-dispatcher not in cargo metadata `packages`");
  let manifest_path = dispatcher["manifest_path"]
    .as_str()
    .expect("manta-backend-dispatcher has no manifest_path");

  PathBuf::from(manifest_path)
    .parent()
    .expect("manifest_path has no parent")
    .join("src")
    .join("interfaces")
}

/// Recursively yield every `.rs` file under `dir`.
fn rs_files(dir: &Path) -> Vec<PathBuf> {
  let mut out = vec![];
  let entries = std::fs::read_dir(dir)
    .unwrap_or_else(|e| panic!("cannot read {}: {e}", dir.display()));
  for entry in entries {
    let entry = entry.expect("dir entry read failed");
    let path = entry.path();
    if path.is_dir() {
      out.extend(rs_files(&path));
    } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
      out.push(path);
    }
  }
  out
}

/// Parse `file` and add `(trait_name, method_name)` for every method
/// declared on every `pub trait` item.
fn collect_trait_methods(
  file: &Path,
  out: &mut HashSet<(String, String)>,
) {
  let src = std::fs::read_to_string(file).unwrap_or_else(|e| {
    panic!("cannot read trait source {}: {e}", file.display())
  });
  let ast: syn::File = syn::parse_file(&src).unwrap_or_else(|e| {
    panic!("cannot parse trait source {}: {e}", file.display())
  });
  for item in &ast.items {
    if let Item::Trait(t) = item {
      let trait_name = t.ident.to_string();
      for ti in &t.items {
        if let TraitItem::Fn(f) = ti {
          out.insert((trait_name.clone(), f.sig.ident.to_string()));
        }
      }
    }
  }
}

/// Parse `file` and add `(trait_name, method_name)` for every method
/// in every `impl <trait_name> for StaticBackendDispatcher` block.
/// Inherent impls (`impl StaticBackendDispatcher`) and impls on other
/// types are skipped.
fn collect_impl_methods(file: &Path, out: &mut HashSet<(String, String)>) {
  let src = std::fs::read_to_string(file).unwrap_or_else(|e| {
    panic!("cannot read wrapper source {}: {e}", file.display())
  });
  let ast: syn::File = syn::parse_file(&src).unwrap_or_else(|e| {
    panic!("cannot parse wrapper source {}: {e}", file.display())
  });
  for item in &ast.items {
    let Item::Impl(i) = item else {
      continue;
    };
    // We only care about `impl <Trait> for StaticBackendDispatcher`.
    let Some((_, trait_path, _)) = &i.trait_ else {
      continue;
    };
    let Some(self_ident) = type_name(&i.self_ty) else {
      continue;
    };
    if self_ident != "StaticBackendDispatcher" {
      continue;
    }
    let Some(trait_seg) = trait_path.segments.last() else {
      continue;
    };
    let trait_name = trait_seg.ident.to_string();

    for impl_item in &i.items {
      if let ImplItem::Fn(f) = impl_item {
        out.insert((trait_name.clone(), f.sig.ident.to_string()));
      }
    }
  }
}

/// Extract the last segment of a `Type::Path` as a string. Returns
/// `None` for non-path types (references, tuples, …); the dispatcher's
/// impl blocks all use a bare path so this is sufficient.
fn type_name(ty: &syn::Type) -> Option<String> {
  if let syn::Type::Path(p) = ty {
    p.path.segments.last().map(|s| s.ident.to_string())
  } else {
    None
  }
}
