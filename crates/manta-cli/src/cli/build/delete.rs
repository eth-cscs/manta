//! Clap definitions for `manta delete *` subcommands.

use clap::{ArgAction, ArgGroup, Command, arg};

use super::HOSTLIST_HELP;

pub fn subcommand_delete() -> Command {
  Command::new("delete")
    .arg_required_else_help(true)
    .about("Delete system resources")
    .subcommand(subcommand_delete_group())
    .subcommand(subcommand_delete_node())
    .subcommand(subcommand_delete_kernel_parameter())
    .subcommand(subcommand_delete_boot_parameter())
    .subcommand(subcommand_delete_configuration())
    .subcommand(subcommand_delete_session())
    .subcommand(subcommand_delete_image())
    .subcommand(subcommand_delete_hw_component())
    .subcommand(subcommand_delete_redfish_endpoint())
}

pub fn subcommand_delete_group() -> Command {
  Command::new("group")
    .arg_required_else_help(true)
    .about("Delete a node group")
    .long_about(
      "Delete a node group.\n\n\
      The group must be empty before deletion. \
      Move its members to another group with 'migrate nodes' first.",
    )
    .arg(arg!(-f --force "Force deletion").action(ArgAction::SetTrue))
    // ID preserved as "VALUE" for handler compatibility
    .arg(
      arg!(<VALUE> "Name of the group to delete")
        .value_name("GROUP_NAME")
        .required(true),
    )
}

pub fn subcommand_delete_node() -> Command {
  Command::new("node")
    .arg_required_else_help(true)
    .about("Remove a node from the system")
    // ID preserved as "VALUE" for handler compatibility
    .arg(
      arg!(<VALUE> "Xname of the node to remove")
        .value_name("XNAME")
        .required(true),
    )
}

pub fn subcommand_delete_hw_component() -> Command {
  Command::new("hardware")
    .arg_required_else_help(true)
    .about("[experimental] Remove hardware components from a cluster")
    .arg(arg!(-P --pattern <PATTERN> "Hardware component pattern"))
    .arg(arg!(-t --"target-cluster" <TARGET_CLUSTER_NAME> "Cluster to remove components from"))
    .arg(arg!(-p --"parent-cluster" <PARENT_CLUSTER_NAME> "Cluster that receives the freed components"))
    .arg(arg!(-d --"dry-run" "Simulate the operation without making changes").action(ArgAction::SetTrue))
    .arg(arg!(-D --"delete-hsm-group" "Delete the group if empty after this operation").action(ArgAction::SetTrue))
}

pub fn subcommand_delete_image() -> Command {
  Command::new("images")
    .arg_required_else_help(true)
    .about("[experimental] Delete images")
    .arg(arg!(-d --"dry-run" "Simulate the operation without making changes").action(ArgAction::SetTrue))
    // ID preserved as "IMAGE_LIST" for handler compatibility
    .arg(
      arg!(<IMAGE_LIST> "Comma-separated image IDs to delete.\neg: e2ce82f0-e7ba-4f36-9f5c-750346599600,59e0180a-3fdd-4936-bba7-14ba914ffd34")
        .value_name("IMAGE_IDS")
        .required(true),
    )
}

pub fn subcommand_delete_configuration() -> Command {
  Command::new("configurations")
    .arg_required_else_help(true)
    .about("Delete configurations and all associated data")
    .long_about(
      "Delete configurations and all associated data (sessions, session templates, boot sessions, and images).\n\n\
      Use --since and/or --until to filter by the configuration's last-updated date (format: %Y-%m-%d).\n\n\
      eg:\n  \
      --since 2023-01-01 --until 2023-10-01\n    \
      Deletes all configurations last updated between 2023-01-01 and 2023-10-01.",
    )
    .arg(arg!(-n --"configuration-name" <VALUE> "Glob pattern to filter by name.\neg: my-config*, my-config-v[1,2]"))
    .arg(arg!(-s --since <DATE> "Delete configurations last updated after this date (format: %Y-%m-%d)"))
    .arg(arg!(-u --until <DATE> "Delete configurations last updated before this date (format: %Y-%m-%d)"))
    .arg(arg!(-y --"assume-yes" "Skip confirmation prompts").action(ArgAction::SetTrue))
    .group(
      ArgGroup::new("since_and_until")
        .args(["since", "until"])
        .multiple(true)
        .requires("until")
        .conflicts_with("configuration-name"),
    )
}

pub fn subcommand_delete_session() -> Command {
  Command::new("session")
    .arg_required_else_help(true)
    .about("Delete a configuration session")
    .long_about(
      "Delete a configuration session.\n\n\
      For image-building sessions, the associated image is also removed.\n\
      For dynamic (runtime) sessions, the error count is set to its maximum value.",
    )
    .arg(arg!(-y --"assume-yes" "Skip confirmation prompts").action(ArgAction::SetTrue))
    .arg(arg!(-d --"dry-run" "Simulate the operation without making changes").action(ArgAction::SetTrue))
    .arg(arg!(<SESSION_NAME> "Name of the session to delete").required(true))
}

pub fn subcommand_delete_kernel_parameter() -> Command {
  Command::new("kernel-parameters")
    .arg_required_else_help(true)
    .about("Remove kernel parameters from nodes")
    .arg(arg!(-n --nodes <NODES>).help(HOSTLIST_HELP))
    .arg(arg!(-H --group <GROUP_NAME> "Node group name").visible_alias("hsm-group"))
    .arg(arg!(-y --"assume-yes" "Skip confirmation prompts").action(ArgAction::SetTrue))
    .arg(arg!(--"do-not-reboot" "Do not reboot nodes after applying changes").action(ArgAction::SetTrue))
    .arg(arg!(-d --"dry-run" "Simulate the operation without making changes").action(ArgAction::SetTrue))
    // ID preserved as "VALUE" for handler compatibility
    .arg(arg!(<VALUE> "Comma-separated kernel parameter names to remove.\neg: console,bad_page,crashkernel,hugepagelist,quiet").value_name("PARAMS"))
    .group(
      ArgGroup::new("cluster_or_nodes")
        .args(["group", "nodes"])
        .required(true),
    )
}

pub fn subcommand_delete_boot_parameter() -> Command {
  Command::new("boot-parameters")
    .arg_required_else_help(true)
    .about("Delete boot parameters for nodes")
    .arg(arg!(-H --hosts <XNAMES> "Xnames of the nodes"))
}

pub fn subcommand_delete_redfish_endpoint() -> Command {
  Command::new("redfish-endpoints")
    .visible_alias("redfish-endpoint")
    .arg_required_else_help(true)
    .about("Delete a Redfish endpoint")
    .arg(
      arg!(-i --id <XNAME> "Xname of the Redfish endpoint to delete")
        .required(true),
    )
}
