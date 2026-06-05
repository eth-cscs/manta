//! Clap definition for `manta gen-autocomplete` — generates **and
//! installs** the shell completion script for bash/zsh/fish.

use clap::{ArgAction, Command, ValueHint, arg, value_parser};
use std::path::PathBuf;

use super::output_flag;

pub fn subcommand_gen_autocomplete() -> Command {
  Command::new("gen-autocomplete")
    .about("Generate and install shell completion scripts")
    .long_about(
      "Generate the shell completion script and install it into the \
       shell's standard XDG user completion directory:\n\n  \
       bash → $XDG_DATA_HOME/bash-completion/completions/manta\n  \
       zsh  → $XDG_DATA_HOME/zsh/site-functions/_manta\n  \
       fish → $XDG_CONFIG_HOME/fish/completions/manta.fish\n\n\
       Pass `--path <DIR>` to install elsewhere (e.g. a system-wide \
       location or a directory already on your `$fpath`), or `--print` \
       to emit the script to stdout without touching the filesystem.",
    )
    .arg(
      arg!(-s --shell <SHELL> "Shell type (guessed from $SHELL if omitted)")
        .value_parser(["bash", "zsh", "fish"]),
    )
    .arg(
      arg!(-p --path <PATH> "Override the default install directory")
        .value_parser(value_parser!(PathBuf))
        .value_hint(ValueHint::DirPath),
    )
    .arg(
      arg!(--print "Emit the script to stdout instead of installing it")
        .action(ArgAction::SetTrue)
        .conflicts_with("path"),
    )
    .arg(output_flag())
}
