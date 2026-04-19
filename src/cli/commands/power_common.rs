use std::fmt;

use anyhow::{Context, Error, bail};
use manta_backend_dispatcher::interfaces::{
  hsm::group::GroupTrait, pcs::PCSTrait,
};
use nodeset::NodeSet;

use crate::common::{
  self, app_context::AppContext, audit,
  authorization::get_groups_names_available,
};

/// The three power operations supported by the backend.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerAction {
  On,
  Off,
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
  output: &str,
  token: &str,
) -> Result<(), Error> {
  let backend = ctx.infra.backend;

  // Convert user input to xnames
  let xname_vec = common::node_ops::resolve_hosts_expression(
    backend,
    token,
    hosts_expression,
    false,
  )
  .await?;

  if xname_vec.is_empty() {
    bail!(
      "The list of nodes to operate is empty. \
       Nothing to do.",
    );
  }

  let node_group: NodeSet = xname_vec
    .join(", ")
    .parse()
    .context("Failed to parse node list")?;

  println!(
    "Number of nodes: {}\nlist of nodes: {}",
    node_group.len(),
    node_group
  );

  if !common::user_interaction::confirm(action.confirmation_text(), assume_yes)
  {
    bail!("Operation cancelled by user");
  }

  let power_mgmt_summary =
    call_backend(backend, token, &xname_vec, action, force)
      .await
      .with_context(|| {
        format!("Could not {} node/s '{}'", action.error_verb(), xname_vec.join(", "),)
      })?;

  crate::cli::output::power::print_summary_table(power_mgmt_summary, output);

  // Audit
  audit::maybe_send_audit_with_group_lookup(
    ctx.cli.kafka_audit_opt,
    backend,
    token,
    action.to_string(),
    &xname_vec,
  )
  .await?;

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
  output: &str,
  token: &str,
) -> Result<(), Error> {
  let backend = ctx.infra.backend;

  let target_hsm_group_vec = get_groups_names_available(
    backend,
    token,
    Some(hsm_group_name_arg),
    ctx.cli.settings_hsm_group_name_opt,
  )
  .await?;

  let target_hsm_group = target_hsm_group_vec
    .first()
    .context("The 'cluster name' argument must have a value")?;

  let xname_vec = backend
    .get_member_vec_from_group_name_vec(
      token,
      &[target_hsm_group.to_string()],
    )
    .await
    .context("Failed to get members from HSM group")?;

  let node_group: NodeSet = xname_vec
    .join(", ")
    .parse()
    .context("Failed to parse node list")?;

  println!(
    "Number of nodes: {}\nlist of nodes: {}",
    node_group.len(),
    node_group
  );

  if !common::user_interaction::confirm(action.confirmation_text(), assume_yes)
  {
    bail!("Operation cancelled by user");
  }

  let power_mgmt_summary =
    call_backend(backend, token, &xname_vec, action, force)
      .await
      .with_context(|| {
        format!("Could not {} node/s '{}'", action.error_verb(), xname_vec.join(", "),)
      })?;

  crate::cli::output::power::print_summary_table(power_mgmt_summary, output);

  // Audit
  audit::maybe_send_audit(
    ctx.cli.kafka_audit_opt,
    token,
    action.to_string(),
    None,
    Some(serde_json::json!(hsm_group_name_arg)),
  )
  .await;

  Ok(())
}

/// Dispatches the correct backend power method based on the
/// action.
async fn call_backend(
  backend: &crate::manta_backend_dispatcher::StaticBackendDispatcher,
  shasta_token: &str,
  xname_vec: &[String],
  action: PowerAction,
  force: bool,
) -> Result<
  manta_backend_dispatcher::types::pcs::transitions::types::TransitionResponse,
  anyhow::Error,
> {
  Ok(match action {
    PowerAction::On => backend.power_on_sync(shasta_token, xname_vec).await,
    PowerAction::Off => {
      backend.power_off_sync(shasta_token, xname_vec, force).await
    }
    PowerAction::Reset => {
      backend
        .power_reset_sync(shasta_token, xname_vec, force)
        .await
    }
  }?)
}
