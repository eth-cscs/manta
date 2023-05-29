use clap::ArgMatches;

use crate::shasta::{bos, capmc, cfs, hsm, ims};

pub async fn exec(
    shasta_token: &str,
    shasta_base_url: &str,
    cli_update_hsm: &ArgMatches,
    hsm_group: Option<&String>,
) {
    let hsm_group_name = match hsm_group {
        None => cli_update_hsm.get_one("HSM_GROUP").unwrap(),
        Some(hsm_group_value) => hsm_group_value,
    };
    // Get configuration name
    let cfs_configuration_name = cli_update_hsm
        .get_one::<String>("CFS_CONFIG")
        .unwrap()
        .to_string();

    // Get most recent CFS session target image for the node
    let mut cfs_sessions_details = cfs::session::http_client::get(
        shasta_token,
        shasta_base_url,
        Some(hsm_group_name),
        None,
        None,
        Some(true),
    )
    .await
    .unwrap();

    cfs_sessions_details.retain(|cfs_session_details| {
        cfs_session_details["target"]["definition"].eq("image")
            && cfs_session_details["configuration"]["name"].eq(&cfs_configuration_name)
    });

    if cfs_sessions_details.is_empty() {
        eprintln!(
            "No CFS session target image found for the CFS configuration name provided. Exit"
        );
        std::process::exit(1);
    }

    log::info!("cfs_sessions_details:\n{:#?}", cfs_sessions_details);

    let result_id = cfs_sessions_details.first().unwrap()["status"]["artifacts"]
        .as_array()
        .unwrap()
        .first()
        .unwrap()["result_id"]
        .as_str()
        .unwrap();

    let image_details =
        ims::image::http_client::get(shasta_token, shasta_base_url, result_id).await;

    log::info!("image_details:\n{:#?}", image_details);

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
        .to_string();

    // Create BOS sessiontemplate

    let bos_session_template_name = cfs_configuration_name.clone();

    let create_bos_session_template_payload = bos::template::BosTemplate::new_for_hsm_group(
        cfs_configuration_name.to_string(),
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

    log::info!(
        "create_bos_session_template_resp:\n{:#?}",
        create_bos_session_template_resp
    );

    println!(
        "BOS sessiontemplate created: {}",
        create_bos_session_template_resp.unwrap()
    );

    // Create BOS session. Note: reboot operation shuts down the nodes and don't bring them back
    // up... hence we will split the reboot into 2 operations shutdown and start

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
