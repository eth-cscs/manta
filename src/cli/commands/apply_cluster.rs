use core::time;
use std::{path::PathBuf, thread};

// use clap::ArgMatches;
use mesa::shasta::{cfs::session, ims::image, bos::{template, self}, hsm, capmc};
use serde_yaml::Value;

use crate::common::jwt_ops::get_claims_from_jwt_token;

use super::apply_image;

pub async fn exec(
    vault_base_url: &str,
    vault_secret_path: &str,
    vault_role_id: &str,
    // cli_apply_image: &ArgMatches,
    shasta_token: &str,
    shasta_base_url: &str,
    path_file: &PathBuf,
    // base_image_id: &str,
    hsm_group_param: Option<&String>,
    k8s_api_url: &str,
    tag: String
) {
    // let path_file: &PathBuf = cli_apply_image.get_one("file").unwrap();
    let file_content = std::fs::read_to_string(path_file).unwrap();
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

    // Create CFS configuration and image
    let (_, cfs_session_name) = apply_image::exec(
        vault_base_url,
        vault_secret_path,
        vault_role_id,
        // cli_apply_image,
        path_file,
        shasta_token,
        shasta_base_url,
        // base_image_id,
        Some(&false),
        &tag,
        hsm_group_param,
        k8s_api_url,
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

    log::debug!("CFS session response:\n{:#?}", cfs_sessions_details);

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
    let image_details = image::http_client::get(
        shasta_token,
        shasta_base_url,
        &cfs_session_result_id,
    )
    .await;

    log::debug!("IMS image response:\n{:#?}", image_details);

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
        .replace("__DATE__", &tag);

    // Create BOS sessiontemplate

    let bos_session_template_yaml = bos_session_templates_yaml.iter().next().unwrap();

    let bos_session_template_name = bos_session_template_yaml["name"]
        .as_str()
        .unwrap()
        .to_string()
        .replace("__DATE__", &tag);

    let bos_session_template_hsm_groups: Vec<String> = bos_session_template_yaml["bos_parameters"]
        ["boot_sets"]["compute"]["node_groups"]
        .as_sequence()
        .unwrap()
        .iter()
        .map(|node| node.as_str().unwrap().to_string())
        .collect();

    // Check HSM groups in YAML file session_templates.bos_parameters.boot_sets.compute.node_groups matches with
    // Check hsm groups in SAT file includes the hsm_group_param
    let hsm_group = if hsm_group_param.is_some()
        && !bos_session_template_hsm_groups
            .iter()
            .any(|h_g| h_g.eq(hsm_group_param.unwrap()))
    {
        eprintln!("HSM group in param does not matches with any HSM groups in SAT file under session_templates.bos_parameters.boot_sets.compute.node_groups section. Using HSM group in param as the default");
        hsm_group_param.unwrap()
    } else {
        bos_session_template_hsm_groups.first().unwrap()
    };

    let create_bos_session_template_payload = bos::template::BosTemplate::new_for_hsm_group(
        cfs_configuration_name,
        bos_session_template_name,
        ims_image_name,
        ims_image_path,
        ims_image_type,
        ims_image_etag,
        hsm_group,
    );

    log::debug!(
        "create BOS session template payload:\n{:#?}",
        create_bos_session_template_payload
    );

    let create_bos_session_template_resp = template::http_client::post(
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

    // Create BOS session. Note: reboot operation shuts down the nodes and don't bring them back
    // up... hence we will split the reboot into 2 operations shutdown and start

    // Get nodes members of HSM group
    // Get HSM group details
    let hsm_group_details =
        hsm::http_client::get_hsm_group(shasta_token, shasta_base_url, hsm_group)
            .await;

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
        Some("Shut down cluster to apply changes".to_string()),
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

    // Audit
    let jwt_claims = get_claims_from_jwt_token(shasta_token).unwrap();

    log::info!(target: "app::audit", "User: {} ({}) ; Operation: Apply cluster", jwt_claims["name"].as_str().unwrap(), jwt_claims["preferred_username"].as_str().unwrap());
}
