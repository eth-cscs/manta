use std::path::PathBuf;

use futures::TryStreamExt;

use mesa::{
    mesa::cfs::{
        configuration::{get_put_payload::CfsConfigurationResponse, http_client::http_client::put},
        session::{get_response_struct::CfsSessionGetResponse, http_client::http_client::post},
    },
    shasta::cfs::{configuration::CfsConfigurationRequest, session::CfsSessionRequest},
};
use serde_yaml::Value;

use crate::{
    cli,
    common::{cfs_session_utils, jwt_ops::get_claims_from_jwt_token},
};

/// Creates a CFS configuration and a CFS session from a CSCS SAT file.
/// Note: this method will fail if session name collide. This case happens if the __DATE__
/// placeholder is missing in the session name
/// Return a tuple (<cfs configuration name>, <cfs session name>)
pub async fn exec(
    vault_base_url: &str,
    vault_secret_path: &str,
    vault_role_id: &str,
    path_file: &PathBuf,
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    ansible_verbosity: Option<&String>,
    ansible_passthrough: Option<&String>,
    watch_logs: Option<&bool>,
    tag: &str,
    hsm_group_available_vec_opt: Option<&[String]>,
    k8s_api_url: &str,
    output_opt: Option<&String>,
) -> (Vec<CfsConfigurationResponse>, Vec<CfsSessionGetResponse>) {
    let file_content = std::fs::read_to_string(path_file).expect("SAT file not found. Exit");
    let sat_file_yaml: Value = serde_yaml::from_str(&file_content).unwrap();

    // VALIDATION - WE WON'T PROCESS ANYTHING IF THE USER DOES NOT HAVE ACCESS TO ANY HSM GROUP
    // DEFINED IN THE SAT FILE
    // Get CFS images from SAT YAML file
    let image_yaml_list = sat_file_yaml["images"].as_sequence();

    // Check HSM groups in images section in SAT file matches the HSM group in Manta configuration file
    if let Some(hsm_group_available_vec) = hsm_group_available_vec_opt {
        for image_yaml in image_yaml_list.unwrap_or(&Vec::new()) {
            for hsm_group in image_yaml["configuration_group_names"]
                .as_sequence()
                .unwrap()
                .iter()
                .map(|hsm_group_yaml| hsm_group_yaml.as_str().unwrap())
                .filter(|&hsm_group| {
                    !hsm_group.eq_ignore_ascii_case("Compute")
                        && !hsm_group.eq_ignore_ascii_case("Application")
                        && !hsm_group.eq_ignore_ascii_case("Application_UAN")
                })
            {
                if !hsm_group_available_vec.contains(&hsm_group.to_string()) {
                    println!(
                        "HSM group '{}' in image {} not allowed, List of HSM groups available {:?}. Exit",
                        hsm_group,
                        image_yaml["name"].as_str().unwrap(),
                        hsm_group_available_vec
                    );
                    std::process::exit(-1);
                }
            }
        }
    } else {
        println!("Your user does not have HSM groups assigned in keycloak. Exit");
        std::process::exit(1);
    }

    // Process CFS configurations
    let mut cfs_configuration;

    // Get CFS configurations from SAT YAML file
    let configuration_list_yaml = sat_file_yaml["configurations"].as_sequence();

    let empty_vec = &Vec::new();
    let configuration_yaml_list = configuration_list_yaml.unwrap_or(empty_vec);

    let mut cfs_configuration_vec = Vec::new();

    for configuration_yaml in configuration_yaml_list {
        cfs_configuration = CfsConfigurationRequest::from_sat_file_serde_yaml(configuration_yaml);

        // Rename configuration name
        cfs_configuration.name = cfs_configuration.name.replace("__DATE__", tag);

        log::debug!(
            "CFS configuration creation payload:\n{:#?}",
            cfs_configuration
        );

        let create_cfs_configuration_resp = put(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            &cfs_configuration,
            &cfs_configuration.name,
        )
        .await;

        log::debug!(
            "CFS configuration creation response:\n{:#?}",
            create_cfs_configuration_resp
        );

        if create_cfs_configuration_resp.is_err() {
            eprintln!("CFS configuration creation failed");
            std::process::exit(1);
        }

        cfs_configuration_vec.push(create_cfs_configuration_resp.unwrap());

        log::info!("CFS configuration created: {}", cfs_configuration.name);
    }

    // Process CFS sessions
    let mut cfs_session_resp_list = Vec::new();

    for image_yaml in image_yaml_list.unwrap_or(empty_vec) {
        let mut cfs_session = CfsSessionRequest::from_sat_file_serde_yaml(image_yaml);

        // Rename session name
        cfs_session.name = cfs_session.name.replace("__DATE__", tag);

        // Rename session's configuration name
        cfs_session.configuration_name = cfs_session.configuration_name.replace("__DATE__", tag);


        // Set ansible verbosity
        cfs_session.ansible_verbosity = Some(
            ansible_verbosity
                .cloned()
                .unwrap_or("0".to_string())
                .parse::<u8>()
                .unwrap(),
        );

        // Set ansible passthrough params
        cfs_session.ansible_passthrough = ansible_passthrough.cloned();

        log::debug!("CFS session creation payload:\n{:#?}", cfs_session);

        let create_cfs_session_resp = post(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            &cfs_session,
        )
        .await;

        log::debug!(
            "CFS session creation response:\n{:#?}",
            create_cfs_session_resp
        );

        if create_cfs_session_resp.is_err() {
            eprintln!("CFS session creation failed");
            eprintln!("Reason:\n{:#?}", create_cfs_session_resp);
            std::process::exit(1);
        }

        cfs_session_resp_list.push(create_cfs_session_resp.unwrap());

        // cfs_session_name_list.push(cfs_session.clone());

        log::info!("CFS session created: {}", cfs_session.name);

        // Print output
        if output_opt.is_some() && output_opt.unwrap().eq("json") {
            println!(
                "{}",
                serde_json::to_string_pretty(&cfs_session_resp_list).unwrap()
            );
        } else {
            cfs_session_utils::print_table_struct(&cfs_session_resp_list);
        }

        // Audit to file
        let jwt_claims = get_claims_from_jwt_token(shasta_token).unwrap();

        log::info!(target: "app::audit", "User: {} ({}) ; Operation: Apply image", jwt_claims["name"].as_str().unwrap(), jwt_claims["preferred_username"].as_str().unwrap());

        if let Some(true) = watch_logs {
            log::info!("Fetching logs ...");

            let mut logs_stream = cli::commands::log::session_logs(
                vault_base_url,
                vault_secret_path,
                vault_role_id,
                &cfs_session.name,
                None,
                k8s_api_url,
            )
            .await
            .unwrap();

            while let Some(line) = logs_stream.try_next().await.unwrap() {
                println!("{}", line);
            }
        }
    }

    (cfs_configuration_vec, cfs_session_resp_list)
}
