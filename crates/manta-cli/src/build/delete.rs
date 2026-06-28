//! Clap definitions for `manta delete *` subcommands.
//!
//! Builds the `manta delete` subtree — removing groups, nodes,
//! images, configurations, sessions, boot/kernel parameters, hardware
//! components, and Redfish endpoints. Execution dispatched in
//! `crate::dispatch::delete`.
//!
//! Notes:
//! - `delete node` (singular) removes the node from the hardware
//!   state manager entirely; `delete nodes` (plural) only strips a
//!   group's membership.
//! - `delete configurations` uses an `ArgGroup("since_and_until")`
//!   that requires both `--since` and `--until`, and conflicts with
//!   `--configuration-name`.
//! - `delete kernel-parameters` uses `ArgGroup("cluster_or_nodes")`
//!   to require exactly one of `--nodes` / `--group`.

use clap::{ArgAction, ArgGroup, Command, arg};

use super::{HOSTLIST_HELP, dry_run_flag, output_flag};

/// Top-level `manta delete` verb — wires every `delete <noun>`
/// subcommand together.
pub fn subcommand_delete() -> Command {
  Command::new("delete")
    .arg_required_else_help(true)
    .about("Remove nodes, groups, images, configurations, sessions, boot/kernel parameters, or Redfish endpoints")
    .subcommand(subcommand_delete_group())
    .subcommand(subcommand_delete_node())
    .subcommand(subcommand_delete_nodes())
    .subcommand(subcommand_delete_kernel_parameter())
    .subcommand(subcommand_delete_boot_parameter())
    .subcommand(subcommand_delete_configuration())
    .subcommand(subcommand_delete_session())
    .subcommand(subcommand_delete_image())
    .subcommand(subcommand_delete_hw_component())
    .subcommand(subcommand_delete_redfish_endpoint())
}

/// `manta delete nodes` — remove nodes from a group's membership.
/// Distinct from `delete node` (singular), which removes the node
/// from the system inventory entirely.
pub fn subcommand_delete_nodes() -> Command {
  Command::new("nodes")
    .about("Remove nodes from a group")
    .long_about(
      "Remove nodes from a group's membership.\n\n\
      Differs from `manta delete node` (singular), which removes a node \
      from the system inventory entirely; this command leaves the nodes \
      in place and only changes which group(s) they belong to.",
    )
    .arg_required_else_help(true)
    .arg(
      arg!(-g --group <NAME> "Group to remove the nodes from").required(true),
    )
    .arg(arg!(-n --nodes <NODES>).help(HOSTLIST_HELP).required(true))
    .arg(dry_run_flag())
    .arg(arg!(-y --"assume-yes" "Skip confirmation prompts").action(ArgAction::SetTrue))
    .arg(output_flag())
}

/// `manta delete group` — delete an empty node group. Handler:
/// `crate::dispatch::delete::group`.
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
    .arg(
      arg!(-d --"dry-run" "Validate input and print the request that would be sent to the backend without making changes")
        .action(ArgAction::SetTrue),
    )
    // ID preserved as "VALUE" for handler compatibility
    .arg(
      arg!(<VALUE> "Name of the group to delete")
        .value_name("GROUP_NAME")
        .required(true),
    )
    .arg(output_flag())
}

/// `manta delete node` — remove a node from the hardware state
/// manager entirely. Handler: `crate::dispatch::delete::node`.
pub fn subcommand_delete_node() -> Command {
  Command::new("node")
    .arg_required_else_help(true)
    .about("Remove a node from the hardware state manager")
    .long_about(
      "Remove a node from the hardware state manager (the node disappears from \
      the inventory). To only remove it from a group's membership without \
      deleting it, use `manta delete nodes` (plural).",
    )
    // ID preserved as "VALUE" for handler compatibility
    .arg(
      arg!(<VALUE> "Xname of the node to remove")
        .value_name("XNAME")
        .required(true),
    )
    .arg(dry_run_flag())
    .arg(output_flag())
}

/// `manta delete hardware` — remove hardware components from a group
/// (donating them back to a parent). Handler:
/// `crate::dispatch::delete::hw_component`.
pub fn subcommand_delete_hw_component() -> Command {
  Command::new("hardware")
    .arg_required_else_help(true)
    .about("[experimental] Remove hardware components from a group")
    .arg(arg!(-P --pattern <PATTERN> "Hardware component pattern"))
    .arg(
      arg!(-t --"target-group" <TARGET_GROUP_NAME> "Group to remove components from")
        .visible_alias("target-cluster"),
    )
    .arg(
      arg!(-p --"parent-group" <PARENT_GROUP_NAME> "Group that receives the freed components")
        .visible_alias("parent-cluster"),
    )
    .arg(dry_run_flag())
    .arg(
      arg!(-D --"delete-group" "Delete the group if empty after this operation")
        .action(ArgAction::SetTrue)
        .visible_alias("delete-hsm-group"),
    )
    .arg(output_flag())
}

/// `manta delete images` — delete one or more IMS images by ID
/// (refuses if an image is currently a node's boot image). Handler:
/// `crate::dispatch::delete::image`.
pub fn subcommand_delete_image() -> Command {
  Command::new("images")
    .arg_required_else_help(true)
    .about("[experimental] Delete IMS images by ID (refuses to delete images currently booting a node)")
    .arg(dry_run_flag())
    // ID preserved as "IMAGE_LIST" for handler compatibility
    .arg(
      arg!(<IMAGE_LIST> "Comma-separated image IDs to delete.\neg: e2ce82f0-e7ba-4f36-9f5c-750346599600,59e0180a-3fdd-4936-bba7-14ba914ffd34")
        .value_name("IMAGE_IDS")
        .required(true),
    )
    .arg(output_flag())
}

/// `manta delete configurations` — cascade-delete CFS configurations
/// (and all sessions, templates, and images derived from them).
/// Filter by exact name, glob, or update-date range; the
/// `since`/`until` `ArgGroup` requires both bounds when used.
/// Handler: `crate::dispatch::delete::configuration`.
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
    .arg(dry_run_flag())
    .arg(output_flag())
    .group(
      ArgGroup::new("since_and_until")
        .args(["since", "until"])
        .multiple(true)
        .requires("until")
        .conflicts_with("configuration-name"),
    )
}

/// `manta delete session` — delete a CFS configuration session
/// (image-building sessions also drop their image). Handler:
/// `crate::dispatch::delete::session`.
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
    .arg(dry_run_flag())
    .arg(arg!(<SESSION_NAME> "Name of the session to delete").required(true))
    .arg(output_flag())
}

/// `manta delete kernel-parameters` — remove named kernel parameters
/// (by parameter name, ignoring values) from a node set or group.
/// Handler: `crate::dispatch::delete::kernel_parameters`.
pub fn subcommand_delete_kernel_parameter() -> Command {
  Command::new("kernel-parameters")
    .arg_required_else_help(true)
    .about("Remove kernel parameters from nodes (parameter values are ignored — match is by name)")
    .arg(arg!(-n --nodes <NODES>).help(HOSTLIST_HELP))
    .arg(
      arg!(-H --group <GROUP_NAME> "Remove the listed kernel parameters from every node in this group")
        .visible_alias("hsm-group"),
    )
    .arg(dry_run_flag())
    // ID preserved as "VALUE" for handler compatibility
    .arg(arg!(<VALUE> "Comma-separated kernel parameter names to remove.\neg: console,bad_page,crashkernel,hugepagelist,quiet").value_name("PARAMS"))
    .group(
      ArgGroup::new("cluster_or_nodes")
        .args(["group", "nodes"])
        .required(true),
    )
    .arg(output_flag())
}

/// `manta delete boot-parameters` — delete a BSS boot-parameters
/// entry for the listed hosts. Handler:
/// `crate::dispatch::delete::boot_parameters`.
pub fn subcommand_delete_boot_parameter() -> Command {
  Command::new("boot-parameters")
    .arg_required_else_help(true)
    .about("Delete boot parameters for nodes")
    .arg(arg!(-H --hosts <XNAMES> "Xnames of the nodes"))
    .arg(dry_run_flag())
    .arg(output_flag())
}

/// `manta delete redfish-endpoints` — unregister a Redfish endpoint
/// by xname. Handler:
/// `crate::dispatch::delete::redfish_endpoint`.
pub fn subcommand_delete_redfish_endpoint() -> Command {
  Command::new("redfish-endpoints")
    .visible_alias("redfish-endpoint")
    .arg_required_else_help(true)
    .about("Delete a Redfish endpoint")
    .arg(
      arg!(-i --id <XNAME> "Xname of the Redfish endpoint to delete")
        .required(true),
    )
    .arg(dry_run_flag())
    .arg(output_flag())
}
