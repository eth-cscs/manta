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
    let sat_input_file_yaml: Value = serde_yaml::from_str(&file_content).unwrap();

    let mut cfs_configuration = configuration::CfsConfiguration::new();

    // println!("\n### sat_input_file_yaml:\n{:#?}", sat_input_file_yaml);

    let configurations = sat_input_file_yaml["configurations"].as_sequence().unwrap();
    // println!("\n### configurations:\n{:#?}", configurations);

    for configuration in configurations {
        let configuration_name = configuration["name"].as_str().unwrap().to_string().replace(
            "__DATE__",
            &chrono::Utc::now().format("%Y%m%d%H%M%S").to_string(),
        );
        for layer_json in configuration["layers"].as_sequence().unwrap() {
            // println!("\n\n### Layer:\n{:#?}\n", layer_json);

            if layer_json.get("git").is_some() {
                // Git layer
                let repo_name = layer_json["name"].as_str().unwrap().to_string();
                let repo_url = layer_json["git"]["url"].as_str().unwrap().to_string();
                let layer = configuration::Layer::new(
                    repo_url,
                    // Some(layer_json["git"]["commit"].as_str().unwrap_or_default().to_string()),
                    None,
                    repo_name,
                    layer_json["playbook"]
                        .as_str()
                        .unwrap_or_default()
                        .to_string(),
                    Some(
                        layer_json["git"]["branch"]
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
                    layer_json["name"].as_str().unwrap()
                );
                let layer = configuration::Layer::new(
                    repo_url,
                    // Some(layer_json["product"]["commit"].as_str().unwrap_or_default().to_string()),
                    None,
                    layer_json["product"]["name"]
                        .as_str()
                        .unwrap_or_default()
                        .to_string(),
                    layer_json["playbook"].as_str().unwrap().to_string(),
                    Some(
                        layer_json["product"]["branch"]
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
