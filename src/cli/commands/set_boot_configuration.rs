use dialoguer::{theme::ColorfulTheme, Confirm};
use mesa::{
    bss::{self, bootparameters::BootParameters},
    cfs,
    common::jwt_ops,
    error::Error,
};

use crate::common::ims_ops::get_image_id_from_cfs_configuration_name;

/// Updates the boot image for a set of nodes
/// reboots the nodes which boot image have changed
pub async fn exec(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    configuration_name: &str,
    hsm_group_name_opt: Option<&Vec<String>>,
    xname_vec_opt: Option<&Vec<String>>,
) -> Result<(), Error> {
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

    // Check new configuration exists and exit otherwise
    // Get configuration detail from CSM
    let configuration_detail_list_rslt = cfs::configuration::mesa::http_client::get(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        Some(configuration_name),
    )
    .await;

    // Exit if configuration does not exists
    if configuration_detail_list_rslt.is_err() || configuration_detail_list_rslt.unwrap().is_empty()
    {
        return Err(Error::Message(format!(
            "Configuration '{}' does not exists. Exit",
            configuration_name
        )));
    }

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

    // Get new image id from configuration name
    let image_id_opt = get_image_id_from_cfs_configuration_name(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        configuration_name.to_string(),
    )
    .await;

    // Get image id or exit if configuration does not exists
    let image_id = if let Some(image_id) = image_id_opt {
        image_id
    } else {
        return Err(Error::Message(format!(
            "Image related to configuration '{}' cound not be found.",
            configuration_name,
        )));
    };

    // Check if new image is different than the current one. This will help to know if need to
    // reboot
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
            if boot_parameter.get_boot_image().eq(&image_id) {
                boot_parameter.update_boot_image(&image_id);

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
    log::info!(target: "app::audit", "User: {} ({}) ; Operation: Set boot configuration to {:?}", jwt_ops::get_name(shasta_token).unwrap(), jwt_ops::get_preferred_username(shasta_token).unwrap(), xnames);

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
