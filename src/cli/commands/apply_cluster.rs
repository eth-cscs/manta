use core::time;
use std::{path::PathBuf, thread};

use clap::ArgMatches;
use k8s_openapi::chrono;
use serde_yaml::Value;

use crate::shasta::cfs::session;

use super::apply_image;

pub async fn exec(
    vault_base_url: &str,
    vault_role_id: &String,
    cli_apply_image: &ArgMatches,
    shasta_token: &String,
    shasta_base_url: &String,
    base_image_id: &String,
    hsm_group_param: Option<&String>,
) {
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
    let (_, cfs_session_name) = apply_image::exec(
        vault_base_url,
        vault_role_id,
        cli_apply_image,
        shasta_token,
        shasta_base_url,
        base_image_id,
        Some(&false),
        &timestamp,
    )
    .await;

    // Monitor CFS image creation process ends
    let mut cfs_sessions_details = session::http_client::get(
        shasta_token,
        shasta_base_url,
        None,
        Some(&cfs_session_name),
        Some(&1),
        Some(true),
    )
    .await
    .unwrap();

    // Wait for CFS session target image to finish
    let mut i = 0;
    let max = 1800; // Max ammount of attempts to check if CFS session has ended
    while !cfs_sessions_details.first().unwrap()["status"]["session"]["status"].eq("complete")
        && i <= max
    {
        print!(
            "\rCFS session {} running. Checking again in 2 secs. Attempt {} of {}",
            cfs_session_name,
            i + 1,
            max
        );

        thread::sleep(time::Duration::from_secs(2));
        std::io::Write::flush(&mut std::io::stdout()).unwrap();

        cfs_sessions_details = session::http_client::get(
            shasta_token,
            shasta_base_url,
            None,
            Some(&cfs_session_name),
            Some(&1),
            Some(true),
        )
        .await
        .unwrap();

        i += 1;
    }

    println!();

    log::info!("Get CFS session details:\n{:#?}", cfs_sessions_details);

    /* // Get groups from SAT images.configuration_group_names file
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

    let hsm_group;

    // let hsm_group = hsm_groups.iter().next();

    /* if hsm_group.is_none() {
        eprintln!("SAT file images.configuration_group_names does not have valid HSM groups");
        std::process::exit(1);
    } */

    // Check hsm groups in SAT file includes the hsm group in params
    if !hsm_groups.iter().any(|h_g| h_g.eq(hsm_group_param.unwrap())) {
        eprintln!("HSM group in param does not matches with any HSM groups in SAT file under images.configuration_group_names section. Using HSM group in param as the default");
    }

    hsm_group = hsm_group_param.unwrap();

    if hsm_group.is_empty() {
        eprintln!("No HSM group available. Exiting");
        std::process::exit(1);
    } */

    // Get data from yaml to create BOS session template
    if !cfs_sessions_details.first().unwrap()["status"]["session"]["status"].eq("complete") {
        eprintln!("Session running for too long, exit");
        std::process::exit(1);
    }

    if !cfs_sessions_details.first().unwrap()["status"]["session"]["succeeded"].eq("true") {
        eprintln!("Session failed, exit");
        std::process::exit(1);
    }

    let cfs_session_result_id = cfs_sessions_details.first().unwrap()["status"]["artifacts"]
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
        &cfs_session_result_id,
    )
    .await;

    log::info!("IMS image details:\n{:#?}", image_details);

    /* // Wait till image details are available
    let mut i = 0;
    let max = 50; // Max ammount of attempts to check if IMS image details are available
    while image_details.is_err() && i <= max {
        eprint!(
            "\rCould not fetch image details for result_id {}. Trying again in 2 seconds. Attempt {} of {}",
            cfs_session_result_id, i + 1, max
        );
        thread::sleep(time::Duration::from_secs(2));
        std::io::Write::flush(&mut std::io::stdout()).unwrap();

        i += 1;
    }

    println!();

    if image_details.is_err() {
        eprintln!("Could not fetch image details. Exit");
        std::process::exit(1);
    } */

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

    // Create BOS sessiontemplate

    let bos_session_template_yaml = bos_session_templates_yaml.iter().next().unwrap();

    let bos_session_template_name = bos_session_template_yaml["name"]
            .as_str()
            .unwrap()
            .to_string()
            .replace("__DATE__", &timestamp);

    let bos_session_template_hsm_groups: Vec<String> = bos_session_template_yaml["bos_parameters"]["boot_sets"]
        ["compute"]["node_groups"]
        .as_sequence()
        .unwrap()
        .iter()
        .map(|node| node.as_str().unwrap().to_string())
        .collect();

    // Check HSM groups in YAML file session_templates.bos_parameters.boot_sets.compute.node_groups with
    // Check hsm groups in SAT file includes the hsm_group_param
    let hsm_group = if !bos_session_template_hsm_groups
        .iter()
        .any(|h_g| h_g.eq(hsm_group_param.unwrap()))
    {
        eprintln!("HSM group in param does not matches with any HSM groups in SAT file under session_templates.bos_parameters.boot_sets.compute.node_groups section. Using HSM group in param as the default");
        hsm_group_param.unwrap()
    } else {
        bos_session_template_hsm_groups.first().unwrap()
    };

    let cfs = crate::shasta::bos::template::Cfs {
        clone_url: None,
        branch: None,
        commit: None,
        playbook: None,
        configuration: Some(cfs_configuration_name),
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
        node_groups: Some(vec!(hsm_group.to_string())),
        rootfs_provider: Some("cpss3".to_string()),
        rootfs_provider_passthrough: Some("dvs:api-gw-service-nmn.local:300:nmn0".to_string()),
    };

    let boot_set = crate::shasta::bos::template::BootSet {
        compute: Some(compute_property),
    };

    let create_bos_session_template_payload = crate::shasta::bos::template::BosTemplate {
        name: bos_session_template_name,
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
    // Get HSM group details
    let hsm_group_details =
        crate::shasta::hsm::http_client::get_hsm_group(shasta_token, shasta_base_url, hsm_group)
            .await;

    log::info!("hsm_group_details:\n{:#?}", hsm_group_details);

    // Get list of xnames in HSM group
    let nodes: Vec<String> = hsm_group_details.unwrap()["members"]["ids"]
        .as_array()
        .unwrap()
        .iter()
        .map(|node| node.as_str().unwrap().to_string())
        .collect();

    // Create CAPMC operation shutdown
    let capmc_shutdown_nodes_resp = crate::shasta::capmc::http_client::node_power_off::post_sync(
        shasta_token.to_string(),
        shasta_base_url.to_string(),
        Some(&"testing manta".to_string()),
        nodes.clone(),
        true,
    )
    .await;

    log::info!("CAPMC shutdown nodes response:\n{:#?}", capmc_shutdown_nodes_resp);

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
