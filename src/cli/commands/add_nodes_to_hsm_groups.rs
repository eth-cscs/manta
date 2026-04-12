use anyhow::{Context, Error, bail};

use manta_backend_dispatcher::interfaces::hsm::group::GroupTrait;

use crate::{
  common::{self, audit, authentication::get_api_token, kafka::Kafka},
  manta_backend_dispatcher::StaticBackendDispatcher,
};

/// Add/assign a list of xnames to a list of HSM groups
pub async fn exec(
  backend: &StaticBackendDispatcher,
  site_name: &str,
  target_hsm_name: &str,
  hosts_expression: &str,
  dryrun: bool,
  kafka_audit_opt: Option<&Kafka>,
) -> Result<(), Error> {
  let shasta_token = get_api_token(backend, site_name).await?;

  // Convert user input to xname
  let xname_to_move_vec = common::node_ops::resolve_hosts_expression(
    backend,
    &shasta_token,
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
      "{:?}\nThe nodes above will be added to HSM group '{}'. Do you want to proceed?",
      xname_to_move_vec, target_hsm_name
    ),
    false,
  ) {
    log::info!("Continue",);
  } else {
    bail!("Operation cancelled by user");
  }

  let target_hsm_group =
    backend.get_group(&shasta_token, target_hsm_name).await;

  if target_hsm_group.is_err() {
    bail!(
      "Target HSM group '{}' does not exist. \
       Nothing to do",
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
    .add_members_to_group(&shasta_token, target_hsm_name, &xnames_to_move)
    .await
    .with_context(|| {
      format!(
        "Could not add nodes {:?} \
           to HSM group '{}'",
        xnames_to_move, target_hsm_name
      )
    })?;

  target_hsm_group_member_vec.sort();
  println!(
    "HSM '{}' members: {:?}",
    target_hsm_name, target_hsm_group_member_vec
  );

  // Audit
  if let Some(kafka_audit) = kafka_audit_opt {
    audit::send_audit(
      kafka_audit,
      &shasta_token,
      format!("add nodes to group: {}", target_hsm_name),
      Some(serde_json::json!(xname_to_move_vec)),
      Some(serde_json::json!(vec![target_hsm_name])),
    )
    .await;
  }

  Ok(())
}
