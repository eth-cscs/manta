use anyhow::Error;

use manta_backend_dispatcher::interfaces::hsm::{
  component::ComponentTrait, group::GroupTrait,
};

use crate::{
  common::{self, audit::Audit, jwt_ops, kafka::Kafka, authentication::get_api_token},
  manta_backend_dispatcher::StaticBackendDispatcher,
};

/// Add/assign a list of xnames to a list of HSM groups
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
  let node_metadata_available_vec =
    backend.get_node_metadata_available(&shasta_token).await?;

  let mut xname_to_move_vec =
    common::node_ops::from_hosts_expression_to_xname_vec(
      hosts_expression,
      false,
      node_metadata_available_vec,
    )
    .await
    .map_err(|e| {
      Error::msg(format!(
        "ERROR - Could not convert user input to list of xnames. Reason:\n{}",
        e
      ))
    })?;

  xname_to_move_vec.sort();
  xname_to_move_vec.dedup();

  // Check if there are any xname to migrate/move and exit otherwise
  if xname_to_move_vec.is_empty() {
    return Err(Error::msg(
      "The list of nodes to move is empty. Nothing to do. Exit",
    ));
  }

  if common::user_interaction::confirm(
    &format!(
      "{:?}\nThe nodes above will be added to HSM group '{}'. Do you want to proceed?",
      xname_to_move_vec, target_hsm_name
    ),
    false,
  ) {
    log::info!("Continue",);
  } else {
    return Err(Error::msg("Cancelled by user. Aborting."));
  }

  let target_hsm_group =
    backend.get_group(&shasta_token, &target_hsm_name).await;

  if target_hsm_group.is_err() {
    eprintln!(
      "Target HSM group {} does not exist, Nothing to do. Exit",
      target_hsm_name
    );
  }

  let xnames_to_move: Vec<&str> = xname_to_move_vec
    .iter()
    .map(|xname| xname.as_str())
    .collect();

  if dryrun {
    println!(
      "dryrun - Add nodes {:?} to {}",
      xnames_to_move, target_hsm_name
    );

    return Ok(());
  }

  let mut target_hsm_group_member_vec = backend
    .add_members_to_group(&shasta_token, &target_hsm_name, &xnames_to_move)
    .await
    .map_err(|e| {
      Error::msg(format!(
        "Could not add nodes {:?} to HSM group '{}'. Reason:\n{}",
        xnames_to_move, target_hsm_name, e
      ))
    })?;

  target_hsm_group_member_vec.sort();
  println!(
    "HSM '{}' members: {:?}",
    target_hsm_name, target_hsm_group_member_vec
  );

  // Audit
  if let Some(kafka_audit) = kafka_audit_opt {
    let username = jwt_ops::get_name(&shasta_token).unwrap();
    let user_id = jwt_ops::get_preferred_username(&shasta_token).unwrap();

    let msg_json = serde_json::json!(
        { "user": {"id": user_id, "name": username}, "host": {"hostname": xname_to_move_vec}, "group": vec![target_hsm_name], "message": format!("add nodes to group: {}", target_hsm_name)});

    let msg_data = serde_json::to_string(&msg_json)
      .expect("Could not serialize audit message data");

    if let Err(e) = kafka_audit.produce_message(msg_data.as_bytes()).await {
      log::warn!("Failed producing messages: {}", e);
    }
  }

  Ok(())
}
