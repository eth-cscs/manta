use crate::{
    cli::commands::power_reset_nodes, common::ims_ops::get_image_id_from_cfs_configuration_name,
};

use dialoguer::{theme::ColorfulTheme, Confirm};
use mesa::{
    bss::{self, r#struct::BootParameters},
    cfs, ims,
    node::utils::validate_xnames_format_and_membership_agaisnt_multiple_hsm,
};

use super::config_show::get_hsm_name_available_from_jwt_or_all;

pub async fn exec(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    new_boot_image_id_opt: Option<&String>,
    new_boot_image_configuration_opt: Option<&String>,
    new_runtime_configuration_opt: Option<&String>,
    new_kernel_parameters_opt: Option<&String>,
    xname_vec: Vec<&str>,
    assume_yes: bool,
    dry_run: bool,
) {
    let mut need_restart = false;

    // Validate
    //
    // Check user has provided valid XNAMES
    let target_hsm_group_vec =
        get_hsm_name_available_from_jwt_or_all(shasta_token, shasta_base_url, shasta_root_cert)
            .await;

    if !validate_xnames_format_and_membership_agaisnt_multiple_hsm(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        &xname_vec,
        Some(target_hsm_group_vec),
    )
    .await
    {
        eprintln!("xname/s invalid. Exit");
        std::process::exit(1);
    }

    // Check new configuration exists and exit otherwise
    let runtime_configuration_detail_list_rslt = cfs::configuration::http_client::v3::get(
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

    /* let backup_config_rslt = mesa::config::Config::new(
        &shasta_base_url.to_string(),
        Some(&shasta_token.to_string()),
        &shasta_root_cert.to_vec(),
        "csm", // FIXME: do not hardcode this value and move it to config file
    )
    .await;

    let backup_config = match backup_config_rslt {
        Ok(backup_config) => backup_config,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };

    let mut current_node_boot_param_vec: Vec<BootParameters> = backup_config
        .get_bootparameters(
            &xname_vec
                .iter()
                .map(|xname| xname.to_string())
                .collect::<Vec<String>>(),
        )
        .await
        .unwrap(); */

    // Get current node boot params
    let mut current_node_boot_param_vec: Vec<BootParameters> = bss::http_client::get(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        &xname_vec
            .iter()
            .map(|xname| xname.to_string())
            .collect::<Vec<String>>(),
    )
    .await
    .unwrap();

    // Get new boot image
    let new_boot_image_id_opt: Option<String> =
        if let Some(new_boot_image_configuration) = new_boot_image_configuration_opt {
            log::info!(
                "Boot configuration '{}' provided",
                new_boot_image_configuration
            );
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

            if new_boot_image_id_opt == None {
                eprintln!(
                    "ERROR - Could not find boot image related to configuration '{}'",
                    new_boot_image_configuration
                );
                std::process::exit(1);
            }

            new_boot_image_id_opt
        } else if let Some(boot_image_id) = new_boot_image_id_opt {
            log::info!("Boot image id '{}' provided", boot_image_id);
            // Check image id exists
            let image_id_in_csm = ims::image::http_client::get(
                shasta_token,
                shasta_base_url,
                shasta_root_cert,
                new_boot_image_id_opt.map(|image_id| image_id.as_str()),
            )
            .await;

            if image_id_in_csm.is_err() {
                eprintln!("ERROR - boot image id '{}' not found", boot_image_id);
                std::process::exit(1);
            }

            Some(boot_image_id).cloned()
        } else {
            None
        };

    // Update BSS BOOT PARAMETERS
    //
    // IMPORTANT: ALWAYS SET KERNEL PARAMS BEFORE BOOT IMAGE BECAUSE KERNEL ALSO UPDATES THE BOOT
    // IMAGE, THEREFORE IF USER WANTS TO CHANGE BOTH KERNEL PARAMS AND BOOT IMAGE, THEN, CHANGING
    //
    // THE BOOT IMAGE LATER WILL MAKE SURE WE PUT IN PLACE THE RIGHT BOOT IMAGE
    // Update kernel parameters
    if let Some(new_kernel_parameters) = new_kernel_parameters_opt {
        // Update boot params
        current_node_boot_param_vec
            .iter_mut()
            .for_each(|boot_parameter| {
                log::info!(
                    "Updating '{:?}' kernel parameters to '{}'",
                    boot_parameter.hosts,
                    new_kernel_parameters
                );

                need_restart =
                    need_restart || boot_parameter.apply_kernel_params(&new_kernel_parameters);
                log::info!("need restart? {}", need_restart);
                let _ = boot_parameter.update_boot_image(&boot_parameter.get_boot_image());
            });
    }

    log::debug!("new kernel params: {:#?}", current_node_boot_param_vec);

    // IMPORTANT: ALWAYS SET KERNEL PARAMS BEFORE BOOT IMAGE BECAUSE KERNEL ALSO UPDATES THE BOOT
    // IMAGE, THEREFORE IF USER WANTS TO CHANGE BOTH KERNEL PARAMS AND BOOT IMAGE, THEN, CHANGING
    //
    // Update boot image
    //
    // Check if boot image changes and notify the user and update the node boot params struct
    if let Some(new_boot_image_id) = new_boot_image_id_opt {
        let boot_params_to_update_vec: Vec<&BootParameters> = current_node_boot_param_vec
            .iter()
            .filter(|boot_param| boot_param.get_boot_image() != new_boot_image_id)
            .collect();

        if !boot_params_to_update_vec.is_empty() {
            // Update boot params
            current_node_boot_param_vec
                .iter_mut()
                .for_each(|boot_parameter| {
                    log::info!(
                        "Updating '{:?}' boot image to '{}'",
                        boot_parameter.hosts,
                        new_boot_image_id
                    );

                    let _ = boot_parameter.update_boot_image(&new_boot_image_id);
                });

            need_restart = true;
        }
    } else {
        /* need_restart = false;
        log::info!("Boot image not defined. No need to reboot."); */
    }

    log::info!(
        "boot params to update vec:\n{:#?}",
        current_node_boot_param_vec
    );

    if !assume_yes {
        if need_restart {
            if Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt(format!(
                    "This operation will reboot the nodes below:\n{:?}\nDo you want to continue?",
                    xname_vec
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
    }

    if dry_run {
        println!("Dry-run enabled. No changes persisted into the system");
    } else {
        log::info!("Persist changes");

        // Update boot params
        for boot_parameter in current_node_boot_param_vec {
            let component_patch_rep = bss::http_client::patch(
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

        // Update desired configuration
        if let Some(desired_configuration_name) = new_runtime_configuration_opt {
            println!(
                "Updating runtime configuration to '{}'",
                desired_configuration_name
            );

            cfs::component::utils::update_component_list_desired_configuration(
                shasta_token,
                shasta_base_url,
                shasta_root_cert,
                xname_vec.iter().map(|xname| xname.to_string()).collect(), // TODO: modify function signature
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

            let nodes: Vec<String> = xname_vec
                .into_iter()
                .map(|xname| xname.to_string())
                .collect();

            power_reset_nodes::exec(
                shasta_token,
                shasta_base_url,
                shasta_root_cert,
                &nodes.join(","),
                false,
                true,
                assume_yes,
                "table",
            )
            .await
        }
    }
}
