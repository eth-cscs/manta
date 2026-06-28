//! Clap definitions for `manta log` (aliased as `logs`).
//!
//! Single-verb tail of CFS session logs. The positional argument is
//! polymorphic — a session name, group name, xname, or NID — and the
//! handler resolves the most recent session targeting that entity.
//! Handler: `crate::dispatch::log`.

use clap::{ArgAction, Command, arg};

/// Build `manta log` / `manta logs`.
pub fn subcommand_log() -> Command {
  Command::new("log")
    .alias("logs")
    .about("Stream configuration session logs to stdout (accepts session, node, group, or NID)")
    .arg(arg!(-t --timestamps "Show log timestamps").action(ArgAction::SetTrue))
    // ID preserved as "VALUE" for handler compatibility
    .arg(
      arg!([VALUE] "Session name, node group, xname, or NID.\neg: x1003c1s7b0n0, nid001313, zinal, batcher-64d35a81-d0e1-496d-9eda-0010e502f2a3")
        .value_name("TARGET"),
    )
}
