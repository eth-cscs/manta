//! Clap definitions for `manta migrate *` subcommands.

use clap::{ArgAction, Command, arg};

use super::backup::subcommand_migrate_backup;
use super::output_flag;
use super::restore::subcommand_migrate_restore;

pub fn subcommand_migrate() -> Command {
  Command::new("migrate")
    .arg_required_else_help(true)
    .about("Move nodes between groups (vCluster backup/restore have moved to 'manta backup'/'manta restore')")
    .subcommand(
      Command::new("vCluster")
        .about("[experimental] Migrate a cluster")
        .subcommand(subcommand_migrate_backup())
        .subcommand(subcommand_migrate_restore()),
    )
    .subcommand(
      Command::new("nodes")
        .arg_required_else_help(true)
        .about("Move nodes between clusters")
        .arg(arg!(-f --from <NAME> "Source cluster to move nodes from"))
        .arg(arg!(-t --to <NAME> "Destination cluster to move nodes to").required(true))
        // ID preserved as "XNAMES" for handler compatibility
        .arg(
          arg!(<XNAMES> "Xnames, NIDs, or a hostlist expression.\neg: 'x1003c1s7b0n0,x1003c1s7b0n1'")
            .required(true),
        )
        .arg(arg!(-d --"dry-run" "Simulate the operation without making changes").action(ArgAction::SetTrue))
        .arg(output_flag()),
    )
}
