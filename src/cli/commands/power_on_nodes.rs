use backend_dispatcher::{
    interfaces::{
        hsm::{component::ComponentTrait, group::GroupTrait},
        pcs::PCSTrait,
    },
    types::Component,
};
use dialoguer::{theme::ColorfulTheme, Confirm};
use mesa::common::jwt_ops;

use crate::{
    backend_dispatcher::StaticBackendDispatcher,
    common::{self, audit::Audit, kafka::Kafka},
};

pub async fn exec(
    backend: &StaticBackendDispatcher,
    shasta_token: &str,
    hosts_string: &str,
    assume_yes: bool,
    output: &str,
    kafka_audit: &Kafka,
) {
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

    let mut xname_vec = common::node_ops::resolve_node_list_user_input_to_xname_2(
        hosts_string,
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
    /* let mut xname_vec = common::node_ops::resolve_node_list_user_input_to_xname(
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
    }); */

    if xname_vec.is_empty() {
        eprintln!("The list of nodes to operate is empty. Nothing to do. Exit");
        std::process::exit(0);
    }

    xname_vec.sort();
    xname_vec.dedup();

    if !assume_yes {
        if Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt(format!(
                "{:?}\nThe nodes above will be powered on. Please confirm to proceed?",
                xname_vec.join(", ")
            ))
            .interact()
            .unwrap()
        {
            log::info!("Continue",);
        } else {
            println!("Cancelled by user. Aborting.");
            std::process::exit(0);
        }
    }

    /* let operation = "on";

    let power_mgmt_summary_rslt = pcs::transitions::http_client::post_block(
        shasta_base_url,
        shasta_token,
        shasta_root_cert,
        operation,
        &xname_vec,
    )
    .await
    .map_err(|e| Error::Message(e.to_string())); */
    let power_mgmt_summary_rslt = backend.power_on_sync(shasta_token, &xname_vec).await;

    let power_mgmt_summary = match power_mgmt_summary_rslt {
        Ok(value) => value,
        Err(e) => {
            eprintln!(
                "ERROR - Could not power on node/s '{:?}'. Reason:\n{}",
                xname_vec,
                e.to_string()
            );

            std::process::exit(1);
        }
    };

    common::pcs_utils::print_summary_table(power_mgmt_summary, output);

    // Audit
    let username = jwt_ops::get_name(shasta_token).unwrap();
    let user_id = jwt_ops::get_preferred_username(shasta_token).unwrap();

    let msg_json = serde_json::json!(
        { "user": {"id": user_id, "name": username}, "host": {"hostname": xname_vec}, "message": "power on"});

    let msg_data =
        serde_json::to_string(&msg_json).expect("Could not serialize audit message data");

    if let Err(e) = kafka_audit.produce_message(msg_data.as_bytes()).await {
        log::warn!("Failed producing messages: {}", e);
    }
    // log::info!(target: "app::audit", "User: {} ({}) ; Operation: Power on nodes {:?}", jwt_ops::get_name(shasta_token).unwrap(), jwt_ops::get_preferred_username(shasta_token).unwrap(), xname_vec);
}
