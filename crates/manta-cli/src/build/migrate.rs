//! Clap definitions for `manta migrate *` subcommands.
//!
//! Builds the `manta migrate nodes` command — atomically moves nodes
//! from one group to another (a `delete nodes` + `add nodes` pair).
//! Execution dispatched in `crate::dispatch::migrate`.

use clap::{Command, arg};

use super::{dry_run_flag, output_flag};

/// Top-level `manta migrate` verb.
pub fn subcommand_migrate() -> Command {
  Command::new("migrate")
    .arg_required_else_help(true)
    .about("Move nodes between groups")
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
        .arg(dry_run_flag())
        .arg(output_flag()),
    )
}
