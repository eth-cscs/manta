use dialoguer::theme::ColorfulTheme;
use csm_rs::{
    bss::{self, bootparameters::BootParameters},
    common::jwt_ops,
    error::Error,
};

use crate::common::{audit::Audit, kafka::Kafka};

/// Updates the kernel parameters for a set of nodes
/// reboots the nodes which kernel params have changed
pub async fn exec(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    kernel_params: &str,
    hsm_group_name_opt: Option<&Vec<String>>,
    xname_vec_opt: Option<&Vec<String>>,
    assume_yes: bool,
    kafka_audit: &Kafka,
) -> Result<(), Error> {
    let mut need_restart = false;
    println!("Add kernel parameters");

    let mut xname_to_reboot_vec: Vec<String> = Vec::new();

    let xnames = if let Some(hsm_group_name_vec) = hsm_group_name_opt {
        csm_rs::hsm::group::utils::get_member_vec_from_hsm_name_vec(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            hsm_group_name_vec.clone(),
        )
        .await
    } else if let Some(xname_vec) = xname_vec_opt {
        xname_vec.clone()
    } else {
        return Err(Error::Message(
            "ERROR - Adding kernel parameters without a list of nodes".to_string(),
        ));
    };

    // Get current node boot params
    let current_node_boot_params_vec: Vec<BootParameters> = bss::bootparameters::http_client::get(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        &xnames
            .iter()
            .map(|xname| xname.to_string())
            .collect::<Vec<String>>(),
    )
    .await
    .unwrap();

    println!(
        "Add kernel params:\n{:?}\nFor nodes:\n{:?}",
        kernel_params,
        xnames.join(", ")
    );

    let proceed = dialoguer::Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("This operation will add the kernel parameters for the nodes above. Please confirm to proceed")
        .interact()
        .unwrap();

    if !proceed {
        println!("Operation canceled by the user. Exit");
        std::process::exit(1);
    }

    /* // Add kernel parameters
    current_node_boot_params_vec
        .iter_mut()
        .for_each(|boot_parameter| {
            log::info!(
                "Add '{:?}' kernel parameters to '{}'",
                boot_parameter.hosts,
                kernel_params
            );

            need_restart = need_restart || boot_parameter.add_kernel_param(&kernel_params);
            log::info!("need restart? {}", need_restart);
        }); */

    log::debug!("new kernel params: {:#?}", current_node_boot_params_vec);

    for mut boot_parameter in current_node_boot_params_vec {
        log::info!(
            "Add '{}' kernel parameters to '{:?}'",
            kernel_params,
            boot_parameter.hosts,
        );

        need_restart = boot_parameter.add_kernel_params(&kernel_params);

        log::info!("need restart? {}", need_restart);

        if need_restart {
            // boot_parameter.params = kernel_params.to_string();

            let _ = csm_rs::bss::bootparameters::http_client::put(
                shasta_base_url,
                shasta_token,
                shasta_root_cert,
                boot_parameter.clone(),
            )
            .await;

            need_restart |= need_restart;

            if need_restart {
                xname_to_reboot_vec.push(boot_parameter.hosts.first().unwrap().to_string());
            }
        }
    }

    // Audit
    let username = jwt_ops::get_name(shasta_token).unwrap();
    let user_id = jwt_ops::get_preferred_username(shasta_token).unwrap();

    let msg_json = serde_json::json!(
        { "user": {"id": user_id, "name": username}, "host": {"hostname": xnames}, "group": hsm_group_name_opt.map(|group_name| vec![group_name]), "message": format!("Add kernel parameters: {}", kernel_params)});

    let msg_data =
        serde_json::to_string(&msg_json).expect("Could not serialize audit message data");

    if let Err(e) = kafka_audit.produce_message(msg_data.as_bytes()).await {
        log::warn!("Failed producing messages: {}", e);
    }
    // log::info!(target: "app::audit", "User: {} ({}) ; Operation: Delete kernel parameters to {:?}", jwt_ops::get_name(shasta_token).unwrap(), jwt_ops::get_preferred_username(shasta_token).unwrap(), xnames);

    // Reboot if needed
    if xname_to_reboot_vec.is_empty() {
        println!("Nothing to change. Exit");
    } else {
        crate::cli::commands::power_reset_nodes::exec(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            &xname_to_reboot_vec.join(","),
            false,
            true,
            assume_yes,
            "table",
            kafka_audit,
        )
        .await;
    }

    Ok(())
}
