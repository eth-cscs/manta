use core::time;
use std::{path::PathBuf, thread};

use clap::ArgMatches;
use k8s_openapi::chrono;
use serde_yaml::Value;

use crate::shasta::cfs::session;

use super::apply_image;

pub async fn exec(
    vault_base_url: &String,
    cli_apply_image: &ArgMatches,
    shasta_token: &String,
    shasta_base_url: &String,
    base_image_id: &String,
    // hsm_group: Option<&String>,
) {
    /* // START TEST!!!!!

    let hsm_group = Some("zinal".to_string());
    let hsm_groups = Some(vec!["zinal".to_string()]);

    let cfs = crate::shasta::bos::template::Cfs {
        clone_url: None,
        branch: None,
        commit: None,
        playbook: None,
        configuration: Some("zinal-cos-config-20230314191421".to_string()), // Some(cfs_configuration_yaml["name"].as_str().unwrap().to_string()),
    };

    let compute_property = crate::shasta::bos::template::Property {
        name: Some(
            "cray-shasta-compute-sles15sp3.x86_64-2.3.33_cfs_zinal-cos-20230314191421".to_string(),
        ),
        boot_ordinal: Some(2),
        shutdown_ordinal: None,
        path: Some(
            "s3://boot-images/d7522ade-f698-4ea4-b97f-342be042fc9c/manifest.json".to_string(),
        ),
        type_prop: Some("s3".to_string()),
        etag: Some("90977d638ca2193d4443033c8698bfbb".to_string()),
        kernel_parameters: Some("ip=dhcp quiet spire_join_token=${SPIRE_JOIN_TOKEN}".to_string()),
        network: Some("nmn".to_string()),
        node_list: None,
        node_roles_groups: None,
        node_groups: hsm_groups,
        rootfs_provider: Some("cpss3".to_string()),
        rootfs_provider_passthrough: Some("dvs:api-gw-service-nmn.local:300:nmn0".to_string()),
    };

    let boot_set = crate::shasta::bos::template::BootSet {
        compute: Some(compute_property),
    };
    let create_bos_session_template_payload = crate::shasta::bos::template::BosTemplate {
        name: "zinal-cos-template-20230314191421".to_string(),
        template_url: None,
        description: None,
        cfs_url: None,
        cfs_branch: None,
        enable_cfs: Some(true),
        cfs: Some(cfs),
        partition: None,
        boot_sets: Some(boot_set),
        links: None,
    };

    // Create BOS session. Note: reboot operation shuts down the nodes and don't bring them back
    // up... hence we will split the reboot into 2 operations shutdown and start

    // Get nodes members of HSM group
    let hsm_group_details = crate::shasta::hsm::http_client::get_hsm_group(
        shasta_token,
        shasta_base_url,
        &hsm_group.unwrap(),
    )
    .await;

    log::info!("hsm_group_details:\n{:#?}", hsm_group_details);

    let mut nodes: Vec<String> = hsm_group_details.unwrap()["members"]["ids"]
        .as_array()
        .unwrap()
        .into_iter()
        .map(|node| node.as_str().unwrap().to_string())
        .collect();

    nodes = vec!["x1001c1s6b1n1".to_string(), "x1006c1s4b0n0".to_string()];

    // Create CAPMC operation shutdown
    let capmc_shutdown_nodes_resp = crate::shasta::capmc::http_client::node_power_off::post(
        shasta_token.to_string(),
        shasta_base_url.to_string(),
        Some(&"testing manta".to_string()),
        nodes.clone(),
        true,
    )
    .await;

    log::info!("Shutdown nodes resp:\n{:#?}", capmc_shutdown_nodes_resp);

    // Check Nodes are shutdown
    let mut nodes_status = crate::shasta::hsm::http_client::get_components_status(
        shasta_token,
        shasta_base_url,
        &nodes,
    )
    .await;

    log::info!("nodes_status:\n{:#?}", nodes_status);

    // Check all nodes are OFF
    let mut i = 0;
    while i < 60
        && !nodes_status.as_ref().unwrap()["Components"]
            .as_array()
            .unwrap()
            .iter()
            .all(|node| node["State"].as_str().unwrap().to_string().eq("Off"))
    {
        println!("Waititing nodes to shutdown ...",);
        thread::sleep(time::Duration::from_secs(2));
        i += 1;
        log::info!("nodes_status:\n{:#?}", nodes_status);
        nodes_status = crate::shasta::hsm::http_client::get_components_status(
            shasta_token,
            shasta_base_url,
            &nodes,
        )
        .await;
    }

    log::info!("node status resp:\n{:#?}", nodes_status);

    // Create BOS session operation start
    let create_bos_boot_session_resp = crate::shasta::bos::session::http_client::post(
        shasta_token,
        shasta_base_url,
        &create_bos_session_template_payload.name,
        "boot",
        Some(&nodes.join(",")),
    )
    .await;

    log::info!(
        "Create BOS boot session resp:\n{:#?}",
        create_bos_boot_session_resp
    );

    if create_bos_boot_session_resp.is_err() {
        eprintln!("Error creating BOS boot session. Exit");
        std::process::exit(1);
    }

    std::process::exit(0);

    // END TEST!!!!! */

    let path_buf: &PathBuf = cli_apply_image.get_one("file").unwrap();
    let file_content = std::fs::read_to_string(path_buf).unwrap();
    let sat_file_yaml: Value = serde_yaml::from_str(&file_content).unwrap();

    let bos_session_templates_yaml = sat_file_yaml["session_templates"].as_sequence().unwrap();

    if bos_session_templates_yaml.is_empty() {
        eprintln!("The input file has no configurations!");
        std::process::exit(-1);
    }

    if bos_session_templates_yaml.len() > 1 {
        eprintln!("Multiple CFS configurations found in input file, please clean the file so it only contains one.");
        std::process::exit(-1);
    }

    let timestamp = chrono::Utc::now().format("%Y%m%d%H%M%S").to_string();

    // Create CFS configuration and image
    let cfs_session_name = apply_image::exec(
        vault_base_url,
        cli_apply_image,
        shasta_token,
        shasta_base_url,
        base_image_id,
        Some(&false),
        &timestamp,
    )
    .await;

    // Get groups from SAT images.configuration_group_names file
    let mut hsm_groups: Vec<String> = sat_file_yaml["images"]
        .as_sequence()
        .unwrap()
        .into_iter()
        .next()
        .unwrap()["configuration_group_names"]
        .as_sequence()
        .unwrap()
        .into_iter()
        .map(|member| member.as_str().unwrap().to_string())
        .collect();

    // Filter groups from SAT file to hsm groups only
    hsm_groups = hsm_groups
        .into_iter()
        .filter(|member| {
            !member.to_lowercase().eq(&"compute") && !member.to_lowercase().eq(&"application")
        })
        .collect::<Vec<_>>();

    let hsm_group = hsm_groups.iter().next();

    if hsm_group.is_none() {
        eprintln!("SAT file images.configuration_group_names does not have valid HSM groups");
        std::process::exit(1);
    }

    // Monitor CFS image creation process ends
    let mut cfs_sessions_details = session::http_client::get(
        shasta_token,
        shasta_base_url,
        hsm_group,
        Some(&cfs_session_name),
        Some(&1),
        Some(true),
    )
    .await
    .unwrap();

    // Wait for CFS session target image to finish
    let mut i = 0;
    let max = 720; // Max ammount of attempts to check if CFS session has ended
    while !cfs_sessions_details.iter().next().unwrap()["status"]["session"]["status"].eq("complete")
        && i < max
    {
        println!(
            "CFS session {} running. Checking again in 5 secs ({}/{}) ...",
            cfs_session_name, i, max
        );

        thread::sleep(time::Duration::from_secs(5));

        cfs_sessions_details = session::http_client::get(
            shasta_token,
            shasta_base_url,
            hsm_group,
            Some(&cfs_session_name),
            Some(&1),
            Some(true),
        )
        .await
        .unwrap();

        i += 1;
    }

    log::info!("Get CFS session details:\n{:#?}", cfs_sessions_details);

    if !cfs_sessions_details.iter().next().unwrap()["status"]["session"]["status"].eq("complete") {
        eprintln!("Session running for too long, exit");
        std::process::exit(1);
    }

    if !cfs_sessions_details.iter().next().unwrap()["status"]["session"]["succeeded"].eq("true") {
        eprintln!("Session failed, exit");
        std::process::exit(1);
    }

    let cfs_session_result_id = cfs_sessions_details.iter().next().unwrap()["status"]["artifacts"]
        .as_array()
        .unwrap()
        .iter()
        .next()
        .unwrap()["result_id"]
        .as_str()
        .unwrap()
        .to_string();

    log::info!("CFS session result_id: {}", cfs_session_result_id);

    // Get image details
    let image_details = crate::shasta::ims::image::http_client::get(
        shasta_token,
        shasta_base_url,
        hsm_group,
        &cfs_session_result_id,
    )
    .await;

    log::info!("IMS image details:\n{:#?}", image_details);

    // Wait till image details are available
    let mut i = 0;
    let max = 20; // Max ammount of attempts to check if IMS image details are available
    while image_details.is_err() && i < max {
        eprintln!(
            "Could not fetch image details for result_id {}. Trying again in 2 seconds ({}/{}) ...",
            cfs_session_result_id, i, max
        );
        thread::sleep(time::Duration::from_secs(5));

        i += 1;
    }

    if image_details.is_err() {
        eprintln!("Could not fetch image details. Exit");
        std::process::exit(1);
    }

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

    let cfs_configuration_yaml = sat_file_yaml["configurations"]
        .as_sequence()
        .unwrap()
        .iter()
        .next()
        .unwrap();

    let cfs_configuration_name = cfs_configuration_yaml["name"]
        .as_str()
        .unwrap()
        .to_string()
        .replace("__DATE__", &timestamp);

    /* let cfs_session_status_artifacts_image_id = cfs_sessions_details.into_iter().next().unwrap()
    ["status"]["artifacts"]
    .as_array()
    .unwrap()
    .into_iter()
    .next()
    .unwrap()["image_id"]
    .as_str()
    .unwrap()
    .to_string(); */

    // Create BOS sessiontemplate
    let bos_session_template_yaml = bos_session_templates_yaml.iter().next().unwrap();

    let hsm_groups: Vec<String> = bos_session_template_yaml["bos_parameters"]
        // .as_sequence()
        // .unwrap()
        // .into_iter()
        // .next()
        // .unwrap()
        ["boot_sets"]["compute"]["node_groups"]
        .as_sequence()
        .unwrap()
        .into_iter()
        .map(|node| node.as_str().unwrap().to_string())
        .collect();

    let cfs = crate::shasta::bos::template::Cfs {
        clone_url: None,
        branch: None,
        commit: None,
        playbook: None,
        configuration: Some(cfs_configuration_name), // Some(cfs_configuration_yaml["name"].as_str().unwrap().to_string()),
    };

    let compute_property = crate::shasta::bos::template::Property {
        name: Some(ims_image_name),
        boot_ordinal: Some(2),
        shutdown_ordinal: None,
        path: Some(ims_image_path),
        type_prop: Some(ims_image_type),
        etag: Some(ims_image_etag),
        kernel_parameters: Some("ip=dhcp quiet spire_join_token=${SPIRE_JOIN_TOKEN}".to_string()),
        network: Some("nmn".to_string()),
        node_list: None,
        node_roles_groups: None,
        node_groups: Some(hsm_groups),
        rootfs_provider: Some("cpss3".to_string()),
        rootfs_provider_passthrough: Some("dvs:api-gw-service-nmn.local:300:nmn0".to_string()),
    };

    let boot_set = crate::shasta::bos::template::BootSet {
        compute: Some(compute_property),
    };

    let create_bos_session_template_payload = crate::shasta::bos::template::BosTemplate {
        name: bos_session_template_yaml["name"]
            .as_str()
            .unwrap()
            .to_string()
            .replace("__DATE__", &timestamp),
        template_url: None,
        description: None,
        cfs_url: None,
        cfs_branch: None,
        enable_cfs: Some(true),
        cfs: Some(cfs),
        partition: None,
        boot_sets: Some(boot_set),
        links: None,
    };

    log::info!(
        "create BOS session template payload:\n{:#?}",
        create_bos_session_template_payload
    );

    let create_bos_session_template_resp = crate::shasta::bos::template::http_client::post(
        shasta_token,
        shasta_base_url,
        &create_bos_session_template_payload,
    )
    .await;

    log::info!(
        "Create BOS session template response:\n{:#?}",
        create_bos_session_template_resp
    );

    if create_bos_session_template_resp.is_err() {
        log::error!("BOS session template creation failed");
        std::process::exit(1);
    }

    // Create BOS session. Note: reboot operation shuts down the nodes and don't bring them back
    // up... hence we will split the reboot into 2 operations shutdown and start

    // Get nodes members of HSM group
    let hsm_group_details = crate::shasta::hsm::http_client::get_hsm_group(
        shasta_token,
        shasta_base_url,
        hsm_group.unwrap(),
    )
    .await;

    log::info!("hsm_group_details:\n{:#?}", hsm_group_details);

    let mut nodes: Vec<String> = hsm_group_details.unwrap()["members"]["ids"]
        .as_array()
        .unwrap()
        .into_iter()
        .map(|node| node.as_str().unwrap().to_string())
        .collect();

    // Create CAPMC operation shutdown
    let capmc_shutdown_nodes_resp = crate::shasta::capmc::http_client::node_power_off::post(
        shasta_token.to_string(),
        shasta_base_url.to_string(),
        Some(&"testing manta".to_string()),
        nodes.clone(),
        true,
    )
    .await;

    log::info!("Shutdown nodes resp:\n{:#?}", capmc_shutdown_nodes_resp);

    // Check Nodes are shutdown
    let mut nodes_status = crate::shasta::hsm::http_client::get_components_status(
        shasta_token,
        shasta_base_url,
        &nodes,
    )
    .await;

    log::info!("nodes_status:\n{:#?}", nodes_status);

    // Check all nodes are OFF
    let mut i = 0;
    while i < 60
        && !nodes_status.as_ref().unwrap()["Components"]
            .as_array()
            .unwrap()
            .iter()
            .all(|node| node["State"].as_str().unwrap().to_string().eq("Off"))
    {
        println!("Waititing nodes to shutdown ...",);
        thread::sleep(time::Duration::from_secs(2));
        i += 1;
        log::info!("nodes_status:\n{:#?}", nodes_status);
        nodes_status = crate::shasta::hsm::http_client::get_components_status(
            shasta_token,
            shasta_base_url,
            &nodes,
        )
        .await;
    }

    log::info!("node status resp:\n{:#?}", nodes_status);

    // Create BOS session operation start
    let create_bos_boot_session_resp = crate::shasta::bos::session::http_client::post(
        shasta_token,
        shasta_base_url,
        &create_bos_session_template_payload.name,
        "boot",
        Some(&nodes.join(",")),
    )
    .await;

    log::info!(
        "Create BOS boot session resp:\n{:#?}",
        create_bos_boot_session_resp
    );

    if create_bos_boot_session_resp.is_err() {
        eprintln!("Error creating BOS boot session. Exit");
        std::process::exit(1);
    }
}
