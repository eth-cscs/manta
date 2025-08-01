use dialoguer::{theme::ColorfulTheme, Confirm};
use manta_backend_dispatcher::interfaces::{
  hsm::{component::ComponentTrait, group::GroupTrait},
  pcs::PCSTrait,
};

use crate::{
  common::{self, audit::Audit, jwt_ops, kafka::Kafka},
  manta_backend_dispatcher::StaticBackendDispatcher,
};
use nodeset::NodeSet;

pub async fn exec(
  backend: &StaticBackendDispatcher,
  shasta_token: &str,
  hosts_expression: &str,
  force: bool,
  assume_yes: bool,
  output: &str,
  kafka_audit_opt: Option<&Kafka>,
) {
  // Filter xnames to the ones members to HSM groups the user has access to
  //
  // Convert user input to xname
  let node_metadata_available_vec = backend
    .get_node_metadata_available(shasta_token)
    .await
    .unwrap_or_else(|e| {
      eprintln!("ERROR - Could not get node metadata. Reason:\n{e}\nExit");
      std::process::exit(1);
    });

  let mut xname_vec = common::node_ops::from_hosts_expression_to_xname_vec(
    hosts_expression,
    false,
    node_metadata_available_vec,
  )
  .await
  .unwrap_or_else(|e| {
    eprintln!(
      "ERROR - Could not convert user input to list of xnames. Reason:\n{}",
      e
    );
    std::process::exit(1);
  });

  if xname_vec.is_empty() {
    eprintln!("The list of nodes to operate is empty. Nothing to do. Exit");
    std::process::exit(0);
  }

  xname_vec.sort();
  xname_vec.dedup();

  let node_group: NodeSet = xname_vec.join(", ").parse().unwrap();

  println!(
    "Number of nodes: {}\nlist of nodes: {}",
    node_group.len(),
    node_group.to_string()
  );

  if !assume_yes {
    if Confirm::with_theme(&ColorfulTheme::default())
      .with_prompt("The nodes above will restart. Please confirm to proceed?")
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
    .power_reset_sync(shasta_token, &xname_vec, force)
    .await;

  let power_mgmt_summary = match power_mgmt_summary_rslt {
    Ok(value) => value,
    Err(e) => {
      eprintln!(
        "ERROR - Could not restart node/s '{:?}'. Reason:\n{}",
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

    let group_map = backend
      .get_group_map_and_filter_by_member_vec(
        shasta_token,
        &xname_vec
          .iter()
          .map(|member| member.as_str())
          .collect::<Vec<_>>(),
      )
      .await
      .unwrap();

    let msg_json = serde_json::json!(
        { "user": {"id": user_id, "name": username}, "host": {"hostname": xname_vec}, "group": group_map.keys().collect::<Vec<_>>(), "message": "power reset"});

    let msg_data = serde_json::to_string(&msg_json)
      .expect("Could not serialize audit message data");

    if let Err(e) = kafka_audit.produce_message(msg_data.as_bytes()).await {
      log::warn!("Failed producing messages: {}", e);
    }
  }
}
