use dialoguer::{theme::ColorfulTheme, Confirm};
use manta_backend_dispatcher::interfaces::{
  hsm::group::GroupTrait, pcs::PCSTrait,
};

use crate::{
  common::{self, audit::Audit, jwt_ops, kafka::Kafka},
  manta_backend_dispatcher::StaticBackendDispatcher,
};
use nodeset::NodeSet;

pub async fn exec(
  backend: &StaticBackendDispatcher,
  shasta_token: &str,
  hsm_group_name_arg: &str,
  force: bool,
  assume_yes: bool,
  output: &str,
  kafka_audit_opt: Option<&Kafka>,
) {
  let xname_vec = backend
    .get_member_vec_from_group_name_vec(
      shasta_token,
      vec![hsm_group_name_arg.to_string()],
    )
    .await
    .unwrap();

  let node_group: NodeSet = xname_vec.join(", ").parse().unwrap();

  println!(
    "Number of nodes: {}\nlist of nodes: {}",
    node_group.len(),
    node_group.to_string()
  );

  if !assume_yes {
    if Confirm::with_theme(&ColorfulTheme::default())
      .with_prompt(
        "The nodes above will be powered off. Please confirm to proceed?",
      )
      .interact()
      .unwrap()
    {
      log::info!("Continue",);
    } else {
      println!("Cancelled by user. Aborting.");
      std::process::exit(0);
    }
  }

  let power_mgmt_summary_rslt = backend
    .power_off_sync(shasta_token, &xname_vec, force)
    .await;

  let power_mgmt_summary = match power_mgmt_summary_rslt {
    Ok(value) => value,
    Err(e) => {
      eprintln!(
        "ERROR - Could not power off node/s '{:?}'. Reason:\n{}",
        xname_vec,
        e.to_string()
      );

      std::process::exit(1);
    }
  };

  common::pcs_utils::print_summary_table(power_mgmt_summary, output);

  // Audit
  if let Some(kafka_audit) = kafka_audit_opt {
    let username = jwt_ops::get_name(shasta_token).unwrap();
    let user_id = jwt_ops::get_preferred_username(shasta_token).unwrap();

    let msg_json = serde_json::json!(
        { "user": {"id": user_id, "name": username}, "group": hsm_group_name_arg, "message": "power off"});

    let msg_data = serde_json::to_string(&msg_json)
      .expect("Could not serialize audit message data");

    if let Err(e) = kafka_audit.produce_message(msg_data.as_bytes()).await {
      log::warn!("Failed producing messages: {}", e);
    }
  }
}
