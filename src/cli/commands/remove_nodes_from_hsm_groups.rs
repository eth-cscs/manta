use anyhow::{Context, Error, bail};

use manta_backend_dispatcher::interfaces::hsm::{
  component::ComponentTrait, group::GroupTrait,
};

use crate::{
  common::{self, audit, authentication::get_api_token, jwt_ops, kafka::Kafka},
  manta_backend_dispatcher::StaticBackendDispatcher,
};

/// Remove/unassign a list of xnames to a list of HSM groups
pub async fn exec(
  backend: &StaticBackendDispatcher,
  site_name: &str,
  target_hsm_name: &String,
  hosts_expression: &str,
  dryrun: bool,
  kafka_audit_opt: Option<&Kafka>,
) -> Result<(), Error> {
  let shasta_token = get_api_token(backend, site_name).await?;

  // Convert user input to xname
  let node_metadata_available_vec = backend
    .get_node_metadata_available(&shasta_token)
    .await
    .map_err(|e| {
      Error::msg(format!("Could not get node metadata. Reason:\n{e}"))
    })?;

  let mut xname_to_move_vec =
    common::node_ops::from_hosts_expression_to_xname_vec(
      hosts_expression,
      false,
      node_metadata_available_vec,
    )
    .await
    .map_err(|e| {
      Error::msg(format!(
        "Could not convert user input to list of xnames. Reason:\n{}",
        e
      ))
    })?;

  xname_to_move_vec.sort();
  xname_to_move_vec.dedup();

  // Check if there are any xname to migrate/move and exit otherwise
  if xname_to_move_vec.is_empty() {
    bail!(
      "The list of nodes to move is empty. \
       Nothing to do",
    );
  }

  if common::user_interaction::confirm(
    &format!(
      "{:?}\nThe nodes above will be removed from HSM group '{}'. Do you want to proceed?",
      xname_to_move_vec, target_hsm_name
    ),
    false,
  ) {
    log::info!("Continue",);
  } else {
    bail!("Cancelled by user. Aborting.");
  }

  if backend
    .get_group(&shasta_token, target_hsm_name)
    .await
    .is_ok()
  {
    log::debug!("The HSM group {} exists, good.", target_hsm_name);
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
      .delete_member_from_group(&shasta_token, target_hsm_name, xname)
      .await
      .with_context(|| {
        format!(
          "Failed to remove node '{}' from group '{}'",
          xname, target_hsm_name
        )
      })?;
  }

  // Audit
  if let Some(kafka_audit) = kafka_audit_opt {
    let username = jwt_ops::get_name(&shasta_token).unwrap_or_default();
    let user_id =
      jwt_ops::get_preferred_username(&shasta_token).unwrap_or_default();

    let msg_json = serde_json::json!({
      "user": {"id": user_id, "name": username},
      "host": {"hostname": xname_to_move_vec},
      "group": vec![target_hsm_name],
      "message": format!(
        "Remove nodes from group '{}'",
        target_hsm_name
      ),
    });

    audit::send_audit_message(kafka_audit, msg_json).await;
  }

  Ok(())
}
