//! Clap command tree definition for the manta CLI.
//!
//! Top-level `build_cli()` lives here. The four largest command
//! families (get, apply, delete, add) are split into sibling modules
//! for discoverability; the small ones (config, migrate, power, log,
//! console, update, and the two "nodes-to/from-groups" misc commands)
//! stay inline.

use clap::{ArgAction, Command, ValueHint, arg, value_parser};

use std::path::PathBuf;

mod add;
mod apply;
mod delete;
mod get;

const CLI_TERM_WIDTH: usize = 100;

/// Shared help text for arguments that accept xnames, NIDs, or hostlist expressions.
pub(super) const HOSTLIST_HELP: &str = "Xnames, NIDs, or a hostlist expression.\n\
  eg: 'x1003c1s7b0n0,x1003c1s7b0n1', 'nid001313,nid001314',\n\
  'x1003c1s7b0n[0-1],x1003c1s7b1n0', 'nid00131[0-9]'";

/// Standard `-o/--output {table,json}` flag with `table` as default.
/// Mutating commands consume the result via
/// `crate::cli::output::action_result::print`; read commands wire it
/// into their per-resource renderer in `crate::cli::output::*`.
pub(super) fn output_flag() -> clap::Arg {
  arg!(-o --output <FORMAT> "Output format")
    .value_parser(["table", "json"])
    .default_value("table")
}

/// Build the clap CLI command tree for manta.
pub fn build_cli() -> Command {
  // Hard-coded "manta" rather than `env!("CARGO_PKG_NAME")` because the
  // package is `manta-cli` but the produced binary is `manta` (see the
  // `[[bin]]` block in this crate's Cargo.toml). Using the package name
  // would render wrong help text and produce wrong completion-script
  // names.
  Command::new("manta")
    .term_width(CLI_TERM_WIDTH)
    .version(env!("CARGO_PKG_VERSION"))
    .arg_required_else_help(true)
    .arg(
      arg!(--site <SITE_NAME> "Override the active site for this invocation")
        .required(false),
    )
    .subcommand(subcommand_config())
    .subcommand(get::subcommand_get())
    .subcommand(add::subcommand_add())
    .subcommand(subcommand_update())
    .subcommand(apply::subcommand_apply())
    .subcommand(delete::subcommand_delete())
    .subcommand(subcommand_migrate())
    .subcommand(subcommand_power())
    .subcommand(subcommand_log())
    .subcommand(subcommand_console())
    .subcommand(add::subcommand_add_nodes_to_groups())
    .subcommand(subcommand_remove_nodes_from_groups())
}

fn subcommand_config() -> Command {
  let subcommand_config_set_hsm = Command::new("hsm")
    .about("Set the active node group")
    // ID preserved as "HSM_GROUP_NAME" for handler compatibility
    .arg(arg!(<HSM_GROUP_NAME> "Node group name").value_name("GROUP_NAME"));

  let subcommand_config_set_parent_hsm = Command::new("parent-hsm")
    .about("Set the parent node group")
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

  let subcommand_config_unset_hsm =
    Command::new("hsm").about("Clear the active node group");

  let subcommand_config_unset_parent_hsm =
    Command::new("parent-hsm").about("Clear the parent node group");

  let subcommand_config_unset_auth =
    Command::new("auth").about("Clear the cached authentication token");

  Command::new("config")
    .arg_required_else_help(true)
    .about("Manage manta CLI configuration")
    .subcommand(Command::new("show").about("Show current configuration values"))
    .subcommand(
      Command::new("set")
        .arg_required_else_help(true)
        .about("Set a configuration value")
        .subcommand(subcommand_config_set_hsm)
        .subcommand(subcommand_config_set_parent_hsm)
        .subcommand(subcommand_config_set_site)
        .subcommand(subcommand_config_set_log),
    )
    .subcommand(
      Command::new("unset")
        .arg_required_else_help(true)
        .about("Clear a configuration value")
        .subcommand(subcommand_config_unset_hsm)
        .subcommand(subcommand_config_unset_parent_hsm)
        .subcommand(subcommand_config_unset_auth),
    )
    .subcommand(
      Command::new("gen-autocomplete")
        .about("Generate shell completion scripts")
        .arg(
          arg!(-s --shell <SHELL> "Shell type (guessed from $SHELL if omitted)")
            .value_parser(["bash", "zsh", "fish"]),
        )
        .arg(
          arg!(-p --path <PATH> "Directory to write the script (prints to stdout if omitted)")
            .value_parser(value_parser!(PathBuf))
            .value_hint(ValueHint::DirPath),
        ),
    )
}

fn subcommand_migrate_backup() -> Command {
  Command::new("backup")
    .arg_required_else_help(true)
    .about("Back up a cluster's configuration")
    .long_about(
      "Back up a cluster's configuration: images, boot settings, and group membership.\n\n\
      The backup is derived from the specified session template.",
    )
    .arg(arg!(-b --"bos" <SESSIONTEMPLATE> "Session template to derive the backup from"))
    .arg(
      arg!(-d --"destination" <FOLDER> "Destination directory for the backup files")
        .value_hint(ValueHint::DirPath),
    )
    .arg(arg!(-p --"pre-hook" <SCRIPT> "Command to run before the backup.\neg: --pre-hook \"echo hello\""))
    .arg(
      arg!(-a --"post-hook" <SCRIPT> "Command to run after a successful backup.\neg: --post-hook \"echo hello\""),
    )
}

fn subcommand_migrate_restore() -> Command {
  Command::new("restore")
    .arg_required_else_help(true)
    .about("Restore a cluster from a backup")
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
}

fn subcommand_power() -> Command {
  Command::new("power")
    .arg_required_else_help(true)
    .about("Manage node and cluster power state")
    .subcommand(
      Command::new("on")
        .arg_required_else_help(true)
        .about("Power on nodes or a cluster")
        .subcommand(
          Command::new("cluster")
            .arg_required_else_help(true)
            .about("Power on all nodes in a cluster")
            .arg(arg!(-R --reason <TEXT> "Reason for the power operation"))
            .arg(
              arg!(-y --"assume-yes" "Skip confirmation prompts")
                .action(ArgAction::SetTrue),
            )
            .arg(
              arg!(-o --output <FORMAT> "Output format")
                .value_parser(["table", "json"])
                .default_value("table"),
            )
            .arg(arg!(<CLUSTER_NAME> "Cluster name")),
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
              arg!(-o --output <FORMAT> "Output format")
                .value_parser(["table", "json"])
                .default_value("table"),
            )
            // ID preserved as "VALUE" for handler compatibility
            .arg(arg!(<VALUE>).value_name("NODES").help(HOSTLIST_HELP)),
        ),
    )
    .subcommand(
      Command::new("off")
        .arg_required_else_help(true)
        .about("Power off nodes or a cluster")
        .subcommand(
          Command::new("cluster")
            .arg_required_else_help(true)
            .about("Power off all nodes in a cluster")
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
              arg!(-o --output <FORMAT> "Output format")
                .value_parser(["table", "json"])
                .default_value("table"),
            )
            .arg(arg!(<CLUSTER_NAME> "Cluster name")),
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
              arg!(-o --output <FORMAT> "Output format")
                .value_parser(["table", "json"])
                .default_value("table"),
            )
            // ID preserved as "VALUE" for handler compatibility
            .arg(arg!(<VALUE>).value_name("NODES").help(HOSTLIST_HELP)),
        ),
    )
    .subcommand(
      Command::new("reset")
        .arg_required_else_help(true)
        .about("Reset (reboot) nodes or a cluster")
        .subcommand(
          Command::new("cluster")
            .arg_required_else_help(true)
            .about("Reset all nodes in a cluster")
            .arg(
              arg!(-g --graceful "Perform a graceful reboot")
                .action(ArgAction::SetTrue),
            )
            .arg(
              arg!(-y --"assume-yes" "Skip confirmation prompts")
                .action(ArgAction::SetTrue),
            )
            .arg(
              arg!(-o --output <FORMAT> "Output format")
                .value_parser(["table", "json"])
                .default_value("table"),
            )
            .arg(arg!(-r --reason <TEXT> "Reason for the power operation"))
            .arg(arg!(<CLUSTER_NAME> "Cluster name")),
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
              arg!(-o --output <FORMAT> "Output format")
                .value_parser(["table", "json"])
                .default_value("table"),
            )
            // ID preserved as "VALUE" for handler compatibility
            .arg(arg!(<VALUE>).value_name("NODES").help(HOSTLIST_HELP)),
        ),
    )
}

fn subcommand_log() -> Command {
  Command::new("log")
    .alias("logs")
    .about("Stream configuration session logs")
    .arg(arg!(-t --timestamps "Show log timestamps").action(ArgAction::SetTrue))
    // ID preserved as "VALUE" for handler compatibility
    .arg(
      arg!([VALUE] "Session name, node group, xname, or NID.\neg: x1003c1s7b0n0, nid001313, zinal, batcher-64d35a81-d0e1-496d-9eda-0010e502f2a3")
        .value_name("TARGET"),
    )
}

fn subcommand_update_boot_parameters() -> Command {
  Command::new("boot-parameters")
    .arg_required_else_help(true)
    .about("Update boot parameters for nodes")
    .arg(
      arg!(-H --"hosts" <XNAMES> "Xnames of the nodes to update")
        .required(true),
    )
    .arg(arg!(-p --"params" <VALUE> "Kernel parameters"))
    .arg(arg!(-k --"kernel" <VALUE> "S3 path to the kernel file"))
    .arg(arg!(-i --"initrd" <VALUE> "S3 path to the initrd file"))
    .arg(
      arg!(-d --"dry-run" "Simulate the operation without making changes")
        .action(ArgAction::SetTrue),
    )
    .arg(
      arg!(-y --"assume-yes" "Skip confirmation prompts")
        .action(ArgAction::SetTrue),
    )
}

fn subcommand_update_redfish_endpoint() -> Command {
  Command::new("redfish-endpoints")
    .visible_alias("redfish-endpoint")
    .arg_required_else_help(true)
    .about("Update a Redfish endpoint")
    .arg(arg!(-i --id <XNAME> "Xname of the endpoint to update").required(true))
    .arg(arg!(-n --name <VALUE> "Arbitrary user-provided name for the endpoint"))
    .arg(arg!(-H --hostname <VALUE> "Hostname (FQDN host portion)"))
    .arg(arg!(-d --domain <VALUE> "Domain (FQDN domain portion)"))
    .arg(
      arg!(-f --fqdn <VALUE> "Fully-qualified domain name on the management network"),
    )
    .arg(arg!(-e --enabled "Enable the endpoint").action(ArgAction::SetTrue))
    .arg(arg!(-u --user <VALUE> "Username for endpoint authentication"))
    .arg(arg!(-p --password <VALUE> "Password for endpoint authentication"))
    .arg(arg!(-U --"use-ssdp" "Use SSDP for discovery if the endpoint supports it").action(ArgAction::SetTrue))
    .arg(arg!(-m --"mac-required" "Require a MAC address for geolocation").action(ArgAction::SetTrue))
    .arg(arg!(-M --macaddr <VALUE> "MAC address of the Redfish endpoint on the management network"))
    .arg(
      arg!(-I --ipaddress <VALUE> "IP address of the Redfish endpoint on the management network (IPv4 or IPv6)"),
    )
    .arg(
      arg!(-r --"rediscover-on-update" "Trigger rediscovery when endpoint information is updated")
        .action(ArgAction::SetTrue),
    )
    .arg(arg!(-t --"template-id" <VALUE> "Discovery template ID"))
    .arg_required_else_help(true)
}

fn subcommand_update() -> Command {
  Command::new("update")
    .arg_required_else_help(true)
    .about("Update system resources")
    .subcommand(subcommand_update_boot_parameters())
    .subcommand(subcommand_update_redfish_endpoint())
}

fn subcommand_migrate() -> Command {
  Command::new("migrate")
    .arg_required_else_help(true)
    .about("Move nodes or clusters between groups")
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
        .arg(arg!(-d --"dry-run" "Simulate the operation without making changes").action(ArgAction::SetTrue)),
    )
}

fn subcommand_console() -> Command {
  Command::new("console")
    .arg_required_else_help(true)
    .about("Open an interactive console to a node or configuration session")
    .subcommand(
      Command::new("node")
        .about("Connect to a node's serial console")
        .long_about(
          "Connect to a node's serial console.\n\nAccepts an xname or NID.\n\
          eg: 'x1003c1s7b0n0' or 'nid001313'",
        )
        .arg(arg!(<XNAME> "Node xname or NID").required(true)),
    )
    .subcommand(
      Command::new("target-ansible")
        .arg_required_else_help(true)
        .about(
          "Connect to the Ansible target container of a configuration session",
        )
        .arg(arg!(<SESSION_NAME> "Configuration session name").required(true)),
    )
}

fn subcommand_remove_nodes_from_groups() -> Command {
  Command::new("remove-nodes-from-groups")
    .about("Remove nodes from one or more groups")
    .arg(arg!(-g --group <NAME> "Group to remove the nodes from"))
    .arg(arg!(-n --nodes <NODES>).help(HOSTLIST_HELP))
    .arg(
      arg!(-d --"dry-run" "Simulate the operation without making changes")
        .action(ArgAction::SetTrue),
    )
    .arg(output_flag())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn site_flag_is_optional() {
    let _matches =
      build_cli().try_get_matches_from(["manta", "get", "sessions", "--help"]);
    let result = build_cli().try_get_matches_from(["manta", "--version"]);
    assert!(
      result.is_err(),
      "expected DisplayVersion error for --version"
    );
  }

  #[test]
  fn site_flag_accepted_before_subcommand() {
    let result = build_cli()
      .try_get_matches_from(["manta", "--site", "alps", "get", "--help"]);
    match result {
      Err(e) => assert_eq!(
        e.kind(),
        clap::error::ErrorKind::DisplayHelp,
        "expected DisplayHelp, got: {e}"
      ),
      Ok(_) => panic!("--help should cause an early exit"),
    }
  }

  #[test]
  fn site_flag_value_is_extractable() {
    let matches = build_cli()
      .get_matches_from(["manta", "--site", "prealps", "config", "show"]);
    let site = matches.get_one::<String>("site");
    assert_eq!(site.map(String::as_str), Some("prealps"));
  }

  #[test]
  fn site_flag_absent_returns_none() {
    let matches = build_cli().get_matches_from(["manta", "config", "show"]);
    let site = matches.get_one::<String>("site");
    assert!(site.is_none());
  }

  #[test]
  fn site_flag_is_root_level_only() {
    let matches = build_cli()
      .get_matches_from(["manta", "--site", "alps", "config", "show"]);
    let site = matches.get_one::<String>("site");
    assert_eq!(site.map(String::as_str), Some("alps"));
  }
}
