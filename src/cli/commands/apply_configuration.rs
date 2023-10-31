use mesa::{shasta::cfs::{self, configuration::CfsConfigurationRequest}, mesa::cfs::configuration::get_put_payload::CfsConfigurationResponse};
use serde_yaml::Value;
use std::path::Path;

/// Creates a configuration from a sat file
/// NOTE: this method manages 2 types of methods [git, product]. For type product, the name must
/// match with a git repo name after concatenating it with "-config-management" (eg: layer name
/// "cos" becomes repo name "cos-config-management" which correlates with https://api-gw-service-nmn.local/vcs/api/v1/repos/cray/cos-config-management)
/// Return CFS configuration name
pub async fn exec(
    path_file: &Path,
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    tag: &str,
) -> anyhow::Result<Vec<String>> {
    let file_content = std::fs::read_to_string(path_file).expect("SAT file not found. Exit");
    let sat_file_yaml: Value = serde_yaml::from_str(&file_content).unwrap();

    let mut cfs_configuration_vec = Vec::new();

    // Get CFS configurations from SAT YAML file
    let configuration_list_yaml = sat_file_yaml["configurations"].as_sequence();

    let empty_vec = &Vec::new();
    let configuration_yaml_list = configuration_list_yaml.unwrap_or(empty_vec);

    let mut cfs_configuration_name_list = Vec::new();

    for configuration_yaml in configuration_yaml_list {
        let mut cfs_configuration_request_payload =
            CfsConfigurationRequest::from_sat_file_serde_yaml(configuration_yaml);

        // Rename configuration name
        cfs_configuration_request_payload.name = cfs_configuration_request_payload.name.replace("__DATE__", tag);

        log::debug!(
            "CFS configuration creation payload:\n{:#?}",
            cfs_configuration_request_payload
        );

        let create_cfs_configuration_resp = cfs::configuration::http_client::put(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            &cfs_configuration_request_payload,
            &cfs_configuration_request_payload.name,
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

        cfs_configuration_name_list.push(create_cfs_configuration_resp.unwrap());

        log::info!("CFS configuration created: {}", cfs_configuration_request_payload.name);

        cfs_configuration_vec.push(cfs_configuration_request_payload.name);
    }

    Ok(cfs_configuration_vec)
}
