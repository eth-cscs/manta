use anyhow::{Error, bail};

use crate::{
  common::{self, audit, kafka::Kafka},
  service,
};
use crate::common::app_context::AppContext;

/// Add/assign a list of xnames to an HSM group.
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  target_hsm_name: &str,
  hosts_expression: &str,
  dryrun: bool,
  kafka_audit_opt: Option<&Kafka>,
) -> Result<(), Error> {
  // Resolve hosts to validate input before confirmation
  let xname_to_move_vec = common::node_ops::resolve_hosts_expression(
    ctx.infra.backend,
    token,
    hosts_expression,
    false,
  )
  .await?;

  if xname_to_move_vec.is_empty() {
    bail!(
      "The list of nodes to move is empty. \
       Nothing to do",
    );
  }

  if common::user_interaction::confirm(
    &format!(
      "{}\nThe nodes above will be added to HSM group '{}'. Do you want to proceed?",
      xname_to_move_vec.join(", "), target_hsm_name
    ),
    false,
  ) {
    tracing::info!("Continue",);
  } else {
    bail!("Operation cancelled by user");
  }

  if dryrun {
    println!(
      "dryrun - Add nodes {:?} to {}",
      xname_to_move_vec, target_hsm_name
    );

    return Ok(());
  }

  let (_xnames_added, updated_members) = service::group::add_nodes_to_group(
    &ctx.infra,
    token,
    target_hsm_name,
    hosts_expression,
  )
  .await?;

  println!(
    "HSM '{}' members: {:?}",
    target_hsm_name, updated_members
  );

  // Audit
  audit::maybe_send_audit(
    kafka_audit_opt,
    token,
    format!("add nodes to group: {}", target_hsm_name),
    Some(serde_json::json!(xname_to_move_vec)),
    Some(serde_json::json!(vec![target_hsm_name])),
  )
  .await;

  Ok(())
}
