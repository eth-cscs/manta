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

    for configuration_yaml in configurations_yaml {
        let mut cfs_configuration = configuration::CfsConfiguration::new();
        let configuration_name = configuration_yaml["name"]
            .as_str()
            .unwrap()
            .to_string()
            .replace(
                "__DATE__",
                &chrono::Utc::now().format("%Y%m%d%H%M%S").to_string(),
            );
        for layer_yaml in configuration_yaml["layers"].as_sequence().unwrap() {
            // println!("\n\n### Layer:\n{:#?}\n", layer_json);

            if layer_yaml.get("git").is_some() {
                // Git layer
                let repo_name = layer_yaml["name"].as_str().unwrap().to_string();
                let repo_url = layer_yaml["git"]["url"].as_str().unwrap().to_string();
                let layer = configuration::Layer::new(
                    repo_url,
                    // Some(layer_json["git"]["commit"].as_str().unwrap_or_default().to_string()),
                    None,
                    repo_name,
                    layer_yaml["playbook"]
                        .as_str()
                        .unwrap_or_default()
                        .to_string(),
                    Some(
                        layer_yaml["git"]["branch"]
                            .as_str()
                            .unwrap_or_default()
                            .to_string(),
                    ),
                );
                cfs_configuration.add_layer(layer);
            } else {
                // Product layer
                let repo_url = format!(
                    "https://api-gw-service-nmn.local/vcs/cray/{}-config-management.git",
                    layer_yaml["name"].as_str().unwrap()
                );
                let layer = configuration::Layer::new(
                    repo_url,
                    // Some(layer_json["product"]["commit"].as_str().unwrap_or_default().to_string()),
                    None,
                    layer_yaml["product"]["name"]
                        .as_str()
                        .unwrap_or_default()
                        .to_string(),
                    layer_yaml["playbook"].as_str().unwrap().to_string(),
                    Some(
                        layer_yaml["product"]["branch"]
                            .as_str()
                            .unwrap_or_default()
                            .to_string(),
                    ),
                );
                cfs_configuration.add_layer(layer);
            }
        }

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
}
