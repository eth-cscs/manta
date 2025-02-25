use backend_dispatcher::{
    interfaces::{
        bss::BootParametersTrait,
        hsm::{component::ComponentTrait, group::GroupTrait},
    },
    types::{BootParameters, Component},
};
use dialoguer::theme::ColorfulTheme;
use mesa::{common::jwt_ops, error::Error};

use crate::{
    backend_dispatcher::StaticBackendDispatcher,
    common::{self, audit::Audit, kafka::Kafka},
};

/// Updates the kernel parameters for a set of nodes
/// reboots the nodes which kernel params have changed
pub async fn exec(
    backend: &StaticBackendDispatcher,
    shasta_token: &str,
    kernel_params: &str,
    node_expression: &str,
    assume_yes: bool,
    kafka_audit_opt: Option<&Kafka>,
) -> Result<(), Error> {
    let mut need_restart = false;
    println!("Delete boot parameters");

    // Convert user input to xname
    let xname_available_vec: Vec<String> = backend
        .get_group_available(shasta_token)
        .await
        .unwrap_or_else(|e| {
            eprintln!(
                "ERROR - Could not get group list. Reason:\n{}",
                e.to_string()
            );
            std::process::exit(1);
        })
        .iter()
        .flat_map(|group| group.get_members())
        .collect();

    let node_metadata_vec: Vec<Component> = backend
        .get_all_nodes(shasta_token, Some("true"))
        .await
        .unwrap()
        .components
        .unwrap_or_default()
        .iter()
        .filter(|&node_metadata| xname_available_vec.contains(&node_metadata.id.as_ref().unwrap()))
        .cloned()
        .collect();

    let xname_vec = common::node_ops::resolve_node_list_user_input_to_xname_2(
        node_expression,
        false,
        node_metadata_vec,
    )
    .await
    .unwrap_or_else(|e| {
        eprintln!(
            "ERROR - Could not convert user input to list of xnames. Reason:\n{}",
            e
        );
        std::process::exit(1);
    });

    log::debug!("Delete boot params for nodes:\n{:?}", xname_vec.join(", "));

    let proceed = dialoguer::Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("This operation will delete the boot parameters for the nodes below. Please confirm to proceed")
        .interact()
        .unwrap();

    if !proceed {
        println!("Operation canceled by the user. Exit");
        std::process::exit(1);
    }

    let boot_parameters_vec = BootParameters {
        hosts: xname_vec.clone(),
        macs: None,
        nids: None,
        params: String::from(""),
        kernel: String::from(""),
        initrd: String::from(""),
        cloud_init: None,
    };

    backend
        .delete_bootparameters(shasta_token, &boot_parameters_vec)
        .await
        .map_err(|e| Error::Message(e.to_string()))?;

    // Audit
    if let Some(kafka_audit) = kafka_audit_opt {
        let username = jwt_ops::get_name(shasta_token).unwrap_or_default();
        let user_id = jwt_ops::get_preferred_username(shasta_token).unwrap_or_default();

        let msg_json = serde_json::json!(
        { "user": {"id": user_id, "name": username}, "host": {"hostname": xname_vec}, "message": format!("Delete boot parameters")});

        let msg_data =
            serde_json::to_string(&msg_json).expect("Could not serialize audit message data");

        if let Err(e) = kafka_audit.produce_message(msg_data.as_bytes()).await {
            log::warn!("Failed producing messages: {}", e);
        }
    }

    Ok(())
}
