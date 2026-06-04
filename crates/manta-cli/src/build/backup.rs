//! Clap definitions for `manta backup *` subcommands.

use clap::{Command, ValueHint, arg};

use super::output_flag;

/// Attach the vCluster-backup argument set to a clap `Command`.
fn add_vcluster_backup_args(cmd: Command) -> Command {
  cmd
    .arg_required_else_help(true)
    .arg(arg!(-b --"bos" <SESSIONTEMPLATE> "Session template to derive the backup from"))
    .arg(
      arg!(-d --"destination" <FOLDER> "Destination directory for the backup files")
        .value_hint(ValueHint::DirPath),
    )
    .arg(arg!(-p --"pre-hook" <SCRIPT> "Command to run before the backup.\neg: --pre-hook \"echo hello\""))
    .arg(
      arg!(-a --"post-hook" <SCRIPT> "Command to run after a successful backup.\neg: --post-hook \"echo hello\""),
    )
    .arg(output_flag())
}

/// Top-level `manta backup` verb.
pub fn subcommand_backup() -> Command {
  Command::new("backup")
    .arg_required_else_help(true)
    .about("Back up a virtual cluster (images, boot settings, group membership) to disk")
    .subcommand(
      add_vcluster_backup_args(Command::new("vcluster"))
        .about("Back up a virtual cluster (images, boot settings, group membership)")
        .long_about(
          "Back up a virtual cluster's configuration: images, boot settings, \
          and group membership.\n\nThe backup is derived from the specified session template.",
        ),
    )
}

