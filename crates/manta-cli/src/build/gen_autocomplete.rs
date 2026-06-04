//! Clap definition for `manta gen-autocomplete` — generates shell
//! completion scripts for bash/zsh/fish.

use clap::{Command, ValueHint, arg, value_parser};
use std::path::PathBuf;

pub fn subcommand_gen_autocomplete() -> Command {
  Command::new("gen-autocomplete")
    .about("Generate shell completion scripts")
    .arg(
      arg!(-s --shell <SHELL> "Shell type (guessed from $SHELL if omitted)")
        .value_parser(["bash", "zsh", "fish"]),
    )
    .arg(
      arg!(-p --path <PATH> "Directory to write the script (prints to stdout if omitted)")
        .value_parser(value_parser!(PathBuf))
        .value_hint(ValueHint::DirPath),
    )
}
