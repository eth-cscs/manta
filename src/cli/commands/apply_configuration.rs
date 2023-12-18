use mesa::cfs::configuration::shasta::r#struct::configuration::CfsConfigurationRequest;
use serde_yaml::Value;
use std::path::PathBuf;

use crate::common::cfs_configuration_utils;

/// Creates a configuration from a sat file
/// NOTE: this method manages 2 types of methods [git, product]. For type product, the name must
/// match with a git repo name after concatenating it with "-config-management" (eg: layer name
/// "cos" becomes repo name "cos-config-management" which correlates with https://api-gw-service-nmn.local/vcs/api/v1/repos/cray/cos-config-management)
/// Return CFS configuration name
pub async fn exec(
    path_file: &PathBuf,
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    tag: &str,
    output_opt: Option<&String>,
) -> anyhow::Result<Vec<String>> {
    let file_content = std::fs::read_to_string(path_file).expect("SAT file not found. Exit");
    let sat_file_yaml: Value = serde_yaml::from_str(&file_content).unwrap();

    let mut cfs_configuration_value_vec = Vec::new();

    // Get CFS configurations from SAT YAML file
    let configuration_yaml_vec_opt = sat_file_yaml["configurations"].as_sequence();

    // Get inages from SAT YAML file
    let image_yaml_vec_opt = sat_file_yaml["images"].as_sequence();

    // Get inages from SAT YAML file
    let bos_session_template_list_yaml = sat_file_yaml["session_templates"].as_sequence();

    if configuration_yaml_vec_opt.is_none() {
        eprintln!("No configuration found in SAT file. Exit");
        std::process::exit(1);
    }

    if image_yaml_vec_opt.is_some() {
        log::warn!("SAT file has data in images section. This information will be ignored.")
    }

    if bos_session_template_list_yaml.is_some() {
        log::warn!(
            "SAT file has data in session_template section. This information will be ignored."
        )
    }

    let empty_vec = &Vec::new();
    let configuration_yaml_vec = configuration_yaml_vec_opt.unwrap_or(empty_vec);

    let mut cfs_configuration_name_vec = Vec::new();

    for configuration_yaml in configuration_yaml_vec {
        let mut cfs_configuration_value =
            CfsConfigurationRequest::from_sat_file_serde_yaml(configuration_yaml);

        // Rename configuration name
        cfs_configuration_value.name = cfs_configuration_value.name.replace("__DATE__", tag);

        log::debug!(
            "CFS configuration creation payload:\n{:#?}",
            cfs_configuration_value
        );

        let cfs_configuration_value_rslt = mesa::cfs::configuration::shasta::http_client::put(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            &cfs_configuration_value,
            &cfs_configuration_value.name,
        )
        .await;

        log::debug!(
            "CFS configuration creation response:\n{:#?}",
            cfs_configuration_value_rslt
        );

        let cfs_configuration_value =
            if let Ok(cfs_configuration_value) = cfs_configuration_value_rslt {
                cfs_configuration_value
            } else {
                eprintln!("CFS configuration creation failed");
                std::process::exit(1);
            };

        let cfs_configuration_name = cfs_configuration_value["name"]
            .as_str()
            .unwrap()
            .to_string();

        cfs_configuration_name_vec.push(cfs_configuration_name.clone());

        log::info!("CFS configuration created: {}", cfs_configuration_name);

        cfs_configuration_value_vec.push(cfs_configuration_value.clone());

        // Print output
        if output_opt.is_some() && output_opt.unwrap().eq("json") {
            println!(
                "{}",
                serde_json::to_string_pretty(&cfs_configuration_value).unwrap()
            );
        } else {
            cfs_configuration_utils::print_table_value(&cfs_configuration_value_vec);
        }
    }

    Ok(cfs_configuration_name_vec)
}
