use dialoguer::theme::ColorfulTheme;
use mesa::{
    bss::{self, bootparameters::BootParameters},
    common::jwt_ops,
    error::Error,
};

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
) -> Result<(), Error> {
    let mut need_restart = false;
    println!("Set kernel parameters");

    let mut xname_to_reboot_vec: Vec<String> = Vec::new();

    let xnames = if let Some(hsm_group_name_vec) = hsm_group_name_opt {
        mesa::hsm::group::utils::get_member_vec_from_hsm_name_vec(
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
            "ERROR - Setting kernel parameters without a list of nodes".to_string(),
        ));
    };

    // Get current node boot params
    let mut current_node_boot_params_vec: Vec<BootParameters> =
        bss::bootparameters::http_client::get(
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
        "Set kernel params:\n{:?}\nFor nodes:\n{:?}",
        kernel_params, xnames
    );

    let proceed = dialoguer::Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("This operation will update the kernel parameters for the nodes below. Please confirm to proceed")
        .interact()
        .unwrap();

    if !proceed {
        println!("Operation canceled by the user. Exit");
        std::process::exit(1);
    }

    // Update kernel parameters
    current_node_boot_params_vec
        .iter_mut()
        .for_each(|boot_parameter| {
            log::info!(
                "Updating '{:?}' kernel parameters to '{}'",
                boot_parameter.hosts,
                kernel_params
            );

            need_restart = need_restart || boot_parameter.apply_kernel_params(&kernel_params);
            log::info!("need restart? {}", need_restart);
            let _ = boot_parameter.update_boot_image(&boot_parameter.get_boot_image());
        });

    log::debug!("new kernel params: {:#?}", current_node_boot_params_vec);

    for mut boot_parameter in current_node_boot_params_vec {
        log::info!(
            "Updating '{:?}' kernel parameters to '{}'",
            boot_parameter.hosts,
            kernel_params
        );

        need_restart = need_restart || boot_parameter.apply_kernel_params(&kernel_params);
        log::info!("need restart? {}", need_restart);
        let _ = boot_parameter.update_boot_image(&boot_parameter.get_boot_image());

        if need_restart {
            boot_parameter.params = kernel_params.to_string();

            let _ = mesa::bss::bootparameters::http_client::patch(
                shasta_base_url,
                shasta_token,
                shasta_root_cert,
                &boot_parameter,
            )
            .await;

            need_restart |= need_restart;

            if need_restart {
                xname_to_reboot_vec.push(boot_parameter.hosts.first().unwrap().to_string());
            }
        }
    }

    // Audit
    log::info!(target: "app::audit", "User: {} ({}) ; Operation: Set kernel parameters to {:?}", jwt_ops::get_name(shasta_token).unwrap(), jwt_ops::get_preferred_username(shasta_token).unwrap(), xnames);

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
        )
        .await;
    }

    Ok(())
}
