use crate::{
    backend_dispatcher::StaticBackendDispatcher,
    common::{audit::Audit, jwt_ops, kafka::Kafka},
};
use backend_dispatcher::{error::Error, interfaces::bss::BootParametersTrait, types};
use dialoguer::theme::ColorfulTheme;

/// Updates the kernel parameters for a set of nodes
/// reboots the nodes which kernel params have changed
pub async fn exec(
    backend: StaticBackendDispatcher,
    shasta_token: &str,
    kernel_params: &str,
    xname_vec: Vec<String>,
    assume_yes: bool,
    kafka_audit_opt: Option<&Kafka>,
) -> Result<(), Error> {
    let mut need_restart = false;
    log::info!("Add kernel parameters");

    let mut xname_to_reboot_vec: Vec<String> = Vec::new();

    let current_node_boot_params_vec: Vec<types::BootParameters> = backend
        .get_bootparameters(
            shasta_token,
            &xname_vec
                .iter()
                .map(|xname| xname.to_string())
                .collect::<Vec<String>>(),
        )
        .await
        .unwrap();

    println!(
        "Add kernel params:\n{:?}\nFor nodes:\n{:?}",
        kernel_params,
        xname_vec.join(", ")
    );

    let proceed = dialoguer::Confirm::with_theme(
        &ColorfulTheme::default())
        .with_prompt("This operation will add the kernel parameters for the nodes below. Please confirm to proceed")
        .interact()
        .unwrap();

    if !proceed {
        println!("Operation canceled by the user. Exit");
        std::process::exit(1);
    }

    log::debug!("new kernel params: {:#?}", current_node_boot_params_vec);

    for mut boot_parameter in current_node_boot_params_vec {
        log::info!(
            "Add '{:?}' kernel parameters to '{}'",
            boot_parameter.hosts,
            kernel_params
        );

        need_restart = need_restart || boot_parameter.add_kernel_params(&kernel_params);
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

        let msg_json = serde_json::json!(
        { "user": {"id": user_id, "name": username}, "host": {"hostname": xname_vec}, "message": format!("Add kernel parameters: {}", kernel_params)});

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
