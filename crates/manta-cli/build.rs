//! Regenerate man pages and shell completions into the source tree
//! when explicitly asked, otherwise do nothing.
//!
//! The generated files at `man/` and `autocomplete_shell_scripts/`
//! are checked into git so cargo-dist's `include` directive can
//! reference them at stable, glob-free paths.
//!
//! Default behaviour: no-op. This keeps `cargo build` and
//! `cargo publish` from mutating the source tree (cargo publish's
//! verify step rejects build-script writes outside `OUT_DIR`,
//! including "regenerating identical files" because the mtime
//! changes).
//!
//! To refresh the committed files after a CLI definition change:
//!
//!   MANTA_REGENERATE_DOCS=1 cargo build -p manta-cli
//!   git diff -- crates/manta-cli/{man,autocomplete_shell_scripts}/
//!
//! CI should run that command followed by `git diff --exit-code` so
//! PRs with stale generated docs fail loudly.

use clap_complete::{Shell, generate_to};
use std::fs;
use std::io::Error;
use std::path::Path;

#[path = "src/build/mod.rs"]
mod cli;

fn main() -> Result<(), Error> {
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
  println!("cargo:warning=man page regenerated at {}", man_path.display());

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
