//! Clap definitions for `manta add *` subcommands.

use clap::{ArgAction, ArgGroup, Command, arg, value_parser};
use std::path::PathBuf;

use super::{HOSTLIST_HELP, output_flag};

pub fn subcommand_add_group() -> Command {
  Command::new("group")
    .about("Create a node group")
    .arg_required_else_help(true)
    .arg(arg!(-l --label <NAME> "Group name").required(true))
    .arg(arg!(-d --description <VALUE> "Group description"))
    .arg(arg!(-n --nodes <NODES>).help(HOSTLIST_HELP))
    .arg(
      arg!(--"dry-run" "Validate input and print the payload(s) that would be sent to the backend without making changes")
        .action(ArgAction::SetTrue),
    )
    .arg(output_flag())
}

pub fn subcommand_add_node() -> Command {
  Command::new("node")
    .about("Register a new node in the hardware state manager")
    .long_about(
      "Register a new node in the hardware state manager.\n\n\
      Use this for nodes that don't exist yet. To attach existing nodes to a \
      group, use `manta add nodes` (plural).",
    )
    .arg_required_else_help(true)
    .arg(arg!(-i --id <XNAME> "Xname to register").required(true))
    .arg(
      arg!(-g --group <NAME> "Group to put the new node into").required(true),
    )
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
        .action(ArgAction::SetTrue),
    )
    .arg(output_flag())
}

pub fn subcommand_add_hwcomponent() -> Command {
  Command::new("hardware")
    .arg_required_else_help(true)
    .about("[experimental] Add hardware components to a group")
    .arg(arg!(-P --pattern <PATTERN> "Hardware component pattern"))
    .arg(
      arg!(-t --"target-group" <NAME> "Group to add components to")
        .visible_alias("target-cluster"),
    )
    .arg(
      arg!(-p --"parent-group" <NAME> "Group that donates the components")
        .visible_alias("parent-cluster"),
    )
    .arg(
      arg!(-d --"dry-run" "Simulate the operation without making changes")
        .action(ArgAction::SetTrue),
    )
    .arg(
      arg!(-c --"create-group" "Create the target group if it does not exist")
        .visible_alias("create-hsm-group"),
    )
    .arg(output_flag())
}

pub fn subcommand_add_redfish_endpoint() -> Command {
  Command::new("redfish-endpoints")
    .visible_alias("redfish-endpoint")
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
    .arg(output_flag())
    .arg_required_else_help(true)
}

pub fn subcommand_add_boot_parameters() -> Command {
  Command::new("boot-parameters")
    .arg_required_else_help(true)
    .about("Create a BSS boot-parameters entry (kernel, initrd, params, cloud-init) for one or more nodes")
    .arg(arg!(-H --"hosts" <XNAMES> "Xnames of the nodes").required(true))
    .arg(arg!(-n --"nids" <VALUE> "Comma-separated NIDs of the nodes"))
    .arg(arg!(-m --"macs" <VALUE> "Comma-separated MAC addresses of the nodes"))
    .arg(arg!(-p --"params" <VALUE> "Kernel parameters"))
    .arg(arg!(-k --"kernel" <VALUE> "S3 path to the kernel file"))
    .arg(arg!(-i --"initrd" <VALUE> "S3 path to the initrd file"))
    .arg(arg!(-c --"cloud-init" <VALUE> "Cloud-init script"))
    .arg(
      arg!(-d --"dry-run" "Simulate the operation without making changes")
        .action(ArgAction::SetTrue),
    )
    .arg(
      arg!(-y --"assume-yes" "Skip confirmation prompts")
        .action(ArgAction::SetTrue),
    )
    .arg(output_flag())
}

pub fn subcommand_add_kernel_parameters() -> Command {
  Command::new("kernel-parameters")
    .arg_required_else_help(true)
    .about("Append kernel parameters to nodes (leaves existing parameters untouched unless --overwrite is set)")
    .arg(arg!(-n --nodes <NODES>).help(HOSTLIST_HELP))
    .arg(
      arg!(-H --group <GROUP_NAME> "Append kernel parameters to every node in this group")
        .visible_alias("hsm-group"),
    )
    .arg(output_flag())
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
        .args(["group", "nodes"])
        .required(true),
    )
}

pub fn subcommand_add() -> Command {
  Command::new("add")
    .arg_required_else_help(true)
    .about("Register new nodes, groups, boot/kernel parameters, hardware components, or Redfish endpoints")
    .subcommand(subcommand_add_node())
    .subcommand(subcommand_add_nodes())
    .subcommand(subcommand_add_group())
    .subcommand(subcommand_add_hwcomponent())
    .subcommand(subcommand_add_boot_parameters())
    .subcommand(subcommand_add_kernel_parameters())
    .subcommand(subcommand_add_redfish_endpoint())
}

/// `manta add nodes` — assign existing nodes to a group. Distinct from
/// `add node` (singular), which registers a brand new node in the
/// inventory.
pub fn subcommand_add_nodes() -> Command {
  Command::new("nodes")
    .about("Add existing nodes to a group")
    .long_about(
      "Add existing nodes to a group's membership.\n\n\
      Differs from `manta add node` (singular), which registers a brand-new \
      node in the inventory; this command operates on nodes that already \
      exist and just changes which group(s) they belong to.",
    )
    .arg_required_else_help(true)
    .arg(arg!(-g --group <NAME> "Group to add the nodes to").required(true))
    .arg(arg!(-n --nodes <NODES>).help(HOSTLIST_HELP).required(true))
    .arg(
      arg!(-d --"dry-run" "Simulate the operation without making changes")
        .action(ArgAction::SetTrue),
    )
    .arg(output_flag())
}
