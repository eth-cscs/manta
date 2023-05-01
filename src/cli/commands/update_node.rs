use clap::ArgMatches;

use crate::{
    common::node_ops,
    shasta::{bos, capmc, cfs, ims},
};

pub async fn exec(
    shasta_token: &str,
    shasta_base_url: &str,
    cli_update_node: &ArgMatches,
    xnames: Vec<&str>,
    cfs_configuration_name: Option<&String>,
    hsm_group: Option<&String>,
) {
    // Get HSM group name
    let hsm_group_name = match hsm_group {
        None => cli_update_node.get_one("HSM_GROUP"),
        Some(_) => hsm_group,
    };

    // Check user has provided valid XNAMES
    if !node_ops::validate_xnames(shasta_token, shasta_base_url, &xnames, hsm_group_name).await {
        eprintln!("xname/s invalid. Exit");
        std::process::exit(1);
    }

    // Get all CFS session ended successfully
    let mut cfs_sessions_details = cfs::session::http_client::get(
        shasta_token,
        shasta_base_url,
        hsm_group_name,
        None,
        None,
        Some(true),
    )
    .await
    .unwrap();

    // Filter CFS sessions by target image
    cfs_sessions_details.retain(|cfs_session_details| {
        cfs_session_details["target"]["definition"].eq("image")
            && cfs_session_details["configuration"]["name"].eq(cfs_configuration_name.unwrap())
    });

    if cfs_sessions_details.is_empty() {
        eprintln!("No CFS session found for the nodes and CFS configuration name provided. Exit");
        std::process::exit(1);
    }

    log::info!("cfs_sessions_details:\n{:#?}", cfs_sessions_details);

    // Get result_id value which is the image id generated from the most recent CFS session
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

    let bos_session_template_name = cfs_configuration_name;

    let create_bos_session_template_payload = bos::template::BosTemplate::new_for_node_list(
        cfs_configuration_name.unwrap().to_string(),
        bos_session_template_name.unwrap().to_string(),
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

    log::info!(
        "create_bos_session_template_resp:
        \n{:#?}",
        create_bos_session_template_resp
    );

    println!(
        "BOS sessiontemplate created: {}",
        create_bos_session_template_resp.unwrap()
    );

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
