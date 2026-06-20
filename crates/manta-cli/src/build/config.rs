//! Clap definitions for `manta config *` subcommands.

use clap::{Command, arg};

use super::output_flag;

pub fn subcommand_config() -> Command {
  let subcommand_config_set_hsm = Command::new("hsm")
    .about("Set the active node group")
    // ID preserved as "HSM_GROUP_NAME" for handler compatibility
    .arg(arg!(<HSM_GROUP_NAME> "Node group name").value_name("GROUP_NAME"));

  let subcommand_config_set_site = Command::new("site")
    .about("Set the active site")
    .arg(arg!(<SITE_NAME> "Site name"));

  let subcommand_config_set_log = Command::new("log")
    .about("Set the log verbosity level")
    .arg(
      arg!(<LOG_LEVEL> "Log verbosity level")
        .value_parser(["error", "warn", "info", "debug", "trace"]),
    );

  let subcommand_config_set_read_only = Command::new("read-only")
    .about("Refuse every backend-mutating command until unset");

  let subcommand_config_unset_hsm =
    Command::new("hsm").about("Clear the active node group");

  let subcommand_config_unset_auth =
    Command::new("auth").about("Clear the cached authentication token");

  let subcommand_config_unset_read_only = Command::new("read-only")
    .about("Allow backend-mutating commands again");

  Command::new("config")
    .arg_required_else_help(true)
    .about("Show or change CLI-side settings (active site, default node group, log level)")
    .subcommand(
      Command::new("show")
        .about("Show current configuration values")
        .arg(output_flag()),
    )
    .subcommand(
      Command::new("set")
        .arg_required_else_help(true)
        .about("Set a configuration value")
        .subcommand(subcommand_config_set_hsm)
        .subcommand(subcommand_config_set_site)
        .subcommand(subcommand_config_set_log)
        .subcommand(subcommand_config_set_read_only),
    )
    .subcommand(
      Command::new("unset")
        .arg_required_else_help(true)
        .about("Clear a configuration value")
        .subcommand(subcommand_config_unset_hsm)
        .subcommand(subcommand_config_unset_auth)
        .subcommand(subcommand_config_unset_read_only),
    )
}
