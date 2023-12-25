use core::time;
use std::{path::PathBuf, thread};

// use clap::ArgMatches;
use mesa::{
    cfs::{
        self,
        configuration::shasta::r#struct::cfs_configuration_request::CfsConfigurationRequest,
        session::mesa::r#struct::{CfsSessionGetResponse, CfsSessionRequest},
    },
    {capmc, hsm},
};
use serde_yaml::Value;

use crate::{
    cli::commands::apply_image::validate_sat_file_images_section,
    common::jwt_ops::get_claims_from_jwt_token,
};

pub async fn exec(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    path_file: &PathBuf,
    hsm_group_param_opt: Option<&String>,
    hsm_group_available_vec: &Vec<String>,
    ansible_verbosity_opt: Option<&String>,
    ansible_passthrough_opt: Option<&String>,
    tag: String,
) {
    let file_content = std::fs::read_to_string(path_file).unwrap();
    let sat_file_yaml: Value = serde_yaml::from_str(&file_content).unwrap();

    // Get CFS configurations from SAT YAML file
    let configuration_yaml_value_vec_opt = sat_file_yaml["configurations"].as_sequence();

    // Get inages from SAT YAML file
    let image_yaml_vec_opt = sat_file_yaml["images"].as_sequence();

    // Get inages from SAT YAML file
    let bos_session_template_yaml_vec_opt = sat_file_yaml["session_templates"].as_sequence();

    // VALIDATION
    validate_sat_file_images_section(image_yaml_vec_opt, hsm_group_available_vec);
    // Check HSM groups in session_templates in SAT file section matches the ones in JWT token (keycloak roles) in  file
    // This is a bit messy... images section in SAT file valiidation is done inside apply_image::exec but the
    // validation of session_templates section in the SAT file is below
    validate_sat_file_session_template_section(
        bos_session_template_yaml_vec_opt,
        hsm_group_available_vec,
    );
    /* for bos_session_template_yaml in bos_session_template_list_yaml.unwrap_or(&vec![]) {
        let bos_session_template_hsm_groups: Vec<String> = if let Some(boot_sets_compute) =
            bos_session_template_yaml["bos_parameters"]["boot_sets"].get("compute")
        {
            boot_sets_compute["node_groups"]
                .as_sequence()
                .unwrap_or(&vec![])
                .iter()
                .map(|node| node.as_str().unwrap().to_string())
                .collect()
        } else if let Some(boot_sets_compute) =
            bos_session_template_yaml["bos_parameters"]["boot_sets"].get("uan")
        {
            boot_sets_compute["node_groups"]
                .as_sequence()
                .unwrap_or(&vec![])
                .iter()
                .map(|node| node.as_str().unwrap().to_string())
                .collect()
        } else {
            println!("No HSM group found in session_templates section in SAT file");
            std::process::exit(1);
        };

        for hsm_group in bos_session_template_hsm_groups {
            if !hsm_group_available_vec.contains(&hsm_group.to_string()) {
                println!(
                        "HSM group '{}' in session_templates {} not allowed, List of HSM groups available {:?}. Exit",
                        hsm_group,
                        bos_session_template_yaml["name"].as_str().unwrap(),
                        hsm_group_available_vec
                    );
                std::process::exit(-1);
            }
        }
    } */

    /* // Create CFS configuration and image
    let (cfs_configuration_yaml_vec, cfs_session_vec) = apply_image::exec(
        vault_base_url,
        vault_secret_path,
        vault_role_id,
        path_file,
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        ansible_verbosity_opt,
        ansible_passthrough_opt,
        watch_logs,
        &tag,
        hsm_group_available_vec,
        k8s_api_url,
        output_opt,
    )
    .await; */

    // Process "configurations" section in SAT file

    let mut cfs_configuration_value_vec = Vec::new();

    let mut cfs_configuration_name_vec = Vec::new();

    for cfs_configuration_yaml_value in configuration_yaml_value_vec_opt.unwrap_or(&vec![]).iter() {
        let mut cfs_configuration_value =
            CfsConfigurationRequest::from_sat_file_serde_yaml(cfs_configuration_yaml_value);

        // Rename configuration name
        cfs_configuration_value.name = cfs_configuration_value.name.replace("__DATE__", &tag);

        log::debug!(
            "CFS configuration creation payload:\n{:#?}",
            cfs_configuration_value
        );

        let cfs_configuration_value_rslt = cfs::configuration::shasta::http_client::put(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            &cfs_configuration_value,
            &cfs_configuration_value.name,
        )
        .await;

        let cfs_configuration_value =
            if let Ok(cfs_configuration_value) = cfs_configuration_value_rslt {
                cfs_configuration_value
            } else {
                eprintln!("CFS configuration creation failed. Exit");
                std::process::exit(1);
            };

        let cfs_configuration_name = cfs_configuration_value["name"]
            .as_str()
            .unwrap()
            .to_string();

        cfs_configuration_name_vec.push(cfs_configuration_name.clone());

        log::info!("CFS configuration created: {}", cfs_configuration_name);

        cfs_configuration_value_vec.push(cfs_configuration_value.clone());
    }

    // Process "images" section in SAT file

    let mut cfs_session_complete_vec: Vec<CfsSessionGetResponse> = Vec::new();

    for image_yaml in image_yaml_vec_opt.unwrap_or(&vec![]) {
        let mut cfs_session = CfsSessionRequest::from_sat_file_serde_yaml(image_yaml);

        // Rename session name
        cfs_session.name = cfs_session.name.replace("__DATE__", &tag);

        // Rename session's configuration name
        cfs_session.configuration_name = cfs_session.configuration_name.replace("__DATE__", &tag);

        // Set ansible verbosity
        cfs_session.ansible_verbosity = Some(
            ansible_verbosity_opt
                .cloned()
                .unwrap_or("0".to_string())
                .parse::<u8>()
                .unwrap(),
        );

        // Set ansible passthrough params
        cfs_session.ansible_passthrough = ansible_passthrough_opt.cloned();

        let create_cfs_session_resp = mesa::cfs::session::mesa::http_client::post(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            &cfs_session,
        )
        .await;

        if create_cfs_session_resp.is_err() {
            eprintln!("CFS session creation failed. Exit");
            std::process::exit(1);
        }

        // Monitor CFS image creation process ends
        let mut i = 0;
        let max = 1800; // Max ammount of attempts to check if CFS session has ended
        loop {
            let mut cfs_session_value_vec = mesa::cfs::session::shasta::http_client::get(
                shasta_token,
                shasta_base_url,
                shasta_root_cert,
                Some(&cfs_session.name.to_string()),
                Some(true),
            )
            .await
            .unwrap();

            mesa::cfs::session::shasta::http_client::filter_by_hsm(
                shasta_token,
                shasta_base_url,
                shasta_root_cert,
                &mut cfs_session_value_vec,
                hsm_group_available_vec,
                Some(&1),
            )
            .await;

            if !cfs_session_value_vec.is_empty()
                && cfs_session_value_vec.first().unwrap()["status"]["session"]["status"]
                    .eq("complete")
                && i <= max
            {
                let cfs_session_aux: CfsSessionGetResponse =
                    CfsSessionGetResponse::from_csm_api_json(
                        cfs_session_value_vec.first().unwrap().clone(),
                    );

                cfs_session_complete_vec.push(cfs_session_aux.clone());

                log::info!("CFS session created: {}", cfs_session_aux.name.unwrap());

                break;
            } else {
                print!(
                    "\rCFS session '{}' running. Checking again in 2 secs. Attempt {} of {}",
                    cfs_session.name,
                    i + 1,
                    max
                );

                thread::sleep(time::Duration::from_secs(2));
                std::io::Write::flush(&mut std::io::stdout()).unwrap();

                i += 1;
            }
        }
    }

    println!(); // Don't delete we do need to print an empty line here for the previous waiting CFS
                // session message

    // Process "session_templates" section in SAT file

    process_session_template_section_in_sat_file(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        hsm_group_param_opt,
        hsm_group_available_vec,
        sat_file_yaml,
        &tag,
    )
    .await;

    /* let empty_vec = Vec::new();
    let bos_session_template_list_yaml = sat_file_yaml["session_templates"]
        .as_sequence()
        .unwrap_or(&empty_vec);

    for bos_session_template_yaml in bos_session_template_list_yaml {
        let mut bos_session_template_image_name = bos_session_template_yaml["image"]
            .as_str()
            .unwrap_or("")
            .to_string();

        bos_session_template_image_name = bos_session_template_image_name.replace("__DATE__", &tag);

        // Get CFS configuration to configure the nodes
        let bos_session_template_configuration_name = bos_session_template_yaml["configuration"]
            .as_str()
            .unwrap()
            .to_string()
            .replace("__DATE__", &tag);

        let cfs_configuration_detail_opt = mesa::shasta::cfs::configuration::http_client::get(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            Some(hsm_group_available_vec),
            Some(&bos_session_template_configuration_name),
            None,
        )
        .await;

        if cfs_configuration_detail_opt.is_err() {
            eprintln!(
                "ERROR: BOS session template configuration not found in SAT file image list."
            );
            std::process::exit(1);
        }

        // Get base image details
        let image_detail_vec = image::http_client::get_fuzzy(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            hsm_group_available_vec,
            Some(&bos_session_template_image_name),
            Some(&1),
        )
        .await
        .unwrap_or(Vec::new());

        log::info!(
            "Image name: {}",
            image_detail_vec.first().unwrap()["name"].as_str().unwrap()
        );

        let ims_image_name = image_detail_vec.first().unwrap()["name"]
            .as_str()
            .unwrap()
            .to_string();
        let ims_image_etag = image_detail_vec.first().unwrap()["link"]["etag"]
            .as_str()
            .unwrap()
            .to_string();
        let ims_image_path = image_detail_vec.first().unwrap()["link"]["path"]
            .as_str()
            .unwrap()
            .to_string();
        let ims_image_type = image_detail_vec.first().unwrap()["link"]["type"]
            .as_str()
            .unwrap()
            .to_string();

        let bos_session_template_name = bos_session_template_yaml["name"]
            .as_str()
            .unwrap_or("")
            .to_string()
            .replace("__DATE__", &tag);

        let bos_session_template_hsm_groups: Vec<String> = if let Some(boot_sets_compute) =
            bos_session_template_yaml["bos_parameters"]["boot_sets"].get("compute")
        {
            boot_sets_compute["node_groups"]
                .as_sequence()
                .unwrap_or(&vec![])
                .iter()
                .map(|node| node.as_str().unwrap().to_string())
                .collect()
        } else if let Some(boot_sets_compute) =
            bos_session_template_yaml["bos_parameters"]["boot_sets"].get("uan")
        {
            boot_sets_compute["node_groups"]
                .as_sequence()
                .unwrap_or(&vec![])
                .iter()
                .map(|node| node.as_str().unwrap().to_string())
                .collect()
        } else {
            println!("No HSM group found in session_templates section in SAT file");
            std::process::exit(1);
        };

        // Check HSM groups in YAML file session_templates.bos_parameters.boot_sets.compute.node_groups matches with
        // Check hsm groups in SAT file includes the hsm_group_param
        let hsm_group = if hsm_group_param_opt.is_some()
            && !bos_session_template_hsm_groups
                .iter()
                .any(|h_g| h_g.eq(hsm_group_param_opt.unwrap()))
        {
            eprintln!("HSM group in param does not matches with any HSM groups in SAT file under session_templates.bos_parameters.boot_sets.compute.node_groups section. Using HSM group in param as the default");
            hsm_group_param_opt.unwrap()
        } else {
            bos_session_template_hsm_groups.first().unwrap()
        };

        let create_bos_session_template_payload =
            bos::template::BosTemplateRequest::new_for_hsm_group(
                bos_session_template_configuration_name,
                bos_session_template_name,
                ims_image_name,
                ims_image_path,
                ims_image_type,
                ims_image_etag,
                hsm_group,
            );

        let create_bos_session_template_resp = template::http_client::post(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            &create_bos_session_template_payload,
        )
        .await;

        if create_bos_session_template_resp.is_err() {
            eprintln!("BOS session template creation failed");
            std::process::exit(1);
        }

        // Create BOS session. Note: reboot operation shuts down the nodes and they may not start
        // up... hence we will split the reboot into 2 operations shutdown and start

        // Get nodes members of HSM group
        // Get HSM group details
        let hsm_group_details = hsm::http_client::get_hsm_group(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            hsm_group,
        )
        .await;

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
            shasta_root_cert,
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
            shasta_root_cert,
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
    } */
}

pub fn validate_sat_file_session_template_section(
    bos_session_template_list_yaml: Option<&Vec<Value>>,
    hsm_group_available_vec: &Vec<String>,
) {
    for bos_session_template_yaml in bos_session_template_list_yaml.unwrap_or(&vec![]) {
        let bos_session_template_hsm_groups: Vec<String> = if let Some(boot_sets_compute) =
            bos_session_template_yaml["bos_parameters"]["boot_sets"].get("compute")
        {
            boot_sets_compute["node_groups"]
                .as_sequence()
                .unwrap_or(&vec![])
                .iter()
                .map(|node| node.as_str().unwrap().to_string())
                .collect()
        } else if let Some(boot_sets_compute) =
            bos_session_template_yaml["bos_parameters"]["boot_sets"].get("uan")
        {
            boot_sets_compute["node_groups"]
                .as_sequence()
                .unwrap_or(&vec![])
                .iter()
                .map(|node| node.as_str().unwrap().to_string())
                .collect()
        } else {
            println!("No HSM group found in session_templates section in SAT file");
            std::process::exit(1);
        };

        for hsm_group in bos_session_template_hsm_groups {
            if !hsm_group_available_vec.contains(&hsm_group.to_string()) {
                println!(
                        "HSM group '{}' in session_templates {} not allowed, List of HSM groups available {:?}. Exit",
                        hsm_group,
                        bos_session_template_yaml["name"].as_str().unwrap(),
                        hsm_group_available_vec
                    );
                std::process::exit(-1);
            }
        }
    }
}

pub async fn process_session_template_section_in_sat_file(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    hsm_group_param_opt: Option<&String>,
    hsm_group_available_vec: &Vec<String>,
    sat_file_yaml: Value,
    tag: &str,
) {
    let empty_vec = Vec::new();
    let bos_session_template_list_yaml = sat_file_yaml["session_templates"]
        .as_sequence()
        .unwrap_or(&empty_vec);

    for bos_session_template_yaml in bos_session_template_list_yaml {
        let mut bos_session_template_image_name = bos_session_template_yaml["image"]
            .as_str()
            .unwrap_or("")
            .to_string();

        bos_session_template_image_name = bos_session_template_image_name.replace("__DATE__", tag);

        // Get CFS configuration to configure the nodes
        let bos_session_template_configuration_name = bos_session_template_yaml["configuration"]
            .as_str()
            .unwrap()
            .to_string()
            .replace("__DATE__", tag);

        let mut cfs_configuration_value_vec = mesa::cfs::configuration::shasta::http_client::get(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            Some(&bos_session_template_configuration_name),
        )
        .await
        .unwrap();

        mesa::cfs::configuration::shasta::http_client::filter(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            &mut cfs_configuration_value_vec,
            Some(hsm_group_available_vec),
            None,
            None,
        ).await;

        if cfs_configuration_value_vec.is_empty() {
            eprintln!(
                "ERROR: BOS session template configuration not found in SAT file image list."
            );
            std::process::exit(1);
        }

        // Get base image details
        let image_detail_vec = mesa::ims::image::http_client::get_fuzzy(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            hsm_group_available_vec,
            Some(&bos_session_template_image_name),
            Some(&1),
        )
        .await
        .unwrap_or(Vec::new());

        log::info!("Image name: {}", image_detail_vec.first().unwrap().0.name);

        let ims_image_name = image_detail_vec.first().unwrap().0.name.to_string();
        let ims_image_etag = image_detail_vec
            .first()
            .unwrap()
            .0
            .link
            .as_ref()
            .unwrap()
            .etag
            .as_ref()
            .unwrap();
        let ims_image_path = &image_detail_vec
            .first()
            .unwrap()
            .0
            .link
            .as_ref()
            .unwrap()
            .path;
        let ims_image_type = &image_detail_vec
            .first()
            .unwrap()
            .0
            .link
            .as_ref()
            .unwrap()
            .r#type;

        let bos_session_template_name = bos_session_template_yaml["name"]
            .as_str()
            .unwrap_or("")
            .to_string()
            .replace("__DATE__", tag);

        let bos_session_template_hsm_groups: Vec<String> = if let Some(boot_sets_compute) =
            bos_session_template_yaml["bos_parameters"]["boot_sets"].get("compute")
        {
            boot_sets_compute["node_groups"]
                .as_sequence()
                .unwrap_or(&vec![])
                .iter()
                .map(|node| node.as_str().unwrap().to_string())
                .collect()
        } else if let Some(boot_sets_compute) =
            bos_session_template_yaml["bos_parameters"]["boot_sets"].get("uan")
        {
            boot_sets_compute["node_groups"]
                .as_sequence()
                .unwrap_or(&vec![])
                .iter()
                .map(|node| node.as_str().unwrap().to_string())
                .collect()
        } else {
            println!("No HSM group found in session_templates section in SAT file");
            std::process::exit(1);
        };

        // Check HSM groups in YAML file session_templates.bos_parameters.boot_sets.compute.node_groups matches with
        // Check hsm groups in SAT file includes the hsm_group_param
        let hsm_group = if hsm_group_param_opt.is_some()
            && !bos_session_template_hsm_groups
                .iter()
                .any(|h_g| h_g.eq(hsm_group_param_opt.unwrap()))
        {
            eprintln!("HSM group in param does not matches with any HSM groups in SAT file under session_templates.bos_parameters.boot_sets.compute.node_groups section. Using HSM group in param as the default");
            hsm_group_param_opt.unwrap()
        } else {
            bos_session_template_hsm_groups.first().unwrap()
        };

        let create_bos_session_template_payload =
            mesa::bos::template::mesa::r#struct::request_payload::BosSessionTemplate::new_for_hsm_group(
                bos_session_template_configuration_name,
                bos_session_template_name,
                ims_image_name,
                ims_image_path.to_string(),
                ims_image_type.to_string(),
                ims_image_etag.to_string(),
                hsm_group,
            );

        let create_bos_session_template_resp = mesa::bos::template::shasta::http_client::post(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            &create_bos_session_template_payload,
        )
        .await;

        if create_bos_session_template_resp.is_err() {
            eprintln!("BOS session template creation failed");
            std::process::exit(1);
        }

        // Create BOS session. Note: reboot operation shuts down the nodes and they may not start
        // up... hence we will split the reboot into 2 operations shutdown and start

        // Get nodes members of HSM group
        // Get HSM group details
        let hsm_group_details = hsm::http_client::get_hsm_group(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            hsm_group,
        )
        .await;

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
            shasta_root_cert,
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
        let create_bos_boot_session_resp = mesa::bos::session::shasta::http_client::post(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
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
}
