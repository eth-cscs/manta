//! Clap definition for `manta upgrade` — self-update the manta binary
//! from the project's GitHub releases.

use clap::{ArgAction, Command, arg};

use super::output_flag;

pub fn subcommand_upgrade() -> Command {
  Command::new("upgrade")
    .about("Replace this `manta` binary with the latest release")
    .long_about(
      "Fetch the highest `manta-cli-v*` tag from \
       https://github.com/eth-cscs/manta/releases, compare against the \
       current version, and (if newer) download the right platform \
       tarball and atomically replace the running binary.\n\n\
       Use `brew upgrade manta-cli` instead if you installed via \
       Homebrew — this command will warn but not block in that case.",
    )
    .arg(
      arg!(-c --check "Check for a newer version and print it, but don't apply")
        .action(ArgAction::SetTrue),
    )
    .arg(
      arg!(-d --"dry-run" "Show what would happen without downloading or replacing")
        .action(ArgAction::SetTrue),
    )
    .arg(
      arg!(-y --"assume-yes" "Skip the confirmation prompt before replacing the binary")
        .action(ArgAction::SetTrue),
    )
    .arg(output_flag())
}
