use backend_dispatcher::{interfaces::bss::BootParametersTrait, types};
use dialoguer::theme::ColorfulTheme;
use mesa::{common::jwt_ops, error::Error};

use crate::backend_dispatcher::StaticBackendDispatcher;

/// Updates the kernel parameters for a set of nodes
/// reboots the nodes which kernel params have changed
pub async fn exec(
    backend: StaticBackendDispatcher,
    shasta_token: &str,
    kernel_params: &str,
    xname_vec: Vec<String>,
    assume_yes: bool,
) -> Result<(), Error> {
    let mut need_restart = false;
    println!("Delete kernel parameters");

    let mut xname_to_reboot_vec: Vec<String> = Vec::new();

    /* let xnames = if let Some(hsm_group_name_vec) = hsm_group_name_opt {
        hsm::group::utils::get_member_vec_from_hsm_name_vec(
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
            "ERROR - Deleting kernel parameters without a list of nodes".to_string(),
        ));
    };

    // Get current node boot params
    let current_node_boot_params_vec: Vec<BootParameters> = bss::http_client::get(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        &xnames
            .iter()
            .map(|xname| xname.to_string())
            .collect::<Vec<String>>(),
    )
    .await
    .unwrap(); */

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

    /* // Delete kernel parameters
    current_node_boot_params_vec
        .iter_mut()
        .for_each(|boot_parameter| {
            log::info!(
                "Deleting '{:?}' kernel parameters to '{}'",
                boot_parameter.hosts,
                kernel_params
            );

            need_restart = need_restart || boot_parameter.delete_kernel_param(&kernel_params);
            log::info!("need restart? {}", need_restart);
            // NOTE: No need to touch 'kernel' and 'initrd' values
            // let _ = boot_parameter.delete_boot_image(&boot_parameter.get_boot_image());
        }); */

    log::debug!(
        "kernel params to delete: {:#?}",
        current_node_boot_params_vec
    );

    for mut boot_parameter in current_node_boot_params_vec {
        log::info!(
            "Deleting '{:?}' kernel parameters to '{}'",
            boot_parameter.hosts,
            kernel_params
        );

        need_restart = need_restart || boot_parameter.delete_kernel_params(&kernel_params);
        log::info!("need restart? {}", need_restart);

        if need_restart {
            /* let _ = mesa::bss::http_client::patch(
                shasta_base_url,
                shasta_token,
                shasta_root_cert,
                &boot_parameter,
            )
            .await; */

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
    log::info!(target: "app::audit", "User: {} ({}) ; Operation: Delete kernel parameters to {:?}", jwt_ops::get_name(shasta_token).unwrap_or("".to_string()), jwt_ops::get_preferred_username(shasta_token).unwrap_or("".to_string()), xname_vec);

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
        )
        .await;
    }

    Ok(())
}
