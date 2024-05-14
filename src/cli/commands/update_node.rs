use crate::{
    cli::commands::power_reset_nodes, common::ims_ops::get_image_id_from_cfs_configuration_name,
};

use dialoguer::{theme::ColorfulTheme, Confirm};
use mesa::{
    bss::{self, bootparameters::BootParameters},
    cfs,
    node::utils::validate_xnames,
};

pub async fn exec(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    hsm_group_name: Option<&String>,
    // new_kernel_params_opt: Option<&String>,
    new_boot_image_id_opt: Option<&String>,
    new_boot_image_configuration_opt: Option<&String>,
    new_runtime_configuration_opt: Option<&String>,
    xnames: Vec<&str>,
) {
    let mut need_restart = false;

    // Validate
    //
    // Check user has provided valid XNAMES
    if hsm_group_name.is_some()
        && !validate_xnames(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            &xnames,
            hsm_group_name,
        )
        .await
    {
        eprintln!("xname/s invalid. Exit");
        std::process::exit(1);
    }

    // Check new runtime configuration exists
    let runtime_configuration_detail_list_rslt = cfs::configuration::mesa::http_client::get(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        new_runtime_configuration_opt.map(|elem| elem.as_str()),
    )
    .await;

    if runtime_configuration_detail_list_rslt.is_err()
        || runtime_configuration_detail_list_rslt.unwrap().is_empty()
    {
        eprintln!(
            "Runtime configuration '{}' does not exists. Exit",
            new_runtime_configuration_opt.unwrap()
        );
        std::process::exit(1);
    }

    // Get current node boot params
    let current_node_boot_params: Vec<BootParameters> =
        bss::bootparameters::http_client::get_boot_params(
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

    // Get new boot image
    let new_boot_image_id_opt: Option<String> =
        if let Some(new_boot_image_configuration) = new_boot_image_configuration_opt {
            let new_boot_image_id_opt = get_image_id_from_cfs_configuration_name(
                shasta_token,
                shasta_base_url,
                shasta_root_cert,
                new_boot_image_configuration.to_string(),
            )
            .await;

            if new_boot_image_id_opt.is_some() {
                println!(
                    "Image related to configuration '{}' found.",
                    new_boot_image_configuration,
                );
            }

            new_boot_image_id_opt
        } else if let Some(boot_image_id) = new_boot_image_id_opt {
            log::info!("Boot image does not change '{}'", boot_image_id);
            Some(boot_image_id).cloned()
        } else {
            None
        };

    // Update boot image
    // Check if boot image changes and notify the user and update the node boot params struct
    if let Some(new_boot_image_id) = new_boot_image_id_opt {
        let boot_params_to_update_vec: Vec<&BootParameters> = current_node_boot_params
            .iter()
            .filter(|boot_param| boot_param.get_boot_image() != new_boot_image_id)
            .collect();
        if !boot_params_to_update_vec.is_empty() {
            if Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt(format!(
                "New boot image detected. This operation will reboot the nodes so they can boot with the new image:\n{:?}\nDo you want to continue?",
                xnames
            ))
                .interact()
                .unwrap()
            {
                log::info!("Continue",);
            } else {
                println!("Cancelled by user. Aborting.");
                std::process::exit(0);
            }

            // Update boot image
            for mut boot_parameter in current_node_boot_params {
                boot_parameter.set_boot_image(&new_boot_image_id);

                println!(
                    "Updating '{:?}' boot image to '{}'",
                    boot_parameter.hosts, new_boot_image_id
                );

                let component_patch_rep = mesa::bss::bootparameters::http_client::patch(
                    shasta_base_url,
                    shasta_token,
                    shasta_root_cert,
                    &boot_parameter,
                )
                .await;

                log::debug!(
                    "Component boot parameters resp:\n{:#?}",
                    component_patch_rep
                );
            }

            need_restart = true;
        } else {
            println!("Boot image did not change. No need to reboot.");
        }
    } else {
        log::info!("Boot image not defined. No need to reboot.");
    }

    // Update desired configuration
    if let Some(desired_configuration_name) = new_runtime_configuration_opt {
        println!(
            "Updating runtime configuration to '{}'",
            desired_configuration_name
        );

        mesa::cfs::component::shasta::utils::update_component_list_desired_configuration(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            xnames.iter().map(|xname| xname.to_string()).collect(), // TODO: modify function signature
            // for this field so it accepts
            // Vec<&str> instead of
            // Vec<String>
            desired_configuration_name,
            true,
        )
        .await;
    } else {
        log::info!("Runtime configuration does not change.");
    }

    if need_restart {
        log::info!("Restarting nodes");

        let nodes: Vec<String> = xnames.into_iter().map(|xname| xname.to_string()).collect();

        power_reset_nodes::exec(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            nodes,
            None,
            true,
        )
        .await
    }
}

/* pub async fn exec(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    hsm_group_name: Option<&String>,
    _new_boot_image_id_opt: Option<&String>,
    new_boot_image_configuration_opt: Option<&String>,
    new_runtime_configuration_optb: Option<&String>,
    xnames: Vec<&str>,
) {
    let mut need_restart = false;

    // Validate
    //
    // Check user has provided valid XNAMES
    if hsm_group_name.is_some()
        && !validate_xnames(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            &xnames,
            hsm_group_name,
        )
        .await
    {
        eprintln!("xname/s invalid. Exit");
        std::process::exit(1);
    }

    let runtime_configuration_detail_list_rslt = cfs::configuration::mesa::http_client::get(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        new_runtime_configuration_optb.map(|elem| elem.as_str()),
    )
    .await;

    // Check desired configuration exists
    if runtime_configuration_detail_list_rslt.is_err()
        || runtime_configuration_detail_list_rslt.unwrap().is_empty()
    {
        eprintln!(
            "Desired configuration '{}' does not exists. Exit",
            new_runtime_configuration_optb.unwrap()
        );
        std::process::exit(1);
    };

    log::info!(
        "Desired configuration '{}' exists",
        new_runtime_configuration_optb.unwrap()
    );

    // Get new boot image id
    let (new_boot_image_id_opt, mut node_boot_params_opt) =
        if let Some(new_boot_image_cfs_configuration_name) = new_boot_image_configuration_opt {
            // Get image id related to the boot CFS configuration
            let new_boot_image_id_opt = get_image_id_from_cfs_configuration_name(
                shasta_token,
                shasta_base_url,
                shasta_root_cert,
                new_boot_image_cfs_configuration_name.clone(),
            )
            .await;

            if new_boot_image_id_opt.is_none() {
                eprintln!(
                    "Image ID related to CFS configuration name '{}' not found. Exit",
                    new_boot_image_cfs_configuration_name
                );
                std::process::exit(1);
            }

            let boot_image_value_vec: Vec<Value> = ims::image::shasta::http_client::get(
                shasta_token,
                shasta_base_url,
                shasta_root_cert,
                new_boot_image_id_opt.as_deref(),
            )
            .await
            .unwrap();

            if boot_image_value_vec.is_empty() {
                eprintln!(
                    "Image ID '{}' not found in CSM. Exit",
                    new_boot_image_id_opt.unwrap()
                );
                std::process::exit(1);
            }

            let new_boot_image_id = boot_image_value_vec.first().unwrap()["id"]
                .as_str()
                .unwrap()
                .to_string();

            log::info!("Boot image ID '{}' found in CSM", new_boot_image_id);

            // Get node boot params
            let node_boot_params: BootParameters = bss::http_client::get_boot_params(
                shasta_token,
                shasta_base_url,
                shasta_root_cert,
                &xnames
                    .iter()
                    .map(|xname| xname.to_string())
                    .collect::<Vec<String>>(),
            )
            .await
            .unwrap()
            .first_mut()
            .unwrap()
            .clone();

            // Get current image id
            let current_boot_image_id = node_boot_params.get_boot_image();

            // Check if new image id is different to the current one to find out if need to restart
            if current_boot_image_id != new_boot_image_id {
                need_restart = true;
            } else {
                println!("Boot image does not change. No need to reboot.");
            }

            (Some(new_boot_image_id), Some(node_boot_params))
        } else {
            (None, None)
        };

    if need_restart {
        if Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt(format!(
                "This operation will reboot the following nodes:\n{:?}\nDo you want to continue?",
                xnames
            ))
            .interact()
            .unwrap()
        {
            log::info!("Continue",);
        } else {
            println!("Cancelled by user. Aborting.");
            std::process::exit(0);
        }

        println!(
            "Updating boot configuration to '{}'",
            new_boot_image_configuration_opt.unwrap()
        );

        // Update root kernel param to it uses the new image id
        if let Some(node_boot_params) = &mut node_boot_params_opt {
            node_boot_params.set_boot_image(&new_boot_image_id_opt.unwrap());
        }

        let component_patch_rep = mesa::bss::http_client::patch(
            shasta_base_url,
            shasta_token,
            shasta_root_cert,
            node_boot_params_opt.as_ref().unwrap(),
        )
        .await;

        log::debug!(
            "Component boot parameters resp:\n{:#?}",
            component_patch_rep
        );

        log::info!("Boot params for nodes {:?} updated", xnames);

        // Update desired configuration

        if let Some(desired_configuration_name) = new_runtime_configuration_optb {
            println!(
                "Updating desired configuration to '{}'",
                desired_configuration_name
            );

            mesa::cfs::component::shasta::utils::update_component_list_desired_configuration(
                shasta_token,
                shasta_base_url,
                shasta_root_cert,
                xnames.iter().map(|xname| xname.to_string()).collect(), // TODO: modify function signature
                // for this field so it accepts
                // Vec<&str> instead of
                // Vec<String>
                desired_configuration_name,
                true,
            )
            .await;
        }

        // Create BOS session. Note: reboot operation shuts down the nodes and don't bring them back
        // up... hence we will split the reboot into 2 operations shutdown and start

        log::info!("Restarting nodes");

        let nodes: Vec<String> = xnames.into_iter().map(|xname| xname.to_string()).collect();

        // Create CAPMC operation shutdown
        let capmc_shutdown_nodes_resp = capmc::http_client::node_power_off::post_sync(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            nodes.clone(),
            Some("Update node boot params and/or desired configuration".to_string()),
            true,
        )
        .await;

        log::debug!(
            "CAPMC shutdown nodes response:\n{:#?}",
            capmc_shutdown_nodes_resp
        );

        // Create CAPMC operation to start
        let capmc_start_nodes_resp = capmc::http_client::node_power_on::post(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            nodes,
            Some("Update node boot params and/or desired configuration".to_string()),
        )
        .await;

        log::debug!(
            "CAPMC starting nodes response:\n{:#?}",
            capmc_start_nodes_resp
        );
    }
} */
