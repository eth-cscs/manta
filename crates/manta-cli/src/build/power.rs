//! Clap definitions for `manta power *` subcommands.

use clap::{ArgAction, Command, arg};

use super::{HOSTLIST_HELP, dry_run_flag};

/// Attach the per-group args for `power on group/cluster`.
fn add_power_on_group_args(cmd: Command) -> Command {
  cmd
    .arg_required_else_help(true)
    .arg(arg!(-R --reason <TEXT> "Reason for the power operation"))
    .arg(
      arg!(-y --"assume-yes" "Skip confirmation prompts")
        .action(ArgAction::SetTrue),
    )
    .arg(
      arg!(--"no-wait" "Return as soon as the transition is queued; don't poll for completion")
        .action(ArgAction::SetTrue),
    )
    .arg(
      arg!(-o --output <FORMAT> "Output format")
        .value_parser(["table", "json"])
        .default_value("table"),
    )
    .arg(dry_run_flag())
    .arg(arg!(<GROUP_NAME> "Group name"))
}

/// Attach the per-group args for `power off group/cluster`.
fn add_power_off_group_args(cmd: Command) -> Command {
  cmd
    .arg_required_else_help(true)
    .arg(
      arg!(-g --graceful "Perform a graceful shutdown")
        .action(ArgAction::SetTrue),
    )
    .arg(arg!(-R --reason <TEXT> "Reason for the power operation"))
    .arg(
      arg!(-y --"assume-yes" "Skip confirmation prompts")
        .action(ArgAction::SetTrue),
    )
    .arg(
      arg!(--"no-wait" "Return as soon as the transition is queued; don't poll for completion")
        .action(ArgAction::SetTrue),
    )
    .arg(
      arg!(-o --output <FORMAT> "Output format")
        .value_parser(["table", "json"])
        .default_value("table"),
    )
    .arg(dry_run_flag())
    .arg(arg!(<GROUP_NAME> "Group name"))
}

/// Attach the per-group args for `power reset group/cluster`.
fn add_power_reset_group_args(cmd: Command) -> Command {
  cmd
    .arg_required_else_help(true)
    .arg(
      arg!(-g --graceful "Perform a graceful reboot")
        .action(ArgAction::SetTrue),
    )
    .arg(
      arg!(-y --"assume-yes" "Skip confirmation prompts")
        .action(ArgAction::SetTrue),
    )
    .arg(
      arg!(--"no-wait" "Return as soon as the transition is queued; don't poll for completion")
        .action(ArgAction::SetTrue),
    )
    .arg(
      arg!(-o --output <FORMAT> "Output format")
        .value_parser(["table", "json"])
        .default_value("table"),
    )
    .arg(dry_run_flag())
    .arg(arg!(-r --reason <TEXT> "Reason for the power operation"))
    .arg(arg!(<GROUP_NAME> "Group name"))
}

pub fn subcommand_power() -> Command {
  Command::new("power")
    .arg_required_else_help(true)
    .about("Power nodes on, off, or reset (reboot); waits for the transition unless --no-wait is set")
    .subcommand(
      Command::new("on")
        .arg_required_else_help(true)
        .about("Power on nodes or a group")
        .subcommand(
          add_power_on_group_args(Command::new("group"))
            .about("Power on all nodes in a group"),
        )
        .subcommand(
          Command::new("nodes")
            .arg_required_else_help(true)
            .about("Power on a set of nodes")
            .arg(
              arg!(-y --"assume-yes" "Skip confirmation prompts")
                .action(ArgAction::SetTrue),
            )
            .arg(
              arg!(--"no-wait" "Return as soon as the transition is queued; don't poll for completion")
                .action(ArgAction::SetTrue),
            )
            .arg(
              arg!(-o --output <FORMAT> "Output format")
                .value_parser(["table", "json"])
                .default_value("table"),
            )
            .arg(dry_run_flag())
            // ID preserved as "VALUE" for handler compatibility
            .arg(arg!(<VALUE>).value_name("NODES").help(HOSTLIST_HELP)),
        ),
    )
    .subcommand(
      Command::new("off")
        .arg_required_else_help(true)
        .about("Power off nodes or a group")
        .subcommand(
          add_power_off_group_args(Command::new("group"))
            .about("Power off all nodes in a group"),
        )
        .subcommand(
          Command::new("nodes")
            .arg_required_else_help(true)
            .about("Power off a set of nodes")
            .arg(
              arg!(-g --graceful "Perform a graceful shutdown")
                .action(ArgAction::SetTrue),
            )
            .arg(
              arg!(-y --"assume-yes" "Skip confirmation prompts")
                .action(ArgAction::SetTrue),
            )
            .arg(
              arg!(--"no-wait" "Return as soon as the transition is queued; don't poll for completion")
                .action(ArgAction::SetTrue),
            )
            .arg(
              arg!(-o --output <FORMAT> "Output format")
                .value_parser(["table", "json"])
                .default_value("table"),
            )
            .arg(dry_run_flag())
            // ID preserved as "VALUE" for handler compatibility
            .arg(arg!(<VALUE>).value_name("NODES").help(HOSTLIST_HELP)),
        ),
    )
    .subcommand(
      Command::new("reset")
        .arg_required_else_help(true)
        .about("Reset (reboot) nodes or a group")
        .subcommand(
          add_power_reset_group_args(Command::new("group"))
            .about("Reset all nodes in a group"),
        )
        .subcommand(
          Command::new("nodes")
            .arg_required_else_help(true)
            .about("Reset a set of nodes")
            .arg(
              arg!(-g --graceful "Perform a graceful reboot")
                .action(ArgAction::SetTrue),
            )
            .arg(
              arg!(-y --"assume-yes" "Skip confirmation prompts")
                .action(ArgAction::SetTrue),
            )
            .arg(
              arg!(--"no-wait" "Return as soon as the transition is queued; don't poll for completion")
                .action(ArgAction::SetTrue),
            )
            .arg(
              arg!(-o --output <FORMAT> "Output format")
                .value_parser(["table", "json"])
                .default_value("table"),
            )
            .arg(dry_run_flag())
            // ID preserved as "VALUE" for handler compatibility
            .arg(arg!(<VALUE>).value_name("NODES").help(HOSTLIST_HELP)),
        ),
    )
}
