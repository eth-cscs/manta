use dialoguer::theme::ColorfulTheme;
use mesa::{
    bss::{self, bootparameters::BootParameters},
    common::jwt_ops::get_claims_from_jwt_token,
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
) -> Result<(), Error> {
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
            "Setting runtime configuration without a list of nodes".to_string(),
        ));
    };

    // Get current node boot params
    let current_node_boot_params: Vec<BootParameters> = bss::bootparameters::http_client::get(
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
        .with_prompt("This operation will reboot the nodes. Please confirm to proceed")
        .interact()
        .unwrap();

    if !proceed {
        println!("Operation canceled by the user. Exit");
        std::process::exit(1);
    }

    // Update kernel parameters
    for mut boot_parameter in current_node_boot_params {
        if boot_parameter.kernel.eq(kernel_params) {
            boot_parameter.kernel = kernel_params.to_string();

            println!(
                "Updating '{:?}' kernel parameters to '{}'",
                boot_parameter.hosts, kernel_params
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

            xname_to_reboot_vec.push(boot_parameter.hosts.first().unwrap().to_string());
        }
    }

    // Audit
    let jwt_claims = get_claims_from_jwt_token(shasta_token).unwrap();

    log::info!(target: "app::audit", "User: {} ({}) ; Operation: Apply nodes on {:?}", jwt_claims["name"].as_str().unwrap(), jwt_claims["preferred_username"].as_str().unwrap(), xnames);

    // Reboot if needed
    if xname_to_reboot_vec.is_empty() {
        println!("Nothing to change. Exit");
    } else {
        let _ = mesa::capmc::http_client::node_power_reset::post_sync_vec(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            xname_to_reboot_vec,
            Some("Update kernel parameters".to_string()),
            true,
        )
        .await;
    }

    Ok(())
}
