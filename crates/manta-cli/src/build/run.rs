//! Clap definitions for `manta run *` subcommands.
//!
//! Hosts the shared `add_run_session_args` builder, also used by the
//! deprecated `manta apply session` alias in [`super::apply`].

use clap::{ArgAction, ArgGroup, Command, ValueHint, arg, value_parser};
use std::path::PathBuf;

use super::output_flag;

/// Attach the session-run argument set to a clap `Command`. Shared
/// between the canonical `manta run session` and the deprecated
/// `manta apply session` paths so both stay in lockstep.
pub(super) fn add_run_session_args(cmd: Command) -> Command {
  cmd
    .arg_required_else_help(true)
    .arg(arg!(-n --name <VALUE> "Session name").required(true))
    .arg(
      arg!(-p --"playbook-name" <VALUE> "Ansible playbook filename")
        .default_value("site.yml"),
    )
    .arg(
      arg!(-r --"repo-path" <REPO_PATH> ... "Path to the local git repo containing the Ansible playbook")
        .required(true)
        .value_parser(value_parser!(PathBuf))
        .value_hint(ValueHint::DirPath),
    )
    .arg(arg!(-w --"watch-logs" "Stream session logs to stdout").action(ArgAction::SetTrue))
    .arg(arg!(-t --timestamps "Show log timestamps").action(ArgAction::SetTrue))
    .arg(
      arg!(-v --"ansible-verbosity" <VALUE> "Ansible verbosity level (1 = -v, 2 = -vv, …, max 4)")
        .value_parser(["0", "1", "2", "3", "4"])
        .num_args(1)
        .default_value("2")
        .default_missing_value("2"),
    )
    .arg(
      arg!(-P --"ansible-passthrough" <VALUE>
        "Additional Ansible flags (limited to --extra-vars, --forks, --skip-tags, --start-at-task, --tags)")
        .allow_hyphen_values(true),
    )
    .arg(
      arg!(-l --"ansible-limit" <VALUE>
        "Limit the session to specific nodes (must be a subset of --group if both are provided)")
        .required(true),
    )
    .arg(
      arg!(-H --group <GROUP_NAME> "Run the session against every node in this group")
        .visible_alias("hsm-group"),
    )
    .group(
      ArgGroup::new("hsm-group_or_ansible-limit")
        .args(["group", "ansible-limit"])
        .required(true),
    )
    .arg(output_flag())
}

/// Top-level `manta run` verb.
pub fn subcommand_run() -> Command {
  Command::new("run")
    .arg_required_else_help(true)
    .about("Create and run a configuration session from a local Ansible repo")
    .subcommand(
      add_run_session_args(Command::new("session"))
        .about("Create and run a configuration session from a local repo")
        .long_about(
          "Create and run a configuration session from a local git repo.\n\n\
          The repo must already exist in the system's VCS. The session runs \
          the specified Ansible playbook against the target nodes or group.",
        ),
    )
}
