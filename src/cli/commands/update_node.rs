use crate::common::ims_ops::get_image_id_from_cfs_configuration_name;

use dialoguer::{theme::ColorfulTheme, Confirm};
use mesa::{capmc, cfs, node::utils::validate_xnames};

pub async fn exec(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    hsm_group_name: Option<&String>,
    boot_image_configuration_opt: Option<&String>,
    desired_configuration_opt: Option<&String>,
    xnames: Vec<&str>,
) {
    let mut need_restart = false;

    let desired_configuration_detail_list_rslt = cfs::configuration::mesa::http_client::get(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        desired_configuration_opt.map(|elem| elem.as_str()),
    )
    .await;

    // Check desired configuration exists
    if desired_configuration_detail_list_rslt.is_ok()
        && !desired_configuration_detail_list_rslt
            .as_ref()
            .unwrap()
            .is_empty()
    {
        let desired_configuration_detail_list = desired_configuration_detail_list_rslt.unwrap();

        log::debug!(
            "CFS configuration resp:\n{:#?}",
            desired_configuration_detail_list
        );

        let desired_configuration_name = desired_configuration_detail_list
            .first()
            .unwrap()
            .name
            .clone();

        log::info!(
            "Desired configuration '{}' exists",
            desired_configuration_name
        );
    } else {
        eprintln!(
            "Desired configuration '{}' does not exists. Exit",
            desired_configuration_opt.unwrap()
        );
        std::process::exit(1);
    };

    log::info!(
        "Desired configuration '{}' exists",
        desired_configuration_opt.unwrap()
    );

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

    // Get new image id
    let (new_image_id_opt, node_boot_params_opt) =
        if let Some(boot_image_cfs_configuration_name) = boot_image_configuration_opt {
            // Get image id related to the boot CFS configuration
            let boot_image_id_opt = get_image_id_from_cfs_configuration_name(
                shasta_token,
                shasta_base_url,
                shasta_root_cert,
                boot_image_cfs_configuration_name.clone(),
            )
            .await;

            // Check image artifact exists
            let boot_image_value_vec = if let Some(boot_image_id) = boot_image_id_opt {
                mesa::ims::image::shasta::http_client::get(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    Some(&boot_image_id),
                )
                .await
                .unwrap()
            } else {
                eprintln!(
                    "Image ID related to CFS configuration name {} not found. Exit",
                    boot_image_cfs_configuration_name
                );
                std::process::exit(1);
            };

            log::info!(
                "Boot image found with id '{}'",
                boot_image_value_vec.first().unwrap()["id"]
                    .as_str()
                    .unwrap()
            );

            log::debug!("image_details_value_vec:\n{:#?}", boot_image_value_vec);

            let image_path = boot_image_value_vec.first().unwrap()["link"]["path"]
                .as_str()
                .unwrap()
                .to_string();

            let new_image_id = image_path
                .strip_prefix("s3://boot-images/")
                .unwrap()
                .strip_suffix("/manifest.json")
                .unwrap()
                .to_string();

            // Check if need to reboot. We will reboot if the new boot image is different than the current
            // one
            // Get node boot params
            let node_boot_params = mesa::bss::http_client::get_boot_params(
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
            let current_image_id = node_boot_params["kernel"]
                .as_str()
                .unwrap()
                .trim_start_matches("s3://boot-images/")
                .trim_end_matches("/kernel");

            // Check if new image id is different to the current one to find out if need to restart
            if current_image_id != new_image_id {
                need_restart = true;
            } else {
                println!("Boot image does not change. No need to reboot.");
            }

            (Some(new_image_id), Some(node_boot_params))
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
            boot_image_configuration_opt.unwrap()
        );

        // Update root kernel param to it uses the new image id
        let new_kernel_params_vec: Vec<String> = node_boot_params_opt.unwrap()["params"]
            .as_str()
            .unwrap()
            .split_whitespace()
            .map(|boot_param| {
                if boot_param.contains("root=") {
                    let aux = boot_param
                        .trim_start_matches("root=craycps-s3:s3://boot-images/")
                        .split_once('/')
                        .unwrap()
                        .1;

                    format!(
                        "root=craycps-s3:s3://boot-images/{}/{}",
                        new_image_id_opt.clone().unwrap(),
                        aux
                    )
                } else {
                    boot_param.to_string()
                }
            })
            .collect();

        let component_patch_rep = mesa::bss::http_client::patch(
            shasta_base_url,
            shasta_token,
            shasta_root_cert,
            &xnames
                .iter()
                .map(|xname| xname.to_string())
                .collect::<Vec<String>>(),
            Some(&new_kernel_params_vec.join(" ")),
            // Some(&format!("console=ttyS0,115200 bad_page=panic crashkernel=360M hugepagelist=2m-2g intel_iommu=off intel_pstate=disable iommu.passthrough=on numa_interleave_omit=headless oops=panic pageblock_order=14 rd.neednet=1 rd.retry=10 rd.shell ip=dhcp quiet ksocklnd.skip_mr_route_setup=1 cxi_core.disable_default_svc=0 cxi_core.enable_fgfc=1 cxi_core.disable_default_svc=0 cxi_core.sct_pid_mask=0xf spire_join_token=${{SPIRE_JOIN_TOKEN}} root=craycps-s3:s3://boot-images/{}/rootfs:37df9a2dc2c4b50679def2193c193c40-230:dvs:api-gw-service-nmn.local:300:nmn0", image_id)),
            Some(&format!(
                "s3://boot-images/{}/kernel",
                new_image_id_opt.clone().unwrap()
            )),
            Some(&format!(
                "s3://boot-images/{}/initrd",
                new_image_id_opt.unwrap()
            )),
        )
        .await;

        log::debug!(
            "Component boot parameters resp:\n{:#?}",
            component_patch_rep
        );

        log::info!("Boot params for nodes {:?} updated", xnames);

        // Update desired configuration

        if let Some(desired_configuration_name) = desired_configuration_opt {
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
}
