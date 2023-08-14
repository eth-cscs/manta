use std::path::PathBuf;

use futures_util::TryStreamExt;
use mesa::shasta::cfs::{
    self,
    configuration::{self, CfsConfiguration},
    session::CfsSession,
};
use serde_yaml::Value;

use crate::{cli, common::jwt_ops::get_claims_from_jwt_token};

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
    watch_logs: Option<&bool>,
    tag: &str,
    hsm_group_config: Option<&String>,
    k8s_api_url: &str,
) -> (Vec<CfsConfiguration>, Vec<CfsSession>) {
    let mut cfs_configuration;

    // let path_file: &PathBuf = cli_apply_image.get_one("file").unwrap();
    let file_content = std::fs::read_to_string(path_file).unwrap();
    let sat_file_yaml: Value = serde_yaml::from_str(&file_content).unwrap();

    // Get CFS configurations from SAT YAML file
    let configuration_list_yaml = sat_file_yaml["configurations"].as_sequence();

    // Get CFS images from SAT YAML file
    let image_list_yaml = sat_file_yaml["images"].as_sequence();

    // Used to uniquely identify cfs configuration name and cfs session name. This process follows
    // what the CSCS build script is doing. We need to do this since we are using CSCS SAT file
    // let timestamp = chrono::Utc::now().format("%Y%m%d%H%M%S").to_string();

    let empty_vec = &Vec::new();
    let configuration_yaml_list = configuration_list_yaml.unwrap_or(empty_vec);

    let mut cfs_configuration_name_list = Vec::new();

    let mut cfs_session_name_list = Vec::new();

    for configuration_yaml in configuration_yaml_list {
        cfs_configuration =
            configuration::CfsConfiguration::from_sat_file_serde_yaml(configuration_yaml);

        // Rename configuration name
        cfs_configuration.name = cfs_configuration.name.replace("__DATE__", tag);

        log::debug!(
            "CFS configuration creation payload:\n{:#?}",
            cfs_configuration
        );

        let create_cfs_configuration_resp = cfs::configuration::http_client::put(
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

        cfs_configuration_name_list.push(cfs_configuration.clone());

        println!("CFS configuration created: {}", cfs_configuration.name);
    }

    for image_yaml in image_list_yaml.unwrap_or(empty_vec) {

        let mut cfs_session = CfsSession::from_sat_file_serde_yaml(image_yaml);

        // Rename session name
        cfs_session.name = cfs_session.name.replace("__DATE__", tag);

        // Rename session configuration name
        cfs_session.configuration_name = cfs_session.configuration_name.replace("__DATE__", tag);

        log::debug!("CFS session creation payload:\n{:#?}", cfs_session);

        let create_cfs_session_resp =
            cfs::session::http_client::post(shasta_token, shasta_base_url, &cfs_session).await;

        log::debug!(
            "CFS session creation response:\n{:#?}",
            create_cfs_session_resp
        );

        if create_cfs_session_resp.is_err() {
            eprintln!("CFS session creation failed");
            eprintln!("Reason:\n{:#?}", create_cfs_session_resp);
            std::process::exit(1);
        }

        cfs_session_name_list.push(cfs_session.clone());

        println!("CFS session created: {}", cfs_session.name);

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
                print!("{}", std::str::from_utf8(&line).unwrap());
            }
        }
    }

    (cfs_configuration_name_list, cfs_session_name_list)
}
