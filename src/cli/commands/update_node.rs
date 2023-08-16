use crate::common::{ims_ops::get_image_id_from_cfs_configuration_name, node_ops};

use clap::ArgMatches;
use dialoguer::{theme::ColorfulTheme, Confirm};
use mesa::shasta::{capmc, ims};

pub async fn exec(
    shasta_token: &str,
    shasta_base_url: &str,
    hsm_group_name: Option<&String>,
    cli_update_hsm: &ArgMatches,
    xnames: Vec<&str>,
) {
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
        log::debug!("CFS configuration resp:\n{:#?}", desired_configuration_detail_list);

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

    // Check user has provided valid XNAMES
    if hsm_group_name.is_some() {
        if !node_ops::validate_xnames(shasta_token, shasta_base_url, &xnames, hsm_group_name).await
        {
            eprintln!("xname/s invalid. Exit");
            std::process::exit(1);
        }
    }

    if Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(format!(
            "This operation will reboot nodes {:?}. Do you want to continue?",
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

    // Get boot-image configuration name
    // let boot_image_cfs_configuration_name = cli_update_hsm.get_one::<String>("boot-image");

    // Process boot parameters
    if let Some(boot_image_cfs_configuration_name) = cli_update_hsm.get_one::<String>("boot-image")
    {
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

        let component_patch_rep = mesa::shasta::bss::http_client::patch(
            shasta_base_url,
            shasta_token,
            &xnames.iter().map(|&xname| xname.to_string()).collect(),
            Some(&format!("console=ttyS0,115200 bad_page=panic crashkernel=360M hugepagelist=2m-2g intel_iommu=off intel_pstate=disable iommu.passthrough=on numa_interleave_omit=headless oops=panic pageblock_order=14 rd.neednet=1 rd.retry=10 rd.shell ip=dhcp quiet ksocklnd.skip_mr_route_setup=1 cxi_core.disable_default_svc=0 cxi_core.enable_fgfc=1 cxi_core.disable_default_svc=0 cxi_core.sct_pid_mask=0xf spire_join_token=${{SPIRE_JOIN_TOKEN}} root=craycps-s3:s3://boot-images/{}/rootfs:37df9a2dc2c4b50679def2193c193c40-230:dvs:api-gw-service-nmn.local:300:nmn0", image_id)),
            Some(&format!("s3://boot-images/{}/kernel", image_id)),
            Some(&format!("s3://boot-images/{}/initrd", image_id)),
        )
        .await;

        log::debug!(
            "Component boot parameters resp:\n{:#?}",
            component_patch_rep
        );
    }

    /* let ims_image_name;
    let ims_image_etag;
    let ims_image_path;
    let ims_image_type;

    if let Some(boot_image_cfs_configuration_name) = cli_update_hsm.get_one::<String>("boot-image")
    {
        let image_id = get_image_id_from_cfs_configuration_name(
            shasta_token,
            shasta_base_url,
            boot_image_cfs_configuration_name.clone(),
        )
        .await;

        let image_details_resp =
            ims::image::http_client::get(shasta_token, shasta_base_url, Some(&image_id)).await;

        log::debug!("image_details:\n{:#?}", image_details_resp);

        ims_image_name = Some(
            image_details_resp.as_ref().unwrap()["name"]
                .as_str()
                .unwrap()
                .to_string(),
        );
        ims_image_etag = Some(
            image_details_resp.as_ref().unwrap()["link"]["etag"]
                .as_str()
                .unwrap()
                .to_string(),
        );
        ims_image_path = Some(
            image_details_resp.as_ref().unwrap()["link"]["path"]
                .as_str()
                .unwrap()
                .to_string(),
        );
        ims_image_type = Some(
            image_details_resp.as_ref().unwrap()["link"]["type"]
                .as_str()
                .unwrap()
                .to_string(),
        );
    } else {
        ims_image_name = None;
        ims_image_etag = None;
        ims_image_path = None;
        ims_image_type = None;
    } */

    // Update desired configuration

    mesa::shasta::cfs::component::utils::update_component_list_desired_configuration(
        shasta_token,
        shasta_base_url,
        xnames.iter().map(|xname| xname.to_string()).collect(), // TODO: modify function signature
        // for this field so it accepts
        // Vec<&str> instead of
        // Vec<String>
        &desired_configuration_name,
    )
    .await;

    /* // Create BOS sessiontemplate

    // Get desired-configuration CFS configurantion name
    let desired_configuration_cfs_configuration_name =
        cli_update_hsm.get_one::<String>("desired-configuration");

    let bos_session_template_name = match boot_image_cfs_configuration_name {
        Some(_) => boot_image_cfs_configuration_name.unwrap(),
        None => desired_configuration_cfs_configuration_name.unwrap(),
    };

    let create_bos_session_template_payload = bos::template::BosTemplate::new_for_node_list(
        bos_session_template_name.to_string(),
        desired_configuration_cfs_configuration_name.cloned(),
        ims_image_name,
        ims_image_path,
        ims_image_type,
        ims_image_etag,
        Some(xnames.iter().map(|xname| xname.to_string()).collect()),
    ); */

    if cli_update_hsm.get_one::<String>("boot-image").is_some()
        || cli_update_hsm
            .get_one::<String>("desired-configuration")
            .is_some()
    {
        /* log::debug!(
            "create_bos_session_template_payload:\n{:#?}",
            create_bos_session_template_payload
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
            "create_bos_session_template_resp:
        \n{:#?}",
            create_bos_session_template_resp
        );

        log::info!(
            "BOS sessiontemplate created: {}",
            create_bos_session_template_resp.unwrap()
        ); */

        // Create BOS session. Note: reboot operation shuts down the nodes and don't bring them back
        // up... hence we will split the reboot into 2 operations shutdown and start

        let nodes: Vec<String> = xnames.into_iter().map(|xname| xname.to_string()).collect();

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

        /* // Create BOS session operation start
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
    }
}
