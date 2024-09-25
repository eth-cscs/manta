use std::collections::HashMap;

use dialoguer::theme::ColorfulTheme;
use mesa::{
    cfs::configuration::mesa::r#struct::cfs_configuration_response::v2::CfsConfigurationResponse,
    common::kubernetes, error::Error,
};
use serde_yaml::Value;

use crate::common::sat_file::sat_file_utils::{
    self, import_images_section_in_sat_file, validate_sat_file_images_section,
};

/// Creates a CFS configuration and a CFS session from a CSCS SAT file.
/// Note: this method will fail if session name collide. This case happens if the __DATE__
/// placeholder is missing in the session name
/// Return a tuple (<cfs configuration name>, <cfs session name>)
#[deprecated(since = "1.28.2", note = "Please use `apply_sat_file` instead")]
pub async fn exec(
    vault_base_url: &str,
    vault_secret_path: &str,
    vault_role_id: &str,
    sat_file_content: String,
    values_file_content_opt: Option<String>,
    values_cli_opt: Option<Vec<String>>,
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    ansible_verbosity_opt: Option<u8>,
    ansible_passthrough_opt: Option<&String>,
    _watch_logs_opt: Option<&bool>,
    // tag: &str,
    hsm_group_available_vec: &[String],
    k8s_api_url: &str,
    gitea_base_url: &str,
    gitea_token: &str,
    _output_opt: Option<&String>,
) {
    let sat_file_yaml: Value = sat_file_utils::render_jinja2_sat_file_yaml(
        &sat_file_content,
        values_file_content_opt.as_ref(),
        values_cli_opt,
    );

    println!(
        "SAT file content:\n{}",
        serde_yaml::to_string(&sat_file_yaml).unwrap()
    );
    let process_sat_file = dialoguer::Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Please check the template above and confirm to proceed")
        .interact()
        .unwrap();

    if process_sat_file {
        println!("Proceed and process SAT file");
    } else {
        println!("Operation canceled by user. Exit");
        std::process::exit(0);
    }

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

    // Get Cray/HPE product catalog
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

    // Get configurations from CSM
    let configuration_vec = mesa::cfs::configuration::mesa::http_client::get(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        None,
    )
    .await
    .unwrap();

    // Get images from CSM
    let image_vec = mesa::ims::image::mesa::http_client::get_all(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
    )
    .await
    .unwrap();

    // Get IMS recipes from CSM
    let ims_recipe_vec =
        mesa::ims::recipe::http_client::get(shasta_token, shasta_base_url, shasta_root_cert, None)
            .await
            .unwrap();

    // VALIDATION
    // Check HSM groups in images section in SAT file matches the HSM group in JWT (keycloak roles)
    let image_validation_rslt = validate_sat_file_images_section(
        image_yaml_vec_opt.unwrap(),
        configuration_yaml_vec_opt.unwrap(),
        hsm_group_available_vec,
        &cray_product_catalog,
        image_vec,
        configuration_vec,
        ims_recipe_vec,
    );

    if let Err(error) = image_validation_rslt {
        eprintln!("{}", error);
        std::process::exit(1);
    }

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
        let cfs_configuration_rslt: Result<CfsConfigurationResponse, Error> =
            sat_file_utils::create_cfs_configuration_from_sat_file(
                shasta_token,
                shasta_base_url,
                shasta_root_cert,
                gitea_base_url,
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
            false,
        )
        .await;

    println!(
        "List of new image IDs: {:#?}",
        cfs_session_created_hashmap.keys().collect::<Vec<&String>>()
    );
}
