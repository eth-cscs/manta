use clap_complete::{Shell, generate_to};
use clap_mangen::generate_to as generate_man_to;
use std::fs;
use std::io::Error;

#[path = "src/cli/build.rs"]
mod cli;

fn main() -> Result<(), Error> {
  // ── Man pages ─────────────────────────────────────────────────────────────
  let man_dir = "man";
  fs::create_dir_all(man_dir)?;
  generate_man_to(cli::build_cli(), man_dir)?;
  println!("cargo:warning=man pages generated in {man_dir}/");

  // ── Shell completions ──────────────────────────────────────────────────────
  let completion_dir = "autocomplete_shell_scripts";
  fs::create_dir_all(completion_dir)?;

  for shell in [Shell::Bash, Shell::Zsh, Shell::Fish, Shell::Elvish] {
    let path = generate_to(shell, &mut cli::build_cli(), env!("CARGO_PKG_NAME"), completion_dir)?;
    println!("cargo:warning=completion file generated: {path:?}");
  }

  Ok(())
}
