//! Clap definitions for `manta restore *` subcommands.

use clap::{ArgAction, Command, ValueHint, arg};

use super::output_flag_long_only;

/// Attach the vCluster-restore argument set.
fn add_vcluster_restore_args(cmd: Command) -> Command {
  cmd
    .arg_required_else_help(true)
    .arg(
      arg!(-b --"bos-file" <FILE> "Session template backup file")
        .value_hint(ValueHint::FilePath),
    )
    .arg(
      arg!(-c --"cfs-file" <FILE> "Configuration backup file")
        .value_hint(ValueHint::FilePath),
    )
    .arg(
      arg!(-j --"hsm-file" <FILE> "Group description backup file")
        .value_hint(ValueHint::FilePath),
    )
    .arg(
      arg!(-m --"ims-file" <FILE> "Image metadata backup file")
        .value_hint(ValueHint::FilePath),
    )
    .arg(
      arg!(-i --"image-dir" <PATH> "Directory containing the image files")
        .value_hint(ValueHint::DirPath),
    )
    .arg(arg!(-p --"pre-hook" <SCRIPT> "Command to run before the restore.\neg: --pre-hook \"echo hello\""))
    .arg(
      arg!(-a --"post-hook" <SCRIPT> "Command to run after a successful restore.\neg: --post-hook \"echo hello\""),
    )
    .arg(arg!(-o --"overwrite" "Overwrite existing data").action(ArgAction::SetTrue))
    .arg(
      arg!(-d --"dry-run" "Simulate the operation without making changes")
        .action(ArgAction::SetTrue),
    )
    .arg(output_flag_long_only())
}

/// Top-level `manta restore` verb.
pub fn subcommand_restore() -> Command {
  Command::new("restore")
    .arg_required_else_help(true)
    .about("Restore a virtual cluster from a backup bundle")
    .subcommand(
      add_vcluster_restore_args(Command::new("vcluster"))
        .about("Restore a virtual cluster from a backup bundle"),
    )
}
