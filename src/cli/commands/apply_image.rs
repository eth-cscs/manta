use std::{collections::HashMap, path::PathBuf};

use mesa::{
    cfs::configuration::mesa::r#struct::cfs_configuration_response::{
        ApiError, CfsConfigurationResponse,
    },
    common::kubernetes,
};
use serde_yaml::Value;

use crate::common::{
    self,
    sat_file::{self, import_images_section_in_sat_file},
};

/// Creates a CFS configuration and a CFS session from a CSCS SAT file.
/// Note: this method will fail if session name collide. This case happens if the __DATE__
/// placeholder is missing in the session name
/// Return a tuple (<cfs configuration name>, <cfs session name>)
pub async fn exec(
    vault_base_url: &str,
    vault_secret_path: &str,
    vault_role_id: &str,
    sat_file_path: &PathBuf,
    values_file_path_opt: Option<&PathBuf>,
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    ansible_verbosity_opt: Option<u8>,
    ansible_passthrough_opt: Option<&String>,
    watch_logs_opt: Option<&bool>,
    // tag: &str,
    hsm_group_available_vec: &[String],
    k8s_api_url: &str,
    gitea_token: &str,
    output_opt: Option<&String>,
) {
    let sat_file_content: String =
        std::fs::read_to_string(sat_file_path).expect("SAT file not found. Exit");

    let values_file_content_opt = values_file_path_opt
        .and_then(|values_file_path| std::fs::read_to_string(values_file_path).ok());

    let sat_file_yaml: Value = sat_file::render_jinja2_sat_file_yaml(
        &sat_file_content,
        values_file_content_opt.as_ref(),
        Some(Vec::new()),
    );

    /* let file_content = std::fs::read_to_string(sat_file_path).expect("SAT file not found. Exit");

    let sat_file_yaml: Value = if let Some(session_vars_file_path) = values_file_path_opt {
        log::info!("'Session vars' file provided. Going to process SAT file as a template.");
        // TEMPLATE
        // Read sesson vars file
        let session_vars_file_content = std::fs::read_to_string(session_vars_file_path).unwrap();
        let session_vars_file_yaml: Value =
            serde_yaml::from_str(&session_vars_file_content).unwrap();

        // Render SAT file template
        let env = minijinja::Environment::new();
        let sat_file_rendered = env
            .render_str(&file_content, session_vars_file_yaml)
            .unwrap();

        log::debug!("SAT file rendered:\n:{}", sat_file_rendered);

        serde_yaml::from_str::<Value>(&sat_file_rendered).unwrap()
    } else {
        serde_yaml::from_str(&file_content).unwrap()
    }; */

    // let sat_file_yaml: Value = serde_yaml::from_str(&file_content).unwrap();

    // VALIDATION - WE WON'T PROCESS ANYTHING IF THE USER DOES NOT HAVE ACCESS TO ANY HSM GROUP
    // DEFINED IN THE SAT FILE

    // Get CFS configurations from SAT YAML file
    let configuration_yaml_vec_opt = sat_file_yaml["configurations"].as_sequence();

    // Get inages from SAT YAML file
    let image_yaml_vec_opt: Option<&Vec<Value>> = sat_file_yaml["images"].as_sequence();

    // Get inages from SAT YAML file
    let bos_session_template_list_yaml = sat_file_yaml["session_templates"].as_sequence();

    if bos_session_template_list_yaml.is_some() {
        log::warn!(
            "SAT file has data in session_template section. This information will be ignored."
        )
    }

    // Check HSM groups in images section in SAT file matches the HSM group in JWT (keycloak roles)
    validate_sat_file_images_section(image_yaml_vec_opt, hsm_group_available_vec);

    let shasta_k8s_secrets = crate::common::vault::http_client::fetch_shasta_k8s_secrets(
        vault_base_url,
        vault_secret_path,
        vault_role_id,
    )
    .await;

    let kube_client = kubernetes::get_k8s_client_programmatically(k8s_api_url, shasta_k8s_secrets)
        .await
        .unwrap();
    let cray_product_catalog = kubernetes::get_configmap(kube_client, "cray-product-catalog")
        .await
        .unwrap();

    let mut cfs_configuration_hashmap = HashMap::new();

    for configuration_yaml in configuration_yaml_vec_opt.unwrap_or(&Vec::new()) {
        let cfs_configuration_rslt: Result<CfsConfigurationResponse, ApiError> =
            common::sat_file::create_cfs_configuration_from_sat_file(
                shasta_token,
                shasta_base_url,
                shasta_root_cert,
                gitea_token,
                &cray_product_catalog,
                configuration_yaml,
                // tag,
            )
            .await;

        let cfs_configuration = match cfs_configuration_rslt {
            Ok(cfs_configuration) => cfs_configuration,
            Err(error) => {
                eprintln!("{}", error);
                std::process::exit(1);
            }
        };

        cfs_configuration_hashmap.insert(cfs_configuration.name.clone(), cfs_configuration.clone());
    }

    // List of image.ref_name already processed
    let mut ref_name_processed_hashmap: HashMap<String, String> = HashMap::new();

    let cfs_session_created_hashmap: HashMap<String, serde_yaml::Value> =
        import_images_section_in_sat_file(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            &mut ref_name_processed_hashmap,
            image_yaml_vec_opt.unwrap_or(&Vec::new()).to_vec(),
            &cray_product_catalog,
            ansible_verbosity_opt,
            ansible_passthrough_opt,
            // tag,
        )
        .await;

    println!(
        "List of new image IDs: {:#?}",
        cfs_session_created_hashmap.keys().collect::<Vec<&String>>()
    );
}

pub fn validate_sat_file_images_section(
    image_yaml_vec_opt: Option<&Vec<Value>>,
    hsm_group_available_vec: &[String],
) {
    // Check HSM groups in images section in SAT file matches the HSM group in JWT (keycloak roles)
    for image_yaml_vec in image_yaml_vec_opt.unwrap_or(&Vec::new()) {
        for hsm_group in image_yaml_vec["configuration_group_names"]
            .as_sequence()
            .unwrap_or(&Vec::new())
            .iter()
            .map(|hsm_group_yaml| hsm_group_yaml.as_str().unwrap())
            .filter(|&hsm_group| {
                !hsm_group.eq_ignore_ascii_case("Compute")
                    && !hsm_group.eq_ignore_ascii_case("Application")
                    && !hsm_group.eq_ignore_ascii_case("Application_UAN")
            })
        {
            if !hsm_group_available_vec.contains(&hsm_group.to_string()) {
                println!(
                        "HSM group '{}' in image {} not allowed, List of HSM groups available {:?}. Exit",
                        hsm_group,
                        image_yaml_vec["name"].as_str().unwrap(),
                        hsm_group_available_vec
                    );
                std::process::exit(-1);
            }
        }
    }
}
