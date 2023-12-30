use dialoguer::{theme::ColorfulTheme, Confirm};
use mesa::{capmc, cfs, hsm};

use crate::common::ims_ops::get_image_id_from_cfs_configuration_name;

/// Updates boot params and desired configuration for all nodes that belongs to a HSM group
/// If boot params defined, then nodes in HSM group will be rebooted
pub async fn exec(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    boot_image_configuration_opt: Option<&String>,
    desired_configuration_opt: Option<&String>,
    hsm_group_name: &String,
) {
    let need_restart = boot_image_configuration_opt.is_some();

    let desired_configuration_detail_list_rslt = cfs::configuration::mesa::http_client::get(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        desired_configuration_opt,
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

        let _ = desired_configuration_detail_list
            .first()
            .unwrap()
            .name
            .clone();
    } else {
        eprintln!(
            "Desired configuration {} does not exists. Exit",
            desired_configuration_opt.unwrap()
        );
        std::process::exit(1);
    };

    // Get nodes members of HSM group
    // Get HSM group details
    let hsm_group_details = hsm::group::shasta::http_client::get(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        Some(hsm_group_name),
    )
    .await;

    log::debug!("HSM group response:\n{:#?}", hsm_group_details);

    // Get list of xnames in HSM group
    let nodes: Vec<String> = hsm_group_details.unwrap().first().unwrap()["members"]["ids"]
        .as_array()
        .unwrap()
        .iter()
        .map(|node| node.as_str().unwrap().to_string())
        .collect();

    if need_restart {
        if Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt(format!(
                "This operation will reboot all nodes in HSM group {}. Do you want to continue?",
                hsm_group_name
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

    // Process boot parameters
    if let Some(boot_image_cfs_configuration_name) = boot_image_configuration_opt {
        let image_id_opt = get_image_id_from_cfs_configuration_name(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            boot_image_cfs_configuration_name.clone(),
        )
        .await;

        let image_value = if let Some(image_id) = image_id_opt {
            mesa::ims::image::shasta::http_client::get(
                shasta_token,
                shasta_base_url,
                shasta_root_cert,
                Some(&image_id),
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

        log::debug!("image_details:\n{:#?}", image_value);

        let image_path = image_value.first().unwrap()["link"]["path"]
            .as_str()
            .unwrap()
            .to_string();

        let image_id = image_path
            .strip_prefix("s3://boot-images/")
            .unwrap()
            .strip_suffix("/manifest.json")
            .unwrap()
            .to_string();

        let component_patch_rep = mesa::bss::http_client::put(
            shasta_base_url,
            shasta_token,
            shasta_root_cert,
            &nodes,
            &format!("console=ttyS0,115200 bad_page=panic crashkernel=360M hugepagelist=2m-2g intel_iommu=off intel_pstate=disable iommu.passthrough=on numa_interleave_omit=headless oops=panic pageblock_order=14 rd.neednet=1 rd.retry=10 rd.shell ip=dhcp quiet ksocklnd.skip_mr_route_setup=1 cxi_core.disable_default_svc=0 cxi_core.enable_fgfc=1 cxi_core.disable_default_svc=0 cxi_core.sct_pid_mask=0xf spire_join_token=${{SPIRE_JOIN_TOKEN}} root=craycps-s3:s3://boot-images/{}/rootfs:37df9a2dc2c4b50679def2193c193c40-230:dvs:api-gw-service-nmn.local:300:nmn0", image_id),
            &format!("s3://boot-images/{}/kernel", image_id),
            &format!("s3://boot-images/{}/initrd", image_id),
        )
        .await;

        log::debug!(
            "Component boot parameters resp:\n{:#?}",
            component_patch_rep
        );
    }

    // Process dessired configuration
    if let Some(desired_configuration_name) = desired_configuration_opt {
        log::info!(
            "Updating desired configuration. Need restart? {}",
            need_restart
        );

        mesa::cfs::component::shasta::utils::update_component_list_desired_configuration(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            nodes.clone(),
            // for this field so it accepts
            // Vec<&str> instead of
            // Vec<String>
            desired_configuration_name,
            !need_restart,
        )
        .await;
    }

    // Check if need to reboot
    if need_restart {
        // Create BOS session. Note: reboot operation shuts down the nodes and don't bring them back
        // up... hence we will split the reboot into 2 operations shutdown and start

        log::info!("Restarting nodes");

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
            need_restart,
        )
        .await;

        log::debug!(
            "CAPMC starting nodes response:\n{:#?}",
            capmc_start_nodes_resp
        );
    }
}
