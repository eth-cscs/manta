//! Clap definition for `manta gen-man` — generate and install the
//! consolidated `manta.1` man page directly from the running binary's
//! clap tree.

use clap::{Command, ValueHint, arg, value_parser};
use std::path::PathBuf;

use super::output_flag;

pub fn subcommand_gen_man() -> Command {
  Command::new("gen-man")
    .about("Generate and install the manta man page")
    .long_about(
      "Generate the consolidated `manta.1` man page from the running \
       binary's clap tree and write it into the target directory. The \
       page has a top-level reference plus one SUBCOMMANDS subsection \
       per (sub)subcommand, so `/<verb>` inside `man manta` jumps to \
       the relevant flag table.\n\n\
       Defaults to `$XDG_DATA_HOME/man/man1` (i.e. \
       `~/.local/share/man/man1`) when `--path` is omitted. The \
       directory is created if it doesn't exist.\n\n\
       After install, configure your shell so `man` finds the new \
       location — on Linux this directory is searched by default; on \
       macOS you may need `export MANPATH=\"$HOME/.local/share/man:$MANPATH\"`.",
    )
    .arg(
      arg!(-p --path <PATH> "Directory to write `manta.1` into (defaults to $XDG_DATA_HOME/man/man1)")
        .value_parser(value_parser!(PathBuf))
        .value_hint(ValueHint::DirPath),
    )
    .arg(output_flag())
}
