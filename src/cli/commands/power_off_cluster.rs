use anyhow::Error;
// use dialoguer::{Confirm, theme::ColorfulTheme};
use manta_backend_dispatcher::interfaces::{
  hsm::group::GroupTrait, pcs::PCSTrait,
};

use crate::{
  common::{self, audit::Audit, jwt_ops, kafka::Kafka, authentication::get_api_token, authorization::get_groups_names_available},
  manta_backend_dispatcher::StaticBackendDispatcher,
};
use nodeset::NodeSet;

pub async fn exec(
  backend: &StaticBackendDispatcher,
  site_name: &str,
  hsm_group_name_arg: &str,
  settings_hsm_group_name_opt: Option<&String>,
  force: bool,
  assume_yes: bool,
  output: &str,
  kafka_audit_opt: Option<&Kafka>,
) -> Result<(), Error> {
  let shasta_token = get_api_token(backend, site_name).await?;

  let target_hsm_group_vec = get_groups_names_available(
    backend,
    &shasta_token,
    Some(&hsm_group_name_arg.to_string()),
    settings_hsm_group_name_opt,
  )
  .await?;

  let target_hsm_group = target_hsm_group_vec
    .first()
    .expect("The 'cluster name' argument must have a value");

  let xname_vec = backend
    .get_member_vec_from_group_name_vec(
      &shasta_token,
      &[target_hsm_group.to_string()],
    )
    .await
    .unwrap();

  let node_group: NodeSet = xname_vec.join(", ").parse().unwrap();

  println!(
    "Number of nodes: {}\nlist of nodes: {}",
    node_group.len(),
    node_group.to_string()
  );

  if common::user_interaction::confirm(
    "The nodes above will be powered off. Please confirm to proceed?",
    assume_yes,
  ) {
    log::info!("Continue",);
  } else {
    return Err(Error::msg("Cancelled by user. Aborting."));
  }

  let power_mgmt_summary_rslt = backend
    .power_off_sync(&shasta_token, &xname_vec, force)
    .await;

  let power_mgmt_summary = match power_mgmt_summary_rslt {
    Ok(value) => value,
    Err(e) => {
      return Err(Error::msg(format!(
        "Could not power off node/s '{:?}'. Reason:\n{}",
        xname_vec,
        e.to_string()
      )));
    }
  };

  common::pcs_utils::print_summary_table(power_mgmt_summary, output);

  // Audit
  if let Some(kafka_audit) = kafka_audit_opt {
    let username = jwt_ops::get_name(&shasta_token).unwrap();
    let user_id = jwt_ops::get_preferred_username(&shasta_token).unwrap();

    let msg_json = serde_json::json!(
        { "user": {"id": user_id, "name": username}, "group": hsm_group_name_arg, "message": "power off"});

    let msg_data = serde_json::to_string(&msg_json)
      .expect("Could not serialize audit message data");

    if let Err(e) = kafka_audit.produce_message(msg_data.as_bytes()).await {
      return Err(Error::msg(format!(
        "Failed producing audit messages: {}",
        e.to_string()
      )));
    }
  }

  Ok(())
}
