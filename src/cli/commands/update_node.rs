use crate::common::{ims_ops::get_image_id_from_cfs_configuration_name, node_ops};

use clap::ArgMatches;
use mesa::shasta::{bos, capmc, ims};

pub async fn exec(
    shasta_token: &str,
    shasta_base_url: &str,
    hsm_group_name: Option<&String>,
    cli_update_hsm: &ArgMatches,
    xnames: Vec<&str>,
) {
    // Get boot-image configuration name
    let boot_image_cfs_configuration_name = cli_update_hsm
        .get_one::<String>("boot-image")
        .unwrap()
        .to_string();

    // Get dessired-configuration CFS configurantion name
    let dessired_configuration_cfs_configuration_name = cli_update_hsm
        .get_one::<String>("dessired-configuration")
        .unwrap()
        .to_string();

    // Check user has provided valid XNAMES
    if hsm_group_name.is_some() {
        if !node_ops::validate_xnames(shasta_token, shasta_base_url, &xnames, hsm_group_name)
            .await
        {
            eprintln!("xname/s invalid. Exit");
            std::process::exit(1);
        }
    }

    let image_id = get_image_id_from_cfs_configuration_name(
        shasta_token,
        shasta_base_url,
        boot_image_cfs_configuration_name.clone(),
    )
    .await;

    let image_details_resp =
        ims::image::http_client::get(shasta_token, shasta_base_url, &image_id).await;

    log::debug!("image_details:\n{:#?}", image_details_resp);

    let ims_image_name = image_details_resp.as_ref().unwrap()["name"]
        .as_str()
        .unwrap()
        .to_string();
    let ims_image_etag = image_details_resp.as_ref().unwrap()["link"]["etag"]
        .as_str()
        .unwrap()
        .to_string();
    let ims_image_path = image_details_resp.as_ref().unwrap()["link"]["path"]
        .as_str()
        .unwrap()
        .to_string();
    let ims_image_type = image_details_resp.as_ref().unwrap()["link"]["type"]
        .as_str()
        .unwrap()
        .to_string();

    // Create BOS sessiontemplate

    let bos_session_template_name = boot_image_cfs_configuration_name.clone();

    let create_bos_session_template_payload = bos::template::BosTemplate::new_for_node_list(
        dessired_configuration_cfs_configuration_name,
        bos_session_template_name,
        ims_image_name,
        ims_image_path,
        ims_image_type,
        ims_image_etag,
        xnames.iter().map(|xname| xname.to_string()).collect(),
    );

    log::debug!(
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
    );

    // Create BOS session. Note: reboot operation shuts down the nodes and don't bring them back
    // up... hence we will split the reboot into 2 operations shutdown and start

    let nodes: Vec<String> = xnames.into_iter().map(|xname| xname.to_string()).collect();

    if let Some(true) = cli_update_hsm.get_one::<bool>("reboot") {
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
        }
    }
}
