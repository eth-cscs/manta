use backend_dispatcher::{
    interfaces::{
        bss::BootParametersTrait,
        hsm::{component::ComponentTrait, group::GroupTrait},
    },
    types::{self, Component},
};
use dialoguer::theme::ColorfulTheme;
use mesa::{common::jwt_ops, error::Error};

use crate::{
    backend_dispatcher::StaticBackendDispatcher,
    common::{audit::Audit, kafka::Kafka, node_ops::resolve_node_list_user_input_to_xname_2},
};

/// Updates the kernel parameters for a set of nodes
/// reboots the nodes which kernel params have changed
pub async fn exec(
    backend: StaticBackendDispatcher,
    shasta_token: &str,
    kernel_params: &str,
    node_expression: &str,
    assume_yes: bool,
    kafka_audit_opt: Option<&Kafka>,
) -> Result<(), Error> {
    let mut need_restart = false;
    println!("Delete kernel parameters");

    let mut xname_to_reboot_vec: Vec<String> = Vec::new();

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

    let xname_vec =
        resolve_node_list_user_input_to_xname_2(node_expression, false, node_metadata_vec)
            .await
            .unwrap_or_else(|e| {
                eprintln!(
                    "ERROR - Could not convert user input to list of xnames. Reason:\n{}",
                    e
                );
                std::process::exit(1);
            });

    let current_node_boot_params_vec: Vec<types::BootParameters> = backend
        .get_bootparameters(shasta_token, &xname_vec)
        .await
        .unwrap();

    println!(
        "Delete kernel params:\n{:?}\nFor nodes:\n{:?}",
        kernel_params,
        xname_vec.join(", ")
    );

    let proceed = dialoguer::Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("This operation will delete the kernel parameters for the nodes below. Please confirm to proceed")
        .interact()
        .unwrap();

    if !proceed {
        println!("Operation canceled by the user. Exit");
        std::process::exit(1);
    }

    log::debug!(
        "Current boot parameters: {:#?}",
        current_node_boot_params_vec
    );

    for mut boot_parameter in current_node_boot_params_vec {
        log::info!(
            "Deleting '{}' kernel parameters for nodes '{:?}'",
            kernel_params,
            boot_parameter.hosts,
        );

        let is_kernel_params_deleted = boot_parameter.delete_kernel_params(&kernel_params);

        need_restart = is_kernel_params_deleted;
        log::info!("need restart? {}", need_restart);

        if need_restart {
            let boot_parametes_rslt = backend
                .update_bootparameters(shasta_token, &boot_parameter)
                .await;

            if let Err(e) = boot_parametes_rslt {
                eprintln!("{:#?}", e);
                std::process::exit(1);
            }

            if need_restart {
                xname_to_reboot_vec = [xname_to_reboot_vec, boot_parameter.hosts].concat();
                xname_to_reboot_vec.sort();
                xname_to_reboot_vec.dedup();
            }
        }
    }

    /* // Audit
    log::info!(target: "app::audit", "User: {} ({}) ; Operation: Delete kernel parameters to {:?}", jwt_ops::get_name(shasta_token).unwrap_or("".to_string()), jwt_ops::get_preferred_username(shasta_token).unwrap_or("".to_string()), node_expression); */

    // Audit
    if let Some(kafka_audit) = kafka_audit_opt {
        let username = jwt_ops::get_name(shasta_token).unwrap();
        let user_id = jwt_ops::get_preferred_username(shasta_token).unwrap();

        // FIXME: We should not need to make this call here but at the beginning of the method as a
        // prerequisite
        let xnames: Vec<&str> = xname_vec.iter().map(|xname| xname.as_str()).collect();

        let group_map_vec = backend
            .get_group_map_and_filter_by_member_vec(shasta_token, &xnames)
            .await
            .map_err(|e| Error::Message(e.to_string()))?;

        let msg_json = serde_json::json!(
        { "user": {"id": user_id, "name": username}, "host": {"hostname": xname_vec}, "group": group_map_vec.keys().collect::<Vec<&String>>(), "message": format!("Delete kernel parameters: {}", kernel_params)});

        let msg_data =
            serde_json::to_string(&msg_json).expect("Could not serialize audit message data");

        if let Err(e) = kafka_audit.produce_message(msg_data.as_bytes()).await {
            log::warn!("Failed producing messages: {}", e);
        }
        // log::info!(target: "app::audit", "User: {} ({}) ; Operation: Add kernel parameters to {:?}", jwt_ops::get_name(shasta_token).unwrap_or("".to_string()), jwt_ops::get_preferred_username(shasta_token).unwrap_or("".to_string()), xname_vec);
    }

    // Reboot if needed
    if xname_to_reboot_vec.is_empty() {
        println!("Nothing to change. Exit");
    } else {
        crate::cli::commands::power_reset_nodes::exec(
            &backend,
            shasta_token,
            &xname_to_reboot_vec.join(","),
            false,
            true,
            assume_yes,
            "table",
            kafka_audit_opt,
        )
        .await;
    }

    Ok(())
}
