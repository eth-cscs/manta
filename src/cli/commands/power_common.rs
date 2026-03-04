use std::fmt;

use anyhow::{Context, Error, bail};
use manta_backend_dispatcher::interfaces::{
  hsm::{component::ComponentTrait, group::GroupTrait},
  pcs::PCSTrait,
};
use nodeset::NodeSet;

use crate::common::{
  self, app_context::AppContext, audit, authentication::get_api_token,
  authorization::get_groups_names_available, jwt_ops,
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
) -> Result<(), Error> {
  let backend = ctx.backend;
  let shasta_token = get_api_token(backend, ctx.site_name).await?;

  // Convert user input to xnames
  let node_metadata_available_vec = backend
    .get_node_metadata_available(&shasta_token)
    .await
    .map_err(|e| {
      Error::msg(format!("Could not get node metadata. Reason:\n{e}"))
    })?;

  let mut xname_vec = common::node_ops::from_hosts_expression_to_xname_vec(
    hosts_expression,
    false,
    node_metadata_available_vec,
  )
  .await
  .map_err(|e| {
    Error::msg(format!(
      "Could not convert user input to list of \
         xnames. Reason:\n{e}"
    ))
  })?;

  if xname_vec.is_empty() {
    bail!(
      "The list of nodes to operate is empty. \
       Nothing to do.",
    );
  }

  xname_vec.sort();
  xname_vec.dedup();

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
    bail!("Cancelled by user. Aborting.");
  }

  let power_mgmt_summary =
    call_backend(backend, &shasta_token, &xname_vec, action, force)
      .await
      .with_context(|| {
        format!("Could not {} node/s '{:?}'", action.error_verb(), xname_vec,)
      })?;

  common::pcs_utils::print_summary_table(power_mgmt_summary, output);

  // Audit
  if let Some(kafka_audit) = ctx.kafka_audit_opt {
    let username = jwt_ops::get_name(&shasta_token)
      .context("Failed to get username from token")?;
    let user_id = jwt_ops::get_preferred_username(&shasta_token)
      .context("Failed to get preferred username from token")?;

    let group_map = backend
      .get_group_map_and_filter_by_member_vec(
        &shasta_token,
        &xname_vec.iter().map(String::as_str).collect::<Vec<_>>(),
      )
      .await
      .context("Failed to get group map for audit")?;

    let msg_json = serde_json::json!({
      "user": {"id": user_id, "name": username},
      "host": {"hostname": xname_vec},
      "group": group_map
        .keys()
        .collect::<Vec<_>>(),
      "message": action.to_string(),
    });

    audit::send_audit_message(kafka_audit, msg_json).await;
  }

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
) -> Result<(), Error> {
  let backend = ctx.backend;
  let shasta_token = get_api_token(backend, ctx.site_name).await?;

  let target_hsm_group_vec = get_groups_names_available(
    backend,
    &shasta_token,
    Some(&hsm_group_name_arg.to_string()),
    ctx.settings_hsm_group_name_opt,
  )
  .await?;

  let target_hsm_group = target_hsm_group_vec
    .first()
    .context("The 'cluster name' argument must have a value")?;

  let xname_vec = backend
    .get_member_vec_from_group_name_vec(
      &shasta_token,
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
    bail!("Cancelled by user. Aborting.");
  }

  let power_mgmt_summary =
    call_backend(backend, &shasta_token, &xname_vec, action, force)
      .await
      .with_context(|| {
        format!("Could not {} node/s '{:?}'", action.error_verb(), xname_vec,)
      })?;

  common::pcs_utils::print_summary_table(power_mgmt_summary, output);

  // Audit
  if let Some(kafka_audit) = ctx.kafka_audit_opt {
    let username = jwt_ops::get_name(&shasta_token)
      .context("Failed to get username from token")?;
    let user_id = jwt_ops::get_preferred_username(&shasta_token)
      .context("Failed to get preferred username from token")?;

    let msg_json = serde_json::json!({
      "user": {"id": user_id, "name": username},
      "group": hsm_group_name_arg,
      "message": action.to_string(),
    });

    audit::send_audit_message(kafka_audit, msg_json).await;
  }

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
