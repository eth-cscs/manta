use std::path::PathBuf;

use clap::ArgMatches;
use k8s_openapi::chrono;
use serde_yaml::Value;

use crate::shasta::cfs::{configuration, session::CfsSession};

pub async fn exec(
    vault_base_url: &String,
    cli_apply_image: &ArgMatches,
    shasta_token: &String,
    shasta_base_url: &String,
    base_image_id: &String,
    // hsm_group: Option<&String>
) {
    let mut cfs_configuration;

    let path_buf: &PathBuf = cli_apply_image.get_one("file").unwrap();
    let file_content = std::fs::read_to_string(path_buf).unwrap();
    let sat_file_yaml: Value = serde_yaml::from_str(&file_content).unwrap();

    // println!("\n### sat_input_file_yaml:\n{:#?}", sat_input_file_yaml);

    let configurations_yaml = sat_file_yaml["configurations"].as_sequence().unwrap();
    // println!("\n### configurations:\n{:#?}", configurations);

    if configurations_yaml.is_empty() {
        eprintln!("The input file has no configurations!");
        std::process::exit(-1);
    }

    if configurations_yaml.len() > 1 {
        eprintln!("Multiple CFS configurations found in input file, please clean the file so it only contains one.");
        std::process::exit(-1);
    }

    // Used to uniquely identify cfs configuration name and cfs session name. This process follows
    // what the CSCS build script is doing. We need to do this since we are using CSCS SAT file
    let timestamp = chrono::Utc::now().format("%Y%m%d%H%M%S").to_string();

    let configuration_yaml = &configurations_yaml[0];

    cfs_configuration =
        configuration::CfsConfiguration::from_sat_file_serde_yaml(&configuration_yaml);

    // Rename configuration name
    cfs_configuration.name = cfs_configuration.name.replace("__DATE__", &timestamp);

    log::info!(
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

    log::info!(
        "CFS configuration creation response:\n{:#?}",
        create_cfs_configuration_resp
    );

    if create_cfs_configuration_resp.is_err() {
        log::error!("CFS configuration creation failed");
        std::process::exit(1);
    }

    let images_yaml = sat_file_yaml["images"].as_sequence().unwrap();

    let mut cfs_session = CfsSession::from_sat_file_serde_yaml(&images_yaml[0], base_image_id);

    // Rename session name
    cfs_session.name = cfs_session.name.replace("__DATE__", &timestamp);

    // Rename session configuration name
    cfs_session.configuration_name = cfs_configuration.name.clone();

    log::info!("CFS session payload:\n{:#?}", cfs_session);

    let create_cfs_session_resp =
        crate::shasta::cfs::session::http_client::post(shasta_token, shasta_base_url, &cfs_session)
            .await;

    log::info!(
        "CFS creation session response: {:#?}",
        create_cfs_session_resp
    );

    if create_cfs_session_resp.is_err() {
        log::error!("CFS session creation failed");
        std::process::exit(1);
    }

    let watch_logs = cli_apply_image.get_one::<bool>("watch-logs");

    if let Some(true) = watch_logs {
        log::info!("Fetching logs ...");
        crate::cli::commands::log::session_logs(vault_base_url, &cfs_session.name, None)
            .await
            .unwrap();
    }

    let watch_logs = cli_apply_image.get_one::<bool>("watch-logs");

    if let Some(true) = watch_logs {
        log::info!("Fetching logs ...");
        crate::cli::commands::log::session_logs(vault_base_url, &cfs_session.name, None)
            .await
            .unwrap();
    }
}
