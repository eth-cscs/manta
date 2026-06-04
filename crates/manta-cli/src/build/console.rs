//! Clap definitions for `manta console *` subcommands.

use clap::{Command, arg};

pub fn subcommand_console() -> Command {
  Command::new("console")
    .arg_required_else_help(true)
    .about("Attach to a node's serial console, or to a configuration session's Ansible container")
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
