//! Clap definitions for `manta update *` subcommands.

use clap::{ArgAction, Command, arg};

use super::output_flag;

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
    .arg(output_flag())
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
    .arg(output_flag())
    .arg_required_else_help(true)
}

pub fn subcommand_update() -> Command {
  Command::new("update")
    .arg_required_else_help(true)
    .about("Modify existing boot parameters or Redfish endpoints in place")
    .subcommand(subcommand_update_boot_parameters())
    .subcommand(subcommand_update_redfish_endpoint())
}
