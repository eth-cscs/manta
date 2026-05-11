//! Shared argument types for power management commands.

use std::fmt;

use anyhow::{Context, Error, bail};

use crate::{
  cli::http_client::MantaClient,
  common::{self, app_context::AppContext},
};

/// The three power operations supported by the backend.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerAction {
  /// Power nodes on.
  On,
  /// Power nodes off.
  Off,
  /// Power-cycle (reset) nodes.
  Reset,
}

impl fmt::Display for PowerAction {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      PowerAction::On => write!(f, "power on"),
      PowerAction::Off => write!(f, "power off"),
      PowerAction::Reset => write!(f, "power reset"),
    }
  }
}

impl PowerAction {
  /// Human-readable confirmation prompt fragment.
  fn confirmation_text(&self) -> &'static str {
    match self {
      PowerAction::On => {
        "The nodes above will be powered on. \
         Please confirm to proceed?"
      }
      PowerAction::Off => {
        "The nodes above will be powered off. \
         Please confirm to proceed?"
      }
      PowerAction::Reset => {
        "The nodes above will restart. \
         Please confirm to proceed?"
      }
    }
  }

  /// Error verb for failure messages.
  fn error_verb(&self) -> &'static str {
    match self {
      PowerAction::On => "power on",
      PowerAction::Off => "power off",
      PowerAction::Reset => "restart",
    }
  }
}

/// Execute a power action against a list of nodes resolved
/// from a hosts expression.
pub async fn exec_nodes(
  ctx: &AppContext<'_>,
  action: PowerAction,
  hosts_expression: &str,
  force: bool,
  assume_yes: bool,
  _output: &str,
  token: &str,
) -> Result<(), Error> {
  let action_str = match action {
    PowerAction::On => "on",
    PowerAction::Off => "off",
    PowerAction::Reset => "reset",
  };

  let server_url = ctx.cli.manta_server_url
    .context("manta server URL must be configured")?;
  println!("Nodes expression: {}", hosts_expression);
  if !common::user_interaction::confirm(action.confirmation_text(), assume_yes) {
    bail!("Operation cancelled by user");
  }
  let result = MantaClient::new(server_url, ctx.infra.site_name)?
    .power(token, action_str, hosts_expression, "nodes", force)
    .await?;
  println!("{}", serde_json::to_string_pretty(&result).unwrap_or_default());
  Ok(())
}

/// Execute a power action against all nodes in an HSM group
/// (cluster).
pub async fn exec_cluster(
  ctx: &AppContext<'_>,
  action: PowerAction,
  hsm_group_name_arg: &str,
  force: bool,
  assume_yes: bool,
  _output: &str,
  token: &str,
) -> Result<(), Error> {
  let action_str = match action {
    PowerAction::On => "on",
    PowerAction::Off => "off",
    PowerAction::Reset => "reset",
  };

  let server_url = ctx.cli.manta_server_url
    .context("manta server URL must be configured")?;
  println!("Cluster: {}", hsm_group_name_arg);
  if !common::user_interaction::confirm(action.confirmation_text(), assume_yes) {
    bail!("Operation cancelled by user");
  }
  let result = MantaClient::new(server_url, ctx.infra.site_name)?
    .power(token, action_str, hsm_group_name_arg, "cluster", force)
    .await?;
  println!("{}", serde_json::to_string_pretty(&result).unwrap_or_default());
  Ok(())
}
