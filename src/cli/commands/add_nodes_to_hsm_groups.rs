use dialoguer::{theme::ColorfulTheme, Confirm};
use manta_backend_dispatcher::interfaces::hsm::{
  component::ComponentTrait, group::GroupTrait,
};

use crate::{
  common::{self, audit::Audit, jwt_ops, kafka::Kafka},
  manta_backend_dispatcher::StaticBackendDispatcher,
};

/// Add/assign a list of xnames to a list of HSM groups
pub async fn exec(
  backend: &StaticBackendDispatcher,
  shasta_token: &str,
  target_hsm_name: &String,
  hosts_expression: &str,
  dryrun: bool,
  kafka_audit_opt: Option<&Kafka>,
) {
  // Convert user input to xname
  let node_metadata_available_vec = backend
    .get_node_metadata_available(shasta_token)
    .await
    .unwrap_or_else(|e| {
      eprintln!("ERROR - Could not get node metadata. Reason:\n{e}\nExit");
      std::process::exit(1);
    });

  let mut xname_to_move_vec =
    common::node_ops::from_hosts_expression_to_xname_vec(
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

  xname_to_move_vec.sort();
  xname_to_move_vec.dedup();

  // Check if there are any xname to migrate/move and exit otherwise
  if xname_to_move_vec.is_empty() {
    println!("No hosts to move. Exit");
    std::process::exit(0);
  }

  if Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(format!(
            "{:?}\nThe nodes above will be added to HSM group '{}'. Do you want to proceed?",
            xname_to_move_vec, target_hsm_name
        ))
        .interact()
        .unwrap()
    {
        log::info!("Continue",);
    } else {
        println!("Cancelled by user. Aborting.");
        std::process::exit(0);
    }

  let target_hsm_group =
    backend.get_group(shasta_token, &target_hsm_name).await;

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
    std::process::exit(0);
  }

  let node_migration_rslt = backend
    .add_members_to_group(shasta_token, &target_hsm_name, xnames_to_move)
    .await;

  match node_migration_rslt {
    Ok(mut target_hsm_group_member_vec) => {
      target_hsm_group_member_vec.sort();
      println!(
        "HSM '{}' members: {:?}",
        target_hsm_name, target_hsm_group_member_vec
      );
    }
    Err(e) => eprintln!("{}", e),
  }

  // Audit
  if let Some(kafka_audit) = kafka_audit_opt {
    let username = jwt_ops::get_name(shasta_token).unwrap();
    let user_id = jwt_ops::get_preferred_username(shasta_token).unwrap();

    let msg_json = serde_json::json!(
        { "user": {"id": user_id, "name": username}, "host": {"hostname": xname_to_move_vec}, "group": vec![target_hsm_name], "message": format!("add nodes to group: {}", target_hsm_name)});

    let msg_data = serde_json::to_string(&msg_json)
      .expect("Could not serialize audit message data");

    if let Err(e) = kafka_audit.produce_message(msg_data.as_bytes()).await {
      log::warn!("Failed producing messages: {}", e);
    }
  }
}
