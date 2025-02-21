use backend_dispatcher::interfaces::hsm::group::GroupTrait;
use dialoguer::{theme::ColorfulTheme, Confirm};
use mesa::common::jwt_ops;

use crate::{
    backend_dispatcher::StaticBackendDispatcher,
    common::{self, audit::Audit, kafka::Kafka},
};

/// Remove/unassign a list of xnames to a list of HSM groups
pub async fn exec(
    backend: &StaticBackendDispatcher,
    shasta_token: &str,
    target_hsm_name: &String,
    is_regex: bool,
    hosts_string: &str,
    dryrun: bool,
    kafka_audit_opt: Option<&Kafka>,
) {
    // Convert user input to xname
    let mut xname_to_move_vec = common::node_ops::resolve_node_list_user_input_to_xname(
        backend,
        shasta_token,
        hosts_string,
        false,
        is_regex,
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
            "{:?}\nThe nodes above will be removed from HSM group '{}'. Do you want to proceed?",
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

    if backend
        .get_group(shasta_token, target_hsm_name)
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
        std::process::exit(0);
    }

    // Remove xnames from HSM group
    for xname in &xname_to_move_vec {
        let _ = backend
            .delete_member_from_group(shasta_token, &target_hsm_name, &xname)
            .await
            .unwrap();
    }

    // Audit
    if let Some(kafka_audit) = kafka_audit_opt {
        let username = jwt_ops::get_name(shasta_token).unwrap();
        let user_id = jwt_ops::get_preferred_username(shasta_token).unwrap();

        let msg_json = serde_json::json!(
        { "user": {"id": user_id, "name": username}, "host": {"hostname": xname_to_move_vec}, "message": format!("Remove nodes from group '{}'", target_hsm_name)});

        let msg_data =
            serde_json::to_string(&msg_json).expect("Could not serialize audit message data");

        if let Err(e) = kafka_audit.produce_message(msg_data.as_bytes()).await {
            log::warn!("Failed producing messages: {}", e);
        }
    }
}
