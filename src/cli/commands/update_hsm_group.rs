use clap::ArgMatches;

use dialoguer::{theme::ColorfulTheme, Confirm};
use mesa::shasta::{capmc, hsm, ims};

use crate::common::ims_ops::get_image_id_from_cfs_configuration_name;

/// Updates boot params and desired configuration for all nodes that belongs to a HSM group
pub async fn exec(
    shasta_token: &str,
    shasta_base_url: &str,
    cli_update_hsm: &ArgMatches,
    hsm_group: Option<&String>,
) {
    // Get boot image configuration
    let boot_image_configuration_opt = cli_update_hsm.get_one::<String>("boot-image");

    // Check desired configuration exists
    let desired_configuration_opt = cli_update_hsm.get_one::<String>("desired-configuration");

    let desired_configuration_name = if let Ok(desired_configuration_detail_list) =
        mesa::shasta::cfs::configuration::http_client::get(
            shasta_token,
            shasta_base_url,
            desired_configuration_opt,
            Some(&1),
        )
        .await
    {
        log::debug!(
            "CFS configuration resp:\n{:#?}",
            desired_configuration_detail_list
        );

        desired_configuration_detail_list.first().unwrap()["name"]
            .as_str()
            .unwrap()
            .to_string()
    } else {
        eprintln!(
            "Desired configuration {} does not exists. Exit",
            desired_configuration_opt.unwrap()
        );
        std::process::exit(1);
    };

    let hsm_group_name = match hsm_group {
        None => cli_update_hsm.get_one("HSM_GROUP").unwrap(),
        Some(hsm_group_value) => hsm_group_value,
    };

    // Get nodes members of HSM group
    // Get HSM group details
    let hsm_group_details =
        hsm::http_client::get_hsm_group(shasta_token, shasta_base_url, hsm_group_name).await;

    log::debug!("HSM group response:\n{:#?}", hsm_group_details);

    // Get list of xnames in HSM group
    let nodes: Vec<String> = hsm_group_details.unwrap()["members"]["ids"]
        .as_array()
        .unwrap()
        .iter()
        .map(|node| node.as_str().unwrap().to_string())
        .collect();

    // if boot_image_configuration_opt.is_some() {
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
    // }

    /* // Get boot-image configuration name
    let boot_image_cfs_configuration_name = cli_update_hsm
        .get_one::<String>("boot-image")
        .unwrap()
        .to_string();

    log::info!(
        "Looking for image ID related to CFS configuration {}",
        boot_image_cfs_configuration_name
    );

    // Get desired-configuration CFS configurantion name
    let desired_configuration_cfs_configuration_name = cli_update_hsm
        .get_one::<String>("desired-configuration")
        .unwrap()
        .to_string();

    let boot_image_id_opt = Some(
        get_image_id_from_cfs_configuration_name(
            shasta_token,
            shasta_base_url,
            boot_image_cfs_configuration_name.clone(),
        )
        .await,
    );

    let image_details = if let Some(boot_image_id) = boot_image_id_opt {
        if let Ok(image_details) =
            ims::image::http_client::get(shasta_token, shasta_base_url, Some(&boot_image_id)).await
        {
            Some(image_details)
        } else {
            eprintln!("No image details found for image ID {}", boot_image_id);
            std::process::exit(1);
        }
    } else {
        eprintln!(
            "No image found related to CFS configuration {}. Exit",
            boot_image_cfs_configuration_name
        );
        std::process::exit(1);
    };

    log::debug!("image_details:\n{:#?}", image_details);

    let ims_image_name = image_details.as_ref().unwrap()["name"]
        .as_str()
        .unwrap()
        .to_string();
    let ims_image_etag = image_details.as_ref().unwrap()["link"]["etag"]
        .as_str()
        .unwrap()
        .to_string();
    let ims_image_path = image_details.as_ref().unwrap()["link"]["path"]
        .as_str()
        .unwrap()
        .to_string();
    let ims_image_type = image_details.as_ref().unwrap()["link"]["type"]
        .as_str()
        .unwrap()
        .to_string(); */

    // Process boot parameters
    if let Some(boot_image_cfs_configuration_name) = boot_image_configuration_opt {
        let image_id = get_image_id_from_cfs_configuration_name(
            shasta_token,
            shasta_base_url,
            boot_image_cfs_configuration_name.clone(),
        )
        .await;

        let image_details_resp =
            ims::image::http_client::get(shasta_token, shasta_base_url, Some(&image_id)).await;

        log::debug!("image_details:\n{:#?}", image_details_resp);

        let image_path = Some(
            image_details_resp.as_ref().unwrap()["link"]["path"]
                .as_str()
                .unwrap()
                .to_string(),
        );

        let image_id = image_path
            .unwrap()
            .strip_prefix("s3://boot-images/")
            .unwrap()
            .strip_suffix("/manifest.json")
            .unwrap()
            .to_string();

        /* println!("image id: {}", image_id);
        println!("kernel id: s3://boot-images/{}/kernel", image_id);
        println!("initrd: s3://boot-images/{}/initrd", image_id);
        println!("console=ttyS0,115200 bad_page=panic crashkernel=360M hugepagelist=2m-2g intel_iommu=off intel_pstate=disable iommu.passthrough=on numa_interleave_omit=headless oops=panic pageblock_order=14 rd.neednet=1 rd.retry=10 rd.shell ip=dhcp quiet ksocklnd.skip_mr_route_setup=1 cxi_core.disable_default_svc=0 cxi_core.enable_fgfc=1 cxi_core.disable_default_svc=0 cxi_core.sct_pid_mask=0xf spire_join_token=${{SPIRE_JOIN_TOKEN}} root=craycps-s3:s3://boot-images/{}/rootfs:37df9a2dc2c4b50679def2193c193c40-230:dvs:api-gw-service-nmn.local:300:nmn0", image_id); */

        let component_patch_rep = mesa::shasta::bss::http_client::put(
            shasta_base_url,
            shasta_token,
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

    /* // Create BOS sessiontemplate

    let bos_session_template_name = desired_configuration_cfs_configuration_name.clone();

    let create_bos_session_template_payload = bos::template::BosTemplate::new_for_hsm_group(
        desired_configuration_cfs_configuration_name,
        bos_session_template_name,
        ims_image_name,
        ims_image_path,
        ims_image_type,
        ims_image_etag,
        hsm_group_name,
    );

    let create_bos_session_template_resp = bos::template::http_client::post(
        shasta_token,
        shasta_base_url,
        &create_bos_session_template_payload,
    )
    .await;

    log::debug!(
        "Create BOS session template response:\n{:#?}",
        create_bos_session_template_resp
    );

    if create_bos_session_template_resp.is_err() {
        eprintln!("BOS session template creation failed");
        std::process::exit(1);
    }

    log::debug!(
        "create_bos_session_template_resp:\n{:#?}",
        create_bos_session_template_resp
    );

    log::info!(
        "BOS sessiontemplate created: {}",
        create_bos_session_template_resp.unwrap()
    );

    log::info!("Rebooting nodes {:?}", nodes);
    log::debug!("Rebooting nodes {:?} through CAPMC", nodes);

    // Create CAPMC operation shutdown
    let capmc_shutdown_nodes_resp = capmc::http_client::node_power_off::post_sync(
        shasta_token,
        shasta_base_url,
        nodes.clone(),
        Some("Update node boot params".to_string()),
        true,
    )
    .await;

    log::debug!(
        "CAPMC shutdown nodes response:\n{:#?}",
        capmc_shutdown_nodes_resp
    );

    // Create BOS session operation start
    let create_bos_boot_session_resp = bos::session::http_client::post(
        shasta_token,
        shasta_base_url,
        &create_bos_session_template_payload.name,
        "boot",
        Some(&nodes.join(",")),
    )
    .await;

    log::debug!(
        "Create BOS boot session response:\n{:#?}",
        create_bos_boot_session_resp
    );

    if create_bos_boot_session_resp.is_err() {
        eprintln!("Error creating BOS boot session. Exit");
        std::process::exit(1);
    } */

    // Update desired configuration

    /* if let Some(desired_configuration_cfs_configuration_name) =
        cli_update_hsm.get_one::<String>("desired-configuration")
    { */
    mesa::shasta::cfs::component::utils::update_component_list_desired_configuration(
        shasta_token,
        shasta_base_url,
        nodes.clone(),
        // for this field so it accepts
        // Vec<&str> instead of
        // Vec<String>
        &desired_configuration_name,
        false,
    )
    .await;
    // }

    // Check if need to reboot
    if boot_image_configuration_opt.is_some() || desired_configuration_opt.is_some() {
        // Create BOS session. Note: reboot operation shuts down the nodes and don't bring them back
        // up... hence we will split the reboot into 2 operations shutdown and start

        // Create CAPMC operation shutdown
        let capmc_shutdown_nodes_resp = capmc::http_client::node_power_off::post_sync(
            shasta_token,
            shasta_base_url,
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
            nodes,
            Some("Update node boot params and/or desired configuration".to_string()),
            false,
        )
        .await;

        log::debug!(
            "CAPMC starting nodes response:\n{:#?}",
            capmc_start_nodes_resp
        );
    }
}
