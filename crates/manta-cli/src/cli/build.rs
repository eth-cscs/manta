//! Clap command tree definition for the manta CLI.

use clap::{arg, value_parser, ArgAction, ArgGroup, Command, ValueHint};

use std::path::PathBuf;

const CLI_TERM_WIDTH: usize = 100;

/// Shared help text for arguments that accept xnames, NIDs, or hostlist expressions.
const HOSTLIST_HELP: &str = "Xnames, NIDs, or a hostlist expression.\n\
  eg: 'x1003c1s7b0n0,x1003c1s7b0n1', 'nid001313,nid001314',\n\
  'x1003c1s7b0n[0-1],x1003c1s7b1n0', 'nid00131[0-9]'";

/// Build the clap CLI command tree for manta.
pub fn build_cli() -> Command {
  Command::new(env!("CARGO_PKG_NAME"))
    .term_width(CLI_TERM_WIDTH)
    .version(env!("CARGO_PKG_VERSION"))
    .arg_required_else_help(true)
    .arg(
      arg!(--site <SITE_NAME> "Override the active site for this invocation")
        .required(false),
    )
    .subcommand(subcommand_config())
    .subcommand(subcommand_get())
    .subcommand(subcommand_add())
    .subcommand(subcommand_update())
    .subcommand(subcommand_apply())
    .subcommand(subcommand_delete())
    .subcommand(subcommand_migrate())
    .subcommand(subcommand_power())
    .subcommand(subcommand_log())
    .subcommand(subcommand_console())
    .subcommand(subcommand_add_nodes_to_groups())
    .subcommand(subcommand_remove_nodes_from_groups())
    .subcommand(subcommand_serve())
}

fn subcommand_serve() -> Command {
  Command::new("serve")
    .about("Run manta as an HTTP/HTTPS API server")
    .arg(
      arg!(--port <PORT> "Port to listen on")
        .default_value("8080")
        .value_parser(value_parser!(u16)),
    )
    .arg(
      arg!(--cert <CERT_FILE> "Path to TLS certificate PEM file (enables HTTPS)")
        .required(false)
        .value_hint(ValueHint::FilePath),
    )
    .arg(
      arg!(--key <KEY_FILE> "Path to TLS private key PEM file (required with --cert)")
        .required(false)
        .value_hint(ValueHint::FilePath),
    )
    .arg(
      arg!(--"listen-address" <ADDRESS> "Address to bind to")
        .default_value("0.0.0.0"),
    )
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

fn subcommand_delete() -> Command {
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

fn subcommand_delete_group() -> Command {
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
    .arg(arg!(<VALUE> "Name of the group to delete").value_name("GROUP_NAME").required(true))
}

fn subcommand_delete_node() -> Command {
  Command::new("node")
    .arg_required_else_help(true)
    .about("Remove a node from the system")
    // ID preserved as "VALUE" for handler compatibility
    .arg(arg!(<VALUE> "Xname of the node to remove").value_name("XNAME").required(true))
}

fn subcommand_delete_hw_component() -> Command {
  Command::new("hardware")
    .arg_required_else_help(true)
    .about("[experimental] Remove hardware components from a cluster")
    .arg(arg!(-P --pattern <PATTERN> "Hardware component pattern"))
    .arg(arg!(-t --"target-cluster" <TARGET_CLUSTER_NAME> "Cluster to remove components from"))
    .arg(arg!(-p --"parent-cluster" <PARENT_CLUSTER_NAME> "Cluster that receives the freed components"))
    .arg(arg!(-d --"dry-run" "Simulate the operation without making changes").action(ArgAction::SetTrue))
    .arg(arg!(-D --"delete-hsm-group" "Delete the group if empty after this operation").action(ArgAction::SetTrue))
}

fn subcommand_delete_image() -> Command {
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

fn subcommand_delete_configuration() -> Command {
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

fn subcommand_delete_session() -> Command {
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

fn subcommand_delete_kernel_parameter() -> Command {
  Command::new("kernel-parameters")
    .arg_required_else_help(true)
    .about("Remove kernel parameters from nodes")
    .arg(arg!(-n --nodes <NODES>).help(HOSTLIST_HELP))
    .arg(arg!(-H --"hsm-group" <GROUP_NAME> "Node group name"))
    .arg(arg!(-y --"assume-yes" "Skip confirmation prompts").action(ArgAction::SetTrue))
    .arg(arg!(--"do-not-reboot" "Do not reboot nodes after applying changes").action(ArgAction::SetTrue))
    .arg(arg!(-d --"dry-run" "Simulate the operation without making changes").action(ArgAction::SetTrue))
    // ID preserved as "VALUE" for handler compatibility
    .arg(arg!(<VALUE> "Comma-separated kernel parameter names to remove.\neg: console,bad_page,crashkernel,hugepagelist,quiet").value_name("PARAMS"))
    .group(
      ArgGroup::new("cluster_or_nodes")
        .args(["hsm-group", "nodes"])
        .required(true),
    )
}

fn subcommand_delete_boot_parameter() -> Command {
  Command::new("boot-parameters")
    .arg_required_else_help(true)
    .about("Delete boot parameters for nodes")
    .arg(arg!(-H --hosts <XNAMES> "Xnames of the nodes"))
}

fn subcommand_delete_redfish_endpoint() -> Command {
  Command::new("redfish-endpoint")
    .arg_required_else_help(true)
    .about("Delete a Redfish endpoint")
    .arg(
      arg!(-i --id <XNAME> "Xname of the Redfish endpoint to delete")
        .required(true),
    )
}

fn subcommand_get_group() -> Command {
  Command::new("groups")
    .about("List node groups")
    // ID preserved as "VALUE" for handler compatibility
    .arg(arg!(<VALUE> "Group name (lists all groups if omitted)").value_name("GROUP_NAME").required(false))
    .arg(
      arg!(-o --output <FORMAT> "Output format")
        .value_parser(["json", "table"])
        .default_value("table"),
    )
}

fn subcommand_get_hardware() -> Command {
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

fn subcommand_get_cfs_configuration() -> Command {
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

fn subcommand_get_cfs_session() -> Command {
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

fn subcommand_get_bos_template() -> Command {
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

fn subcommand_get_cluster_details() -> Command {
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

fn subcommand_get_node_details() -> Command {
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

fn subcommand_get_images() -> Command {
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

fn subcommand_get_boot_parameters() -> Command {
  Command::new("boot-parameters")
    .arg_required_else_help(true)
    .about("Show boot parameters for nodes or a group")
    .arg(arg!(-H --"hsm-group" <GROUP_NAME> "Node group name"))
    .arg(arg!(-n --nodes <NODES>).help(HOSTLIST_HELP))
}

fn subcommand_get_kernel_parameters() -> Command {
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

fn subcommand_get_redfish_endpoints() -> Command {
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

fn subcommand_get() -> Command {
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

fn subcommand_apply_hw_configuration() -> Command {
  Command::new("hardware")
    .about("[experimental] Rescale a cluster's hardware allocation")
    .arg_required_else_help(true)
    .subcommand(
      Command::new("cluster")
        .arg_required_else_help(true)
        .about("[experimental] Rescale a cluster's hardware allocation")
        .long_about(
          "[experimental] Upscale or downscale a cluster by specifying a hardware component pattern.\n\n\
          If the cluster does not exist it will be created; otherwise its node assignment is updated.\n\n\
          Pattern format: <component>:<quantity>[:<component>:<quantity>...]\n\
          eg: 'a100:12:epyc:5'  — assign nodes with 12 A100 GPUs and 5 EPYC CPUs total",
        )
        .arg(
          arg!(-P -- pattern <PATTERN> "Hardware pattern: <component>:<qty>[:<component>:<qty>...].\neg: 'a100:12:epyc:5'")
            .required(true),
        )
        .arg(arg!(-t --"target-cluster" <TARGET_CLUSTER_NAME> "Cluster to rescale").required(true))
        .arg(
          arg!(-p --"parent-cluster" <PARENT_CLUSTER_NAME> "Cluster that donates or receives the redistributed nodes")
            .required(true),
        )
        .arg(arg!(-d --"dry-run" "Simulate the operation without making changes").action(ArgAction::SetTrue))
        .arg(arg!(-c --"create-target-hsm-group" "Create the target cluster if it does not exist"))
        .arg(arg!(-D --"delete-empty-parent-hsm-group" "Delete the parent cluster if empty after this operation"))
        .arg(arg!(-u --"unpin-nodes" "Allow any available nodes to be selected")),
    )
}

fn subcommand_apply_session() -> Command {
  Command::new("session")
    .arg_required_else_help(true)
    .about("Create and run a configuration session from a local repo")
    .long_about(
      "Create and run a configuration session from a local git repo.\n\n\
      The repo must already exist in the system's VCS. The session runs \
      the specified Ansible playbook against the target nodes or group.",
    )
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
        "Limit the session to specific nodes (must be a subset of --hsm-group if both are provided)")
        .required(true),
    )
    .arg(arg!(-H --"hsm-group" <GROUP_NAME> "Node group name"))
    .group(
      ArgGroup::new("hsm-group_or_ansible-limit")
        .args(["hsm-group", "ansible-limit"])
        .required(true),
    )
}

fn subcommand_apply_configuration() -> Command {
  Command::new("configuration")
    .arg_required_else_help(true)
    .about("Create a configuration (deprecated — use 'apply sat-file')")
    .arg(
      arg!(-t --"sat-template-file" <SAT_FILE_PATH> "SAT file path")
        .value_parser(value_parser!(PathBuf))
        .required(true),
    )
    .arg(
      arg!(-f --"values-file" <VALUES_FILE_PATH> "Values file for SAT jinja2 templates")
        .value_parser(value_parser!(PathBuf)),
    )
    .arg(arg!(-V --"values" <VALUES> ... "Inline values for SAT jinja2 templates (overrides --values-file)"))
    .arg(arg!(-o --output <FORMAT> "Output format").value_parser(["json"]))
    .arg(arg!(-H --"hsm-group" <GROUP_NAME> "Node group name"))
}

fn subcommand_apply_template() -> Command {
  Command::new("template")
    .arg_required_else_help(true)
    .about("Boot nodes using an existing session template")
    .arg(arg!(-n --name <VALUE> "Name of the boot session to create"))
    .arg(
      arg!(-o --operation <VALUE> "Boot operation to perform")
        .value_parser(["reboot", "boot", "shutdown"])
        .default_value("reboot"),
    )
    .arg(arg!(-t --template <VALUE> "Session template name").required(true))
    .arg(
      arg!(-l --limit <VALUE>
        "Limit to specific nodes, groups, or roles (OR by default; prefix with '&' for AND or '!' for NOT)")
        .required(true),
    )
    .arg(
      arg!(-i --"include-disabled" "Include nodes marked as disabled in the hardware state manager")
        .action(ArgAction::SetTrue),
    )
    .arg(arg!(-y --"assume-yes" "Skip confirmation prompts").action(ArgAction::SetTrue))
    .arg(arg!(-d --"dry-run" "Simulate the operation without making changes").action(ArgAction::SetTrue))
}

fn subcommand_apply_ephemeral_environment() -> Command {
  Command::new("ephemeral-environment")
    .arg_required_else_help(true)
    .about("Launch an ephemeral SSH environment from an image")
    .long_about(
      "Launch an ephemeral SSH environment from an image.\n\n\
      Returns an SSH hostname once the environment is ready (usually within a few seconds).",
    )
    .arg(arg!(-i --"image-id" <IMAGE_ID> "Image ID to use").required(true))
}

fn subcommand_apply_sat_file() -> Command {
  Command::new("sat-file")
    .arg_required_else_help(true)
    .about("Process a SAT file to create configurations, images, and session templates")
    .long_about(
      "Process a SAT file containing up to three sections:\n\
      \n\
      - `configurations`:   configurations to create\n\
      - `images`:           images to build from those configurations\n\
      - `session_templates`: session templates to create\n\
      \n\
      Use --image-only to process only configurations and images.\n\
      Use --sessiontemplate-only to process only configurations and session templates.",
    )
    .arg(
      arg!(-t --"sat-template-file" <FILE> "SAT file path (may be a jinja2 template)")
        .value_parser(value_parser!(PathBuf))
        .required(true)
        .value_hint(ValueHint::FilePath),
    )
    .arg(
      arg!(-f --"values-file" <FILE> "Values file to expand jinja2 variables in the SAT file")
        .value_parser(value_parser!(PathBuf))
        .value_hint(ValueHint::FilePath),
    )
    .arg(arg!(-V --"values" <VALUE> ... "Inline values to expand jinja2 variables (overrides --values-file)"))
    .arg(arg!(--"reboot" "Reboot nodes after applying session templates").action(ArgAction::SetTrue))
    .arg(
      arg!(-v --"ansible-verbosity" <VALUE> "Ansible verbosity level (1 = -v, 2 = -vv, …, max 4)")
        .value_parser(["1", "2", "3", "4"])
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
      arg!(-o --"overwrite-configuration" "Overwrite an existing configuration with the same name")
        .action(ArgAction::SetTrue),
    )
    .arg(arg!(-w --"watch-logs" "Stream session logs to stdout").action(ArgAction::SetTrue))
    .arg(arg!(-T --timestamps "Show log timestamps").action(ArgAction::SetTrue))
    .arg(
      arg!(-i --"image-only" "Process only the `configurations` and `images` sections")
        .action(ArgAction::SetTrue),
    )
    .arg(
      arg!(-s --"sessiontemplate-only" "Process only the `configurations` and `session_templates` sections")
        .action(ArgAction::SetTrue),
    )
    .arg(arg!(-p --"pre-hook" <SCRIPT> "Command to run before processing.\neg: --pre-hook \"echo hello\""))
    .arg(arg!(-a --"post-hook" <SCRIPT> "Command to run after successful processing.\neg: --post-hook \"echo hello\""))
    .arg(arg!(-y --"assume-yes" "Skip confirmation prompts").action(ArgAction::SetTrue))
    .arg(arg!(-d --"dry-run" "Simulate the operation without making changes").action(ArgAction::SetTrue))
}

fn subcommand_apply_boot_nodes() -> Command {
  Command::new("nodes")
    .arg_required_else_help(true)
    .about("Update boot parameters for a set of nodes")
    .long_about(
      "Update the boot parameters (image, runtime configuration, and kernel parameters) for a set of nodes.\n\n\
      The boot image can be specified by image ID or by the configuration name used to build it \
      (the most recent matching image is used).\n\n\
      eg:\n  \
      manta apply boot nodes \\\n    \
        --boot-image-configuration <config-name> \\\n    \
        --runtime-configuration <config-name> <nodes>",
    )
    .arg(arg!(-i --"boot-image" <IMAGE_ID> "Image ID to boot the nodes"))
    .arg(
      arg!(-b --"boot-image-configuration" <NAME>
        "Configuration name used to build the boot image (uses the most recent matching image)"),
    )
    .arg(arg!(-r --"runtime-configuration" <NAME> "Configuration to apply to nodes after booting"))
    .arg(arg!(-k --"kernel-parameters" <VALUE> "Kernel parameters to assign to the nodes"))
    .arg(arg!(-y --"assume-yes" "Skip confirmation prompts").action(ArgAction::SetTrue))
    .arg(
      arg!(--"do-not-reboot" "Suppress the automatic reboot after updating boot parameters")
        .action(ArgAction::SetTrue),
    )
    .arg(arg!(-d --"dry-run" "Simulate the operation without making changes").action(ArgAction::SetTrue))
    .group(
      ArgGroup::new("boot-image_or_boot-config")
        .args(["boot-image", "boot-image-configuration"]),
    )
    // ID preserved as "VALUE" for handler compatibility
    .arg(arg!(<VALUE>).value_name("NODES").help(HOSTLIST_HELP))
}

fn subcommand_apply_boot_cluster() -> Command {
  Command::new("cluster")
    .arg_required_else_help(true)
    .about("Update boot parameters for all nodes in a cluster")
    .long_about(
      "Update the boot parameters (image, runtime configuration, and kernel parameters) for all nodes in a cluster.\n\n\
      The boot image can be specified by image ID or by the configuration name used to build it \
      (the most recent matching image is used).\n\n\
      eg:\n  \
      manta apply boot cluster \\\n    \
        --boot-image-configuration <config-name> \\\n    \
        --runtime-configuration <config-name> <cluster-name>",
    )
    .arg(arg!(-i --"boot-image" <IMAGE_ID> "Image ID to boot the nodes"))
    .arg(
      arg!(-b --"boot-image-configuration" <NAME>
        "Configuration name used to build the boot image (uses the most recent matching image)"),
    )
    .arg(arg!(-r --"runtime-configuration" <NAME> "Configuration to apply to nodes after booting"))
    .arg(arg!(-k --"kernel-parameters" <VALUE> "Kernel parameters to assign to all cluster nodes"))
    .arg(arg!(-y --"assume-yes" "Skip confirmation prompts").action(ArgAction::SetTrue))
    .arg(
      arg!(--"do-not-reboot" "Suppress the automatic reboot after updating boot parameters")
        .action(ArgAction::SetTrue),
    )
    .arg(arg!(-d --"dry-run" "Simulate the operation without making changes").action(ArgAction::SetTrue))
    .group(
      ArgGroup::new("boot-image_or_boot-config")
        .args(["boot-image", "boot-image-configuration"]),
    )
    .arg(arg!(<CLUSTER_NAME> "Cluster name").required(true))
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
            .arg(arg!(-y --"assume-yes" "Skip confirmation prompts").action(ArgAction::SetTrue))
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
            .arg(arg!(-y --"assume-yes" "Skip confirmation prompts").action(ArgAction::SetTrue))
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
            .arg(arg!(-g --graceful "Perform a graceful shutdown").action(ArgAction::SetTrue))
            .arg(arg!(-R --reason <TEXT> "Reason for the power operation"))
            .arg(arg!(-y --"assume-yes" "Skip confirmation prompts").action(ArgAction::SetTrue))
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
            .arg(arg!(-g --graceful "Perform a graceful shutdown").action(ArgAction::SetTrue))
            .arg(arg!(-y --"assume-yes" "Skip confirmation prompts").action(ArgAction::SetTrue))
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
            .arg(arg!(-g --graceful "Perform a graceful reboot").action(ArgAction::SetTrue))
            .arg(arg!(-y --"assume-yes" "Skip confirmation prompts").action(ArgAction::SetTrue))
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
            .arg(arg!(-g --graceful "Perform a graceful reboot").action(ArgAction::SetTrue))
            .arg(arg!(-y --"assume-yes" "Skip confirmation prompts").action(ArgAction::SetTrue))
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

fn subcommand_add_group() -> Command {
  Command::new("group")
    .about("Create a node group")
    .arg_required_else_help(true)
    .arg(arg!(-l --label <NAME> "Group name").required(true))
    .arg(arg!(-d --description <VALUE> "Group description"))
    .arg(arg!(-n --nodes <NODES>).help(HOSTLIST_HELP))
}

fn subcommand_add_node() -> Command {
  Command::new("node")
    .about("Register a new node")
    .arg_required_else_help(true)
    .arg(arg!(-i --id <XNAME> "Node xname").required(true))
    .arg(arg!(-g --group <NAME> "Node group to add the node to").required(true))
    .arg(
      arg!(-H --hardware <FILE> "File containing hardware information")
        .value_parser(value_parser!(PathBuf)),
    )
    .arg(
      arg!(-a --arch <VALUE> "Node architecture")
        .value_parser(["X86", "ARM", "Other"]),
    )
    .arg(
      arg!(-d --disabled "Register the node as disabled")
        .action(ArgAction::SetFalse),
    )
}

fn subcommand_add_hwcomponent() -> Command {
  Command::new("hardware")
    .arg_required_else_help(true)
    .about("[experimental] Add hardware components to a cluster")
    .arg(arg!(-P --pattern <PATTERN> "Hardware component pattern"))
    .arg(arg!(-t --"target-cluster" <NAME> "Cluster to add components to"))
    .arg(arg!(-p --"parent-cluster" <NAME> "Cluster that donates the components"))
    .arg(arg!(-d --"dry-run" "Simulate the operation without making changes").action(ArgAction::SetTrue))
    .arg(arg!(-c --"create-hsm-group" "Create the target cluster if it does not exist"))
}

fn subcommand_add_redfish_endpoint() -> Command {
  Command::new("redfish-endpoint")
    .about("Register a new Redfish endpoint")
    .arg(arg!(-i --id <XNAME> "Xname of the BMC or controller").required(true))
    .arg(arg!(-n --name <VALUE> "Arbitrary user-provided name for the endpoint"))
    .arg(arg!(-H --hostname <VALUE> "Hostname (FQDN host portion); normally identical to the xname"))
    .arg(arg!(-d --domain <VALUE> "Domain (FQDN domain portion)"))
    .arg(
      arg!(-f --fqdn <VALUE> "Fully-qualified domain name on the management network (derived from hostname + domain)"),
    )
    .arg(arg!(-e --enabled "Enable the endpoint upon creation").action(ArgAction::SetTrue))
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

fn subcommand_add_boot_parameters() -> Command {
  Command::new("boot-parameters")
    .arg_required_else_help(true)
    .about("Create boot parameters for nodes")
    .arg(arg!(-H --"hosts" <XNAMES> "Xnames of the nodes").required(true))
    .arg(arg!(-n --"nids" <VALUE> "Comma-separated NIDs of the nodes"))
    .arg(arg!(-m --"macs" <VALUE> "Comma-separated MAC addresses of the nodes"))
    .arg(arg!(-p --"params" <VALUE> "Kernel parameters"))
    .arg(arg!(-k --"kernel" <VALUE> "S3 path to the kernel file"))
    .arg(arg!(-i --"initrd" <VALUE> "S3 path to the initrd file"))
    .arg(arg!(-c --"cloud-init" <VALUE> "Cloud-init script"))
    .arg(arg!(-d --"dry-run" "Simulate the operation without making changes").action(ArgAction::SetTrue))
    .arg(arg!(-y --"assume-yes" "Skip confirmation prompts").action(ArgAction::SetTrue))
}

fn subcommand_add_kernel_parameters() -> Command {
  Command::new("kernel-parameters")
    .arg_required_else_help(true)
    .about("Append kernel parameters to nodes")
    .arg(arg!(-n --nodes <NODES>).help(HOSTLIST_HELP))
    .arg(arg!(-H --"hsm-group" <GROUP_NAME> "Node group name"))
    .arg(
      arg!(-O --"overwrite" "Overwrite the value if the parameter already exists")
        .action(ArgAction::SetTrue),
    )
    .arg(arg!(-y --"assume-yes" "Skip confirmation prompts").action(ArgAction::SetTrue))
    .arg(arg!(--"do-not-reboot" "Do not reboot nodes after applying changes").action(ArgAction::SetTrue))
    .arg(arg!(-d --"dry-run" "Simulate the operation without making changes").action(ArgAction::SetTrue))
    // ID preserved as "VALUE" for handler compatibility
    .arg(
      arg!(<VALUE> "Space-separated kernel parameters to append.\neg: bos_update_frequency=4h console=ttyS0,115200 crashkernel=512M")
        .value_name("PARAMS"),
    )
    .group(
      ArgGroup::new("cluster_or_nodes")
        .args(["hsm-group", "nodes"])
        .required(true),
    )
}

fn subcommand_apply_kernel_parameters() -> Command {
  Command::new("kernel-parameters")
    .arg_required_else_help(true)
    .about("Replace kernel parameters on nodes")
    .arg(arg!(-n --nodes <NODES>).help(HOSTLIST_HELP))
    .arg(arg!(-H --"hsm-group" <GROUP_NAME> "Node group name"))
    .arg(arg!(-y --"assume-yes" "Skip confirmation prompts").action(ArgAction::SetTrue))
    .arg(arg!(--"do-not-reboot" "Do not reboot nodes after applying changes").action(ArgAction::SetTrue))
    .arg(arg!(-d --"dry-run" "Simulate the operation without making changes").action(ArgAction::SetTrue))
    // ID preserved as "VALUE" for handler compatibility
    .arg(
      arg!(<VALUE> "Space-separated kernel parameters to apply.\neg: bos_update_frequency=4h console=ttyS0,115200 crashkernel=512M")
        .value_name("PARAMS"),
    )
    .group(
      ArgGroup::new("cluster_or_nodes")
        .args(["hsm-group", "nodes"])
        .required(true),
    )
}

fn subcommand_update_boot_parameters() -> Command {
  Command::new("boot-parameters")
    .arg_required_else_help(true)
    .about("Update boot parameters for nodes")
    .arg(arg!(-H --"hosts" <XNAMES> "Xnames of the nodes to update").required(true))
    .arg(arg!(-p --"params" <VALUE> "Kernel parameters"))
    .arg(arg!(-k --"kernel" <VALUE> "S3 path to the kernel file"))
    .arg(arg!(-i --"initrd" <VALUE> "S3 path to the initrd file"))
    .arg(arg!(-d --"dry-run" "Simulate the operation without making changes").action(ArgAction::SetTrue))
    .arg(arg!(-y --"assume-yes" "Skip confirmation prompts").action(ArgAction::SetTrue))
}

fn subcommand_update_redfish_endpoint() -> Command {
  Command::new("redfish-endpoint")
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

fn subcommand_add() -> Command {
  Command::new("add")
    .arg_required_else_help(true)
    .about("Create system resources")
    .subcommand(subcommand_add_node())
    .subcommand(subcommand_add_group())
    .subcommand(subcommand_add_hwcomponent())
    .subcommand(subcommand_add_boot_parameters())
    .subcommand(subcommand_add_kernel_parameters())
    .subcommand(subcommand_add_redfish_endpoint())
}

fn subcommand_apply() -> Command {
  Command::new("apply")
    .arg_required_else_help(true)
    .about("Apply changes to the system")
    .subcommand(subcommand_apply_hw_configuration())
    .subcommand(subcommand_apply_configuration())
    .subcommand(subcommand_apply_sat_file())
    .subcommand(
      Command::new("boot")
        .arg_required_else_help(true)
        .about("Update boot parameters and runtime configuration")
        .subcommand(subcommand_apply_boot_nodes())
        .subcommand(subcommand_apply_boot_cluster()),
    )
    .subcommand(subcommand_apply_kernel_parameters())
    .subcommand(subcommand_apply_session())
    .subcommand(subcommand_apply_ephemeral_environment())
    .subcommand(subcommand_apply_template())
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
        .about("Connect to the Ansible target container of a configuration session")
        .arg(arg!(<SESSION_NAME> "Configuration session name").required(true)),
    )
}

fn subcommand_add_nodes_to_groups() -> Command {
  Command::new("add-nodes-to-groups")
    .about("Add nodes to one or more groups")
    .arg(arg!(-g --group <NAME> "Group to add the nodes to"))
    .arg(arg!(-n --nodes <NODES>).help(HOSTLIST_HELP))
    .arg(arg!(-d --"dry-run" "Simulate the operation without making changes").action(ArgAction::SetTrue))
}

fn subcommand_remove_nodes_from_groups() -> Command {
  Command::new("remove-nodes-from-groups")
    .about("Remove nodes from one or more groups")
    .arg(arg!(-g --group <NAME> "Group to remove the nodes from"))
    .arg(arg!(-n --nodes <NODES>).help(HOSTLIST_HELP))
    .arg(arg!(-d --"dry-run" "Simulate the operation without making changes").action(ArgAction::SetTrue))
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
