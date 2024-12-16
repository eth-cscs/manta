use backend_dispatcher::{contracts::BackendTrait, types};
use dialoguer::theme::ColorfulTheme;
use mesa::{common::jwt_ops, error::Error};

use crate::backend_dispatcher::StaticBackendDispatcher;

/// Updates the kernel parameters for a set of nodes
/// reboots the nodes which kernel params have changed
pub async fn exec(
    backend: StaticBackendDispatcher,
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    kernel_params: &str,
    xname_vec: Vec<String>,
    assume_yes: bool,
) -> Result<(), Error> {
    let mut need_restart = false;
    log::info!("Add kernel parameters");

    let mut xname_to_reboot_vec: Vec<String> = Vec::new();

    /* let xnames = if let Some(hsm_group_name_vec) = hsm_group_name_opt {
        let xname_vec_rslt = backend
            .get_member_vec_from_hsm_name_vec(shasta_token, hsm_group_name_vec.clone())
            .await;

        println!("DEBUG - hsm members:\n{:#?}", xname_vec_rslt);

        if xname_vec_rslt.is_err() {
            return Err(Error::Message(
                "ERROR - Error finding list of nodes".to_string(),
            ));
        }

        xname_vec_rslt.unwrap()
    } else if let Some(xname_vec) = xname_vec_opt {
        xname_vec.clone()
    } else {
        return Err(Error::Message(
            "ERROR - Deleting kernel parameters without a list of nodes".to_string(),
        ));
    }; */

    // Get current node boot params
    /* let current_node_boot_params_vec: Vec<BootParameters> = bss::http_client::get(
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
            "Add '{:?}' kernel parameters to '{}'",
            boot_parameter.hosts,
            kernel_params
        );

        need_restart = need_restart || boot_parameter.add_kernel_params(&kernel_params);
        log::info!("need restart? {}", need_restart);

        if need_restart {
            /* let _ = bss::http_client::patch(
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

            need_restart |= need_restart;

            if need_restart {
                xname_to_reboot_vec.push(boot_parameter.hosts.first().unwrap().to_string());
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
            shasta_base_url,
            shasta_root_cert,
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
