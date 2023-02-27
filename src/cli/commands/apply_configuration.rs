use clap::ArgMatches;

use crate::shasta::cfs::configuration;
use serde_yaml::Value;
use std::path::PathBuf;

pub async fn exec(
    hsm_group: Option<&String>,
    cli_apply_configuration: &ArgMatches,
    shasta_token: &String,
    shasta_base_url: &String,
    gitea_token: &String,
    gitea_base_url: &String,
) {
    // * Parse input params
    let path_buf: &PathBuf = cli_apply_configuration.get_one("file").unwrap();
    let file_content = std::fs::read_to_string(path_buf.file_name().unwrap()).unwrap();
    let sat_input_file_yaml: Value = serde_yaml::from_str(&file_content).unwrap();

    //    let repos: Vec<PathBuf> = cli_apply_configuration
    //        .get_many("repo-path")
    //        .unwrap()
    //        .cloned()
    //        .collect();

    // Parse hsm group
    let mut hsm_group_value = None;

    // Get hsm_group from cli arg
    if cli_apply_configuration
        .get_one::<String>("hsm-group")
        .is_some()
    {
        hsm_group_value = cli_apply_configuration.get_one::<String>("hsm-group");
    }

    // Get hsm group from config file
    if hsm_group.is_some() {
        hsm_group_value = hsm_group;
    }

    let mut cfs_configuration = configuration::CfsConfiguration::new();

    println!("\n### sat_input_file_yaml:\n{:#?}", sat_input_file_yaml);

    let configurations = sat_input_file_yaml["configurations"].as_sequence().unwrap();
    println!("\n### configurations:\n{:#?}", configurations);

    for configuration in configurations {
        println!(
            "### configuration name: {:#?}",
            configuration["name"].as_str().unwrap()
        );
        for layer_json in configuration["layers"].as_sequence().unwrap() {
            println!("layer: {:#?}", layer_json);

            if layer_json.get("git").is_some() {
                // Git layer
                let layer = configuration::Layer::new(
                    layer_json["git"]["url"].as_str().unwrap().to_string(),
                    None,
                    layer_json["name"].as_str().unwrap().to_string(),
                    layer_json["playbook"].as_str().unwrap().to_string(),
                    Some(layer_json["git"]["branch"].as_str().unwrap().to_string()),
                );
                cfs_configuration.add_layer(layer);
            } else {
                // Product layer
                let git_repo_url = "".to_string();
                let layer = configuration::Layer::new(
                    git_repo_url,
                    None,
                    layer_json["name"].as_str().unwrap().to_string(),
                    layer_json["playbook"].as_str().unwrap().to_string(),
                    Some(layer_json["git"]["branch"].as_str().unwrap().to_string()),
                );
                cfs_configuration.add_layer(layer);
            }
        }
    }

    println!("\ncfs_configuration:\n{:#?}", cfs_configuration);

    let images = sat_input_file_yaml["images"].as_sequence().unwrap();

    println!("\n### images:\n{:#?}", images);

    for image in images {
        println!("### image name: {:#?}", image["name"].as_str().unwrap());
    }
}
