use clap::ArgMatches;
use k8s_openapi::chrono;

use crate::shasta::cfs::configuration;
use serde_yaml::Value;
use std::path::PathBuf;

/// Creates a configuration from a sat file
/// NOTE: this method manages 2 types of methods [git, product]. For type product, the name must
/// match with a git repo name after concatenating it with "-config-management" (eg: layer name
/// "cos" becomes repo name "cos-config-management" which correlates with https://api-gw-service-nmn.local/vcs/api/v1/repos/cray/cos-config-management)
/// Return CFS configuration name
pub async fn exec(
    cli_apply_configuration: &ArgMatches,
    shasta_token: &String,
    shasta_base_url: &String,
    timestamp: &String,
) -> String {
    // * Parse input params
    let path_buf: &PathBuf = cli_apply_configuration.get_one("file").unwrap();
    let file_content = std::fs::read_to_string(path_buf.file_name().unwrap()).unwrap();
    let sat_file_yaml: Value = serde_yaml::from_str(&file_content).unwrap();

    let configurations_yaml = sat_file_yaml["configurations"].as_sequence().unwrap();

    if configurations_yaml.is_empty() {
        eprintln!("The input file has no configurations!");
        std::process::exit(-1);
    }

    if configurations_yaml.len() > 1 {
        eprintln!("Multiple CFS configurations found in input file, please clean the file so it only contains one.");
        std::process::exit(-1);
    }

    let configuration_yaml = &configurations_yaml[0];

    let mut create_cfs_configuration_payload =
        configuration::CfsConfiguration::from_sat_file_serde_yaml(configuration_yaml);

    create_cfs_configuration_payload.name = create_cfs_configuration_payload
        .name
        .replace("__DATE__", timestamp);

    log::info!(
        "CFS configuration creation payload:\n{:#?}",
        create_cfs_configuration_payload
    );

    let create_cfs_configuration_resp = crate::shasta::cfs::configuration::http_client::put(
        shasta_token,
        shasta_base_url,
        &create_cfs_configuration_payload,
        &create_cfs_configuration_payload.name,
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

    let cfs_configuration_name = create_cfs_configuration_resp.unwrap()["name"]
        .as_str()
        .unwrap()
        .to_string();

    log::info!("CFS configuration name: {}", cfs_configuration_name);

    cfs_configuration_name
}
