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
    common::{kafka::Kafka, node_ops::resolve_node_list_user_input_to_xname_2},
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
        dbg!(&boot_parameter);
        log::info!(
            "Deleting '{}' kernel parameters for nodes '{:?}'",
            kernel_params,
            boot_parameter.hosts,
        );

        let is_kernel_params_deleted = boot_parameter.delete_kernel_params(&kernel_params);

        need_restart = need_restart || is_kernel_params_deleted;
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
    log::info!(target: "app::audit", "User: {} ({}) ; Operation: Delete kernel parameters to {:?}", jwt_ops::get_name(shasta_token).unwrap_or("".to_string()), jwt_ops::get_preferred_username(shasta_token).unwrap_or("".to_string()), node_expression);

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
