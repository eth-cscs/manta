use clap::ArgMatches;
use k8s_openapi::chrono;

use crate::shasta::cfs::configuration;
use serde_yaml::Value;
use std::path::PathBuf;

/// Creates a configuration from a sat file
/// NOTE: this method manages 2 types of methods [git, product]. For type product, the name must
/// match with a git repo name after concatenating it with "-config-management" (eg: layer name
/// "cos" becomes repo name "cos-config-management" which correlates with https://api-gw-service-nmn.local/vcs/api/v1/repos/cray/cos-config-management)
pub async fn exec(
    cli_apply_configuration: &ArgMatches,
    shasta_token: &String,
    shasta_base_url: &String,
) {
    // * Parse input params
    let path_buf: &PathBuf = cli_apply_configuration.get_one("file").unwrap();
    let file_content = std::fs::read_to_string(path_buf.file_name().unwrap()).unwrap();
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

    let configuration_yaml = &configurations_yaml[0];

    let cfs_configuration =
        configuration::CfsConfiguration::from_sat_file_serde_yaml(configuration_yaml);

    let configuration_name = cfs_configuration.name.replace(
        "__DATE__",
        &chrono::Utc::now().format("%Y%m%d%H%M%S").to_string(),
    );

    log::debug!("{:#?}", cfs_configuration);

    // println!("\n### images:\n{:#?}", images);

    let configuration = crate::shasta::cfs::configuration::http_client::put(
        shasta_token,
        shasta_base_url,
        &cfs_configuration,
        &configuration_name,
    )
    .await
    .unwrap();

    println!(
        "{}",
        configuration["name"]
            .as_str()
            .unwrap_or_default()
            .to_string()
    );
}
