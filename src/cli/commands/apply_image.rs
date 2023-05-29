use std::path::PathBuf;

use futures_util::TryStreamExt;
use serde_yaml::Value;

use crate::{
    cli,
    common::jwt_ops::get_claims_from_jwt_token,
    shasta::cfs::{configuration, session::CfsSession},
};

/// Creates a CFS configuration and a CFS session from a CSCS SAT file.
/// Note: this method will fail if session name collide. This case happens if the __DATE__
/// placeholder is missing in the session name
/// Return a tuple (<cfs configuration name>, <cfs session name>)
pub async fn exec(
    vault_base_url: &str,
    vault_role_id: &str,
    // cli_apply_image: &ArgMatches,
    path_file: &PathBuf,
    shasta_token: &str,
    shasta_base_url: &str,
    // base_image_id: &str,
    watch_logs: Option<&bool>,
    timestamp: &str,
    hsm_group_config: Option<&String>,
    k8s_api_url: &str,
) -> (String, String) {
    let mut cfs_configuration;

    // let path_file: &PathBuf = cli_apply_image.get_one("file").unwrap();
    let file_content = std::fs::read_to_string(path_file).unwrap();
    let sat_file_yaml: Value = serde_yaml::from_str(&file_content).unwrap();

    // Get CFS configurations from SAT YAML file
    let configurations_yaml = sat_file_yaml["configurations"].as_sequence().unwrap();

    if configurations_yaml.is_empty() {
        eprintln!("The input file has no configurations!");
        std::process::exit(-1);
    }

    if configurations_yaml.len() > 1 {
        eprintln!("Multiple CFS configurations found in input file, please clean the file so it only contains one.");
        std::process::exit(-1);
    }

    // Get CFS images from SAT YAML file
    let images_yaml = sat_file_yaml["images"].as_sequence().unwrap();

    // Check HSM groups in images section matches the HSM group in Manta configuration file
    if let Some(hsm_group_config_value) = hsm_group_config {
        let hsm_group_images: Vec<String> = images_yaml
            .iter()
            .flat_map(|image_yaml| {
                image_yaml["configuration_group_names"]
                    .as_sequence()
                    .unwrap()
                    .iter()
                    .map(|configuration_group_name| {
                        configuration_group_name
                            .as_str()
                            .unwrap_or_default()
                            .to_string()
                    })
            })
            .collect();
        if !hsm_group_images.contains(hsm_group_config_value) {
            eprintln!("HSM group in configuration does not match with the one in SAT file images.configuration_group_names values");
            std::process::exit(1);
        }
    }

    // Used to uniquely identify cfs configuration name and cfs session name. This process follows
    // what the CSCS build script is doing. We need to do this since we are using CSCS SAT file
    // let timestamp = chrono::Utc::now().format("%Y%m%d%H%M%S").to_string();

    let configuration_yaml = &configurations_yaml[0];

    cfs_configuration =
        configuration::CfsConfiguration::from_sat_file_serde_yaml(configuration_yaml);

    // Rename configuration name
    cfs_configuration.name = cfs_configuration.name.replace("__DATE__", timestamp);

    log::debug!(
        "CFS configuration creation payload:\n{:#?}",
        cfs_configuration
    );

    let create_cfs_configuration_resp = crate::shasta::cfs::configuration::http_client::put(
        shasta_token,
        shasta_base_url,
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

    println!("CFS configuration created: {}", cfs_configuration.name);

    let mut cfs_session = CfsSession::from_sat_file_serde_yaml(&images_yaml[0]);

    // Rename session name
    cfs_session.name = cfs_session.name.replace("__DATE__", timestamp);

    // Rename session configuration name
    cfs_session.configuration_name = cfs_configuration.name.clone();

    log::debug!("CFS session creation payload:\n{:#?}", cfs_session);

    let create_cfs_session_resp =
        crate::shasta::cfs::session::http_client::post(shasta_token, shasta_base_url, &cfs_session)
            .await;

    log::debug!(
        "CFS session creation response:\n{:#?}",
        create_cfs_session_resp
    );

    if create_cfs_session_resp.is_err() {
        eprintln!("CFS session creation failed");
        std::process::exit(1);
    }

    let cfs_session_name = create_cfs_session_resp.unwrap()["name"]
        .as_str()
        .unwrap()
        .to_string();

    // let watch_logs = cli_apply_image.get_one::<bool>("watch-logs");

    if let Some(true) = watch_logs {
        log::info!("Fetching logs ...");

        let mut logs_stream = cli::commands::log::session_logs(
            vault_base_url,
            vault_role_id,
            &cfs_session.name,
            None,
            k8s_api_url,
        )
        .await
        .unwrap();

        while let Some(line) = logs_stream.try_next().await.unwrap() {
            print!("{}", std::str::from_utf8(&line).unwrap());
        }
    }

    // let watch_logs = cli_apply_image.get_one::<bool>("watch-logs");

    /* if let Some(true) = watch_logs {
        log::info!("Fetching logs ...");
        crate::cli::commands::log::session_logs(vault_base_url, &cfs_session.name, None)
            .await
            .unwrap();
    } */

    // Audit
    let jwt_claims = get_claims_from_jwt_token(shasta_token).unwrap();

    log::info!(target: "app::audit", "User: {} ({}) ; Operation: Apply image", jwt_claims["name"].as_str().unwrap(), jwt_claims["preferred_username"].as_str().unwrap());

    println!("CFS session created: {}", cfs_session_name);

    (cfs_configuration.name, cfs_session_name)
}
