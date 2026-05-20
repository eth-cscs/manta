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
use clap_mangen::generate_to as generate_man_to;
use std::fs;
use std::io::Error;

#[path = "src/cli/build/mod.rs"]
mod cli;

fn main() -> Result<(), Error> {
  if std::env::var("MANTA_REGENERATE_DOCS").is_err() {
    return Ok(());
  }

  // ── Man pages ─────────────────────────────────────────────────────────────
  let man_dir = "man";
  fs::create_dir_all(man_dir)?;
  generate_man_to(cli::build_cli(), man_dir)?;
  println!("cargo:warning=man pages regenerated in {man_dir}/");

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
