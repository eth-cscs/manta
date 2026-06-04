//! Clap definitions for `manta backup *` subcommands and the
//! deprecated `manta migrate vCluster backup` alias.

use clap::{Command, ValueHint, arg};

use super::output_flag;

/// Attach the vCluster-backup argument set to a clap `Command`. Shared
/// between the canonical `manta backup vcluster` and the deprecated
/// `manta migrate vCluster backup` paths so both stay in lockstep.
pub(super) fn add_vcluster_backup_args(cmd: Command) -> Command {
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

/// Deprecated `manta migrate vCluster backup`; wired into the
/// `subcommand_migrate` tree in [`super::migrate`].
pub(super) fn subcommand_migrate_backup() -> Command {
  add_vcluster_backup_args(Command::new("backup"))
    .about("[DEPRECATED] Use 'manta backup vcluster' instead")
    .long_about(
      "Back up a cluster's configuration: images, boot settings, and group membership.\n\n\
      The backup is derived from the specified session template.\n\n\
      DEPRECATED: this command has moved to the top-level verb tree as \
      `manta backup vcluster`. The old path keeps working for one release.",
    )
}
