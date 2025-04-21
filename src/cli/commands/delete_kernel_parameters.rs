use backend_dispatcher::{
    error::Error,
    interfaces::{
        bss::BootParametersTrait,
        hsm::{component::ComponentTrait, group::GroupTrait},
    },
    types::{self},
};
use dialoguer::theme::ColorfulTheme;

use crate::{
    backend_dispatcher::StaticBackendDispatcher,
    common::{
        self, audit::Audit, jwt_ops, kafka::Kafka,
    },
};

/// Updates the kernel parameters for a set of nodes
/// reboots the nodes which kernel params have changed
pub async fn exec(
    backend: StaticBackendDispatcher,
    shasta_token: &str,
    kernel_params: &str,
    hosts_expression: &str,
    assume_yes: bool,
    kafka_audit_opt: Option<&Kafka>,
) -> Result<(), Error> {
    let mut need_restart = false;
    println!("Delete kernel parameters");

    let mut xname_to_reboot_vec: Vec<String> = Vec::new();

    // Convert user input to xname
    let node_metadata_available_vec = backend
        .get_node_metadata_available(shasta_token)
        .await
        .unwrap_or_else(|e| {
            eprintln!("ERROR - Could not get node metadata. Reason:\n{e}\nExit");
            std::process::exit(1);
        });

    let xname_vec = common::node_ops::from_hosts_expression_to_xname_vec(
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

        let kernel_params_changed = boot_parameter.delete_kernel_params(&kernel_params);
        need_restart = kernel_params_changed || need_restart;

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
    }

    // Reboot if needed
    if xname_to_reboot_vec.is_empty() {
        println!("Nothing to change. Exit");
    } else {
        crate::cli::commands::power_reset_nodes::exec(
            &backend,
            shasta_token,
            &xname_to_reboot_vec.join(","),
            true,
            assume_yes,
            "table",
            kafka_audit_opt,
        )
        .await;
    }

    Ok(())
}
