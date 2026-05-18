//! Clap definitions for `manta get *` subcommands.

use clap::{ArgAction, ArgGroup, Command, arg, value_parser};

use super::HOSTLIST_HELP;

pub fn subcommand_get_group() -> Command {
  Command::new("groups")
    .about("List node groups")
    // ID preserved as "VALUE" for handler compatibility
    .arg(
      arg!(<VALUE> "Group name (lists all groups if omitted)")
        .value_name("GROUP_NAME")
        .required(false),
    )
    .arg(
      arg!(-o --output <FORMAT> "Output format")
        .value_parser(["json", "table"])
        .default_value("table"),
    )
}

pub fn subcommand_get_hardware() -> Command {
  let command_get_hw_configuration_cluster = Command::new("cluster")
    .arg_required_else_help(true)
    .about("Show hardware inventory for a cluster")
    .arg(arg!(<CLUSTER_NAME> "Cluster name").required(true))
    .arg(
      arg!(-o --output <FORMAT> "Output format")
        .value_parser(["json", "summary", "details", "pattern"])
        .default_value("summary"),
    );

  let command_get_hw_nodes = Command::new("nodes")
    .arg_required_else_help(true)
    .about("Show hardware inventory for a set of nodes")
    // ID preserved as "VALUE" for handler compatibility
    .arg(arg!(<VALUE>).value_name("NODES").help(HOSTLIST_HELP))
    .arg(
      arg!(-o --output <FORMAT> "Output format")
        .value_parser(["table", "json"])
        .default_value("table"),
    );

  Command::new("hardware")
    .arg_required_else_help(true)
    .about("Inspect hardware components")
    .subcommand(command_get_hw_configuration_cluster)
    .subcommand(command_get_hw_nodes)
}

pub fn subcommand_get_cfs_configuration() -> Command {
  Command::new("configurations")
    .about("List configurations")
    .arg(arg!(-n --name <NAME> "Configuration name"))
    .arg(arg!(-p --pattern <PATTERN> "Glob pattern to filter by name"))
    .arg(arg!(-m --"most-recent" "Return only the most recent (equivalent to --limit 1)"))
    .arg(
      arg!(-l --limit <VALUE> "Return only the <VALUE> most recent configurations")
        .value_parser(value_parser!(u8).range(1..)),
    )
    .arg(arg!(-o --output <FORMAT> "Output format").value_parser(["json"]))
    .arg(arg!(-H --"hsm-group" <GROUP_NAME> "Node group name"))
    .group(ArgGroup::new("hsm-group_or_configuration").args(["hsm-group", "name"]))
    .group(ArgGroup::new("configuration_limit").args(["most-recent", "limit"]))
}

pub fn subcommand_get_cfs_session() -> Command {
  Command::new("sessions")
    .about("List configuration sessions")
    .arg(arg!(-n --name <SESSION_NAME> "Return only the session with this name"))
    .arg(arg!(-a --"min-age" <VALUE> "Return only sessions older than this age (eg: '1d', '6h')"))
    .arg(arg!(-A --"max-age" <VALUE> "Return only sessions younger than this age (eg: '1d', '6h')"))
    .arg(
      arg!(-t --type <VALUE> "Return only sessions of this type")
        .value_parser(["image", "runtime"]),
    )
    .arg(
      arg!(-s --status <VALUE> "Return only sessions with this status")
        .value_parser(["pending", "running", "complete"]),
    )
    .arg(arg!(-m --"most-recent" "Return only the most recent session (equivalent to --limit 1)"))
    .arg(
      arg!(-l --limit <VALUE> "Return only the <VALUE> most recent sessions")
        .value_parser(value_parser!(u8).range(1..)),
    )
    .arg(arg!(-o --output <FORMAT> "Output format").value_parser(["json"]))
    .arg(arg!(-x --xnames <NODES> "Xnames, NIDs, or hostlist expression. Returns sessions targeting these nodes or their groups"))
    .arg(arg!(-H --"hsm-group" <GROUP_NAME> "Node group name. Returns sessions targeting this group or its members"))
    .group(ArgGroup::new("hsm-group_or_xnames_or_name").args([
      "hsm-group",
      "xnames",
      "name",
    ]))
    .group(ArgGroup::new("session_limit").args(["most-recent", "limit"]))
}

pub fn subcommand_get_bos_template() -> Command {
  Command::new("templates")
    .about("List session templates")
    .arg(arg!(-n --name <NAME> "Template name"))
    .arg(arg!(-m --"most-recent" "Return only the most recent (equivalent to --limit 1)"))
    .arg(
      arg!(-l --limit <VALUE> "Return only the <VALUE> most recent templates")
        .value_parser(value_parser!(u8).range(1..)),
    )
    .arg(arg!(-H --"hsm-group" <GROUP_NAME> "Node group name"))
    .arg(
      arg!(-o --output <FORMAT> "Output format")
        .value_parser(["json", "table"])
        .default_value("table"),
    )
    .group(ArgGroup::new("hsm-group_or_template").args(["hsm-group", "name"]))
}

pub fn subcommand_get_cluster_details() -> Command {
  Command::new("cluster")
    .about("Show cluster node details and status")
    .arg(arg!(-n --"nids-only-one-line" "Print NIDs on a single line").action(ArgAction::SetTrue))
    .arg(arg!(-x --"xnames-only-one-line" "Print xnames on a single line").action(ArgAction::SetTrue))
    .arg(
      arg!(-s --status <VALUE> "Filter nodes by status")
        .value_parser(["OFF", "ON", "READY", "STANDBY", "PENDING", "FAILED", "CONFIGURED"]),
    )
    .arg(
      arg!(-T --"summary-status" "Show a cluster status summary:\n\
        OK          — all nodes booted and configured\n\
        OFF         — at least one node is OFF\n\
        ON          — no nodes OFF, at least one is ON\n\
        STANDBY     — at least one node's heartbeat is lost\n\
        UNCONFIGURED — all nodes READY but at least one is still being configured\n\
        FAILED      — at least one node's configuration failed")
        .action(ArgAction::SetTrue),
    )
    .arg(
      arg!(-o --output <FORMAT> "Output format")
        .value_parser(["table", "table-wide", "json", "summary"])
        .default_value("table"),
    )
    .arg_required_else_help(true)
    // ID preserved as "HSM_GROUP_NAME" for handler compatibility
    .arg(arg!(<HSM_GROUP_NAME> "Cluster name").value_name("CLUSTER_NAME"))
}

pub fn subcommand_get_node_details() -> Command {
  Command::new("nodes")
    .about("Show node details and status")
    .arg(arg!(-n --"nids-only-one-line" "Print NIDs on a single line"))
    .arg(
      arg!(-s --status <VALUE> "Filter nodes by status")
        .value_parser(["OFF", "ON", "READY", "STANDBY", "PENDING", "FAILED", "CONFIGURED"]),
    )
    .arg(
      arg!(-T --"summary-status" "Show a node status summary:\n\
        OK          — all nodes booted and configured\n\
        OFF         — at least one node is OFF\n\
        ON          — no nodes OFF, at least one is ON\n\
        STANDBY     — at least one node's heartbeat is lost\n\
        UNCONFIGURED — all nodes READY but at least one is still being configured\n\
        FAILED      — at least one node's configuration failed")
        .action(ArgAction::SetTrue),
    )
    .arg(
      arg!(-S --"include-siblings" "Also show sibling nodes that share a power supply with the requested nodes")
        .action(ArgAction::SetTrue),
    )
    .arg(
      arg!(-o --output <FORMAT> "Output format")
        .value_parser(["table", "table-wide", "json", "summary"])
        .default_value("table"),
    )
    .arg_required_else_help(true)
    // ID preserved as "VALUE" for handler compatibility
    .arg(arg!(<VALUE>).value_name("NODES").help(HOSTLIST_HELP))
}

pub fn subcommand_get_images() -> Command {
  Command::new("images")
    .about("List images")
    .arg(arg!(-i --id <IMAGE_ID> "Image ID"))
    .arg(arg!(-m --"most-recent" "Return only the most recent (equivalent to --limit 1)"))
    .arg(
      arg!(-l --limit <VALUE> "Return only the <VALUE> most recent images")
        .value_parser(value_parser!(u8).range(1..)),
    )
    .arg(arg!(-H --"hsm-group" <GROUP_NAME> "Node group name"))
}

pub fn subcommand_get_boot_parameters() -> Command {
  Command::new("boot-parameters")
    .arg_required_else_help(true)
    .about("Show boot parameters for nodes or a group")
    .arg(arg!(-H --"hsm-group" <GROUP_NAME> "Node group name"))
    .arg(arg!(-n --nodes <NODES>).help(HOSTLIST_HELP))
}

pub fn subcommand_get_kernel_parameters() -> Command {
  Command::new("kernel-parameters")
    .about("Show kernel parameters for nodes or a group")
    .arg(arg!(-n --nodes <NODES>).help(HOSTLIST_HELP))
    .arg(arg!(-H --"hsm-group" <GROUP_NAME> "Show kernel parameters for all nodes in this group"))
    .arg(
      arg!(-f --filter <VALUE> "Comma-separated list of parameter names to show.\neg: 'console,bad_page,crashkernel,hugepagelist,root'"),
    )
    .arg(
      arg!(-o --output <FORMAT> "Output format")
        .value_parser(["table", "json"])
        .default_value("table"),
    )
    .group(
      ArgGroup::new("hsm-group_or_nodes")
        .args(["hsm-group", "nodes"])
        .required(true),
    )
}

pub fn subcommand_get_redfish_endpoints() -> Command {
  Command::new("redfish-endpoints")
    .about("List Redfish endpoints")
    .arg(arg!(-i --id <VALUE> "Filter by xname (can be specified multiple times)"))
    .arg(arg!(-f --fqdn <VALUE> "Filter by FQDN"))
    .arg(arg!(-u --uuid <VALUE> "Filter by UUID"))
    .arg(arg!(-m --macaddr <VALUE> "Filter by MAC address"))
    .arg(
      arg!(-I --ipaddress <VALUE> "Filter by IP address (empty string matches endpoints without an IP)"),
    )
    .arg(
      arg!(-o --output <FORMAT> "Output format")
        .value_parser(["table", "json"])
        .default_value("table"),
    )
}

pub fn subcommand_get() -> Command {
  Command::new("get")
    .arg_required_else_help(true)
    .about("Query system resources")
    .subcommand(subcommand_get_group())
    .subcommand(subcommand_get_hardware())
    .subcommand(subcommand_get_cfs_session())
    .subcommand(subcommand_get_cfs_configuration())
    .subcommand(subcommand_get_bos_template())
    .subcommand(subcommand_get_cluster_details())
    .subcommand(subcommand_get_node_details())
    .subcommand(subcommand_get_images())
    .subcommand(subcommand_get_boot_parameters())
    .subcommand(subcommand_get_kernel_parameters())
    .subcommand(subcommand_get_redfish_endpoints())
}
