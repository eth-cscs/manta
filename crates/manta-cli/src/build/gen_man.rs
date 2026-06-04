//! Clap definition for `manta gen-man` — generate and install the
//! manta man pages (one `.1` file per subcommand) directly from the
//! running binary's clap tree.

use clap::{Command, ValueHint, arg, value_parser};
use std::path::PathBuf;

use super::output_flag;

pub fn subcommand_gen_man() -> Command {
  Command::new("gen-man")
    .about("Generate and install manta man pages")
    .long_about(
      "Generate one man-page (`.1`) per subcommand from the running \
       binary's clap tree and write them into the target directory.\n\n\
       Defaults to `$XDG_DATA_HOME/man/man1` (i.e. \
       `~/.local/share/man/man1`) when `--path` is omitted. The \
       directory is created if it doesn't exist.\n\n\
       After install, configure your shell so `man` finds the new \
       location — on Linux this directory is searched by default; on \
       macOS you may need `export MANPATH=\"$HOME/.local/share/man:$MANPATH\"`.",
    )
    .arg(
      arg!(-p --path <PATH> "Directory to write the `.1` files into (defaults to $XDG_DATA_HOME/man/man1)")
        .value_parser(value_parser!(PathBuf))
        .value_hint(ValueHint::DirPath),
    )
    .arg(output_flag())
}
