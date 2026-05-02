use anyhow::{Context, Error, bail};

use manta_backend_dispatcher::interfaces::hsm::group::GroupTrait;

use crate::{
  common::{self, audit, kafka::Kafka},
  manta_backend_dispatcher::StaticBackendDispatcher,
};

/// Remove/unassign a list of xnames to a list of HSM groups
pub async fn exec(
  backend: &StaticBackendDispatcher,
  token: &str,
  target_hsm_name: &str,
  hosts_expression: &str,
  dryrun: bool,
  kafka_audit_opt: Option<&Kafka>,
) -> Result<(), Error> {
  // Convert user input to xname
  let xname_to_move_vec = common::node_ops::resolve_hosts_expression(
    backend,
    token,
    hosts_expression,
    false,
  )
  .await?;

  // Check if there are any xname to migrate/move and exit otherwise
  if xname_to_move_vec.is_empty() {
    bail!(
      "The list of nodes to move is empty. \
       Nothing to do",
    );
  }

  if common::user_interaction::confirm(
    &format!(
      "{}\nThe nodes above will be removed from HSM group '{}'. Do you want to proceed?",
      xname_to_move_vec.join(", "), target_hsm_name
    ),
    false,
  ) {
    tracing::info!("Continue",);
  } else {
    bail!("Operation cancelled by user");
  }

  if backend
    .get_group(token, target_hsm_name)
    .await
    .is_ok()
  {
    tracing::debug!("The HSM group {} exists, good.", target_hsm_name);
  }

  if dryrun {
    println!(
      "dryrun - Delete nodes {:?} in {}",
      xname_to_move_vec, target_hsm_name
    );

    return Ok(());
  }

  // Remove xnames from HSM group
  for xname in &xname_to_move_vec {
    backend
      .delete_member_from_group(token, target_hsm_name, xname)
      .await
      .with_context(|| {
        format!(
          "Failed to remove node '{}' from group '{}'",
          xname, target_hsm_name
        )
      })?;
  }

  // Audit
  audit::maybe_send_audit(
    kafka_audit_opt,
    token,
    format!("Remove nodes from group '{}'", target_hsm_name),
    Some(serde_json::json!(xname_to_move_vec)),
    Some(serde_json::json!(vec![target_hsm_name])),
  )
  .await;

  Ok(())
}
