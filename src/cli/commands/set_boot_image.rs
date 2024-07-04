use dialoguer::{theme::ColorfulTheme, Confirm};
use mesa::{
    bss::{self, bootparameters::BootParameters},
    common::jwt_ops::get_claims_from_jwt_token,
    error::Error,
};

/// Set boot image to a set of nodes. This function updates the desired_configuration for the node
/// boot params.
/// If the new image is different than existing one, then the nodes will reboot. This is mandatory
/// to keep CSM data as a true source of truth
pub async fn exec(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    image_id: &str,
    hsm_group_name_opt: Option<&Vec<String>>,
    xname_vec_opt: Option<&Vec<String>>,
) -> Result<(), Error> {
    println!("Set runtime-configuration");

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

    let any_boot_image_change = current_node_boot_params
        .iter()
        .any(|boot_param| boot_param.get_boot_image() != image_id);

    if any_boot_image_change {
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
            if boot_parameter.get_boot_image().eq(image_id) {
                boot_parameter.set_boot_image(&image_id);
                println!(
                    "Updating '{:?}' boot image to '{}'",
                    boot_parameter.hosts, image_id
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
    } else {
        println!("Boot image did not change. No need to reboot.");
    }

    // Audit
    let jwt_claims = get_claims_from_jwt_token(shasta_token).unwrap();

    log::info!(target: "app::audit", "User: {} ({}) ; Operation: Apply nodes on {:?}", jwt_claims["name"].as_str().unwrap(), jwt_claims["preferred_username"].as_str().unwrap(), xname_vec_opt.unwrap());

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
