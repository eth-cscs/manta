//! Generate man pages and shell completions into cargo's `OUT_DIR`.
//!
//! Writing to `OUT_DIR` (per cargo convention) keeps the source tree
//! untouched on every build, which is what `cargo publish`'s verify
//! step requires. The generated files end up at:
//!
//!   target/{profile}/build/manta-cli-{fingerprint}/out/man/*.1
//!   target/{profile}/build/manta-cli-{fingerprint}/out/completions/*
//!
//! `dist` (in `dist-workspace.toml`) glob-includes those paths into
//! release tarballs. The `{fingerprint}` hash varies between builds,
//! so the glob is `target/release/build/manta-cli-*/out/...`.
//!
//! For local inspection:
//!
//!   cargo build -p manta-cli --release
//!   find target/release/build -path '*/manta-cli-*/out/man'
//!
//! There are no committed copies in the source tree.

use clap_complete::{Shell, generate_to};
use clap_mangen::generate_to as generate_man_to;
use std::fs;
use std::io::Error;
use std::path::PathBuf;

#[path = "src/cli/build/mod.rs"]
mod cli;

fn main() -> Result<(), Error> {
  // cargo always sets OUT_DIR for build scripts; the .expect() is
  // safe by contract.
  let out_dir = PathBuf::from(
    std::env::var_os("OUT_DIR").expect("cargo always sets OUT_DIR"),
  );

  // ── Man pages ─────────────────────────────────────────────────────────────
  let man_dir = out_dir.join("man");
  fs::create_dir_all(&man_dir)?;
  generate_man_to(cli::build_cli(), &man_dir)?;

  // ── Shell completions ──────────────────────────────────────────────────────
  let completion_dir = out_dir.join("completions");
  fs::create_dir_all(&completion_dir)?;
  for shell in [Shell::Bash, Shell::Zsh, Shell::Fish, Shell::Elvish] {
    generate_to(
      shell,
      &mut cli::build_cli(),
      env!("CARGO_PKG_NAME"),
      &completion_dir,
    )?;
  }

  Ok(())
}
