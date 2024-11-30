use dialoguer::theme::ColorfulTheme;
use mesa::{
    cfs::configuration::http_client::v3::r#struct::cfs_configuration_response::CfsConfigurationResponse,
    common::kubernetes, error::Error,
};
use serde_yaml::Value;

use crate::{cli::commands::apply_sat_file::utils, common::cfs_configuration_utils};

/// Creates a configuration from a sat file
/// NOTE: this method manages 2 types of methods [git, product]. For type product, the name must
/// match with a git repo name after concatenating it with "-config-management" (eg: layer name
/// "cos" becomes repo name "cos-config-management" which correlates with https://api-gw-service-nmn.local/vcs/api/v1/repos/cray/cos-config-management)
/// Return CFS configuration name
#[deprecated(since = "1.28.2", note = "Please use `apply_sat_file` instead")]
pub async fn exec(
    sat_file_content: String,
    values_file_content_opt: Option<String>,
    values_cli_opt: Option<Vec<String>>,
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    vault_base_url: &str,
    vault_secret_path: &str,
    vault_role_id: &str,
    k8s_api_url: &str,
    gitea_base_url: &str,
    gitea_token: &str,
    // tag: &str,
    output_opt: Option<&String>,
) -> anyhow::Result<Vec<String>> {
    let sat_file_yaml: Value = utils::render_jinja2_sat_file_yaml(
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

    let empty_vec = &Vec::new();
    let configuration_yaml_vec = configuration_yaml_vec_opt.unwrap_or(empty_vec);

    let mut cfs_configuration_name_vec = Vec::new();

    for configuration_yaml in configuration_yaml_vec {
        let cfs_configuration_rslt: Result<CfsConfigurationResponse, Error> =
            utils::create_cfs_configuration_from_sat_file(
                shasta_token,
                shasta_base_url,
                shasta_root_cert,
                gitea_base_url,
                gitea_token,
                &cray_product_catalog,
                configuration_yaml,
                // tag,
                false,
            )
            .await;

        let cfs_configuration = match cfs_configuration_rslt {
            Ok(cfs_configuration) => cfs_configuration,
            Err(error) => {
                eprintln!("{}", error);
                std::process::exit(1);
            }
        };

        let cfs_configuration_name = cfs_configuration.name.to_string();

        cfs_configuration_name_vec.push(cfs_configuration_name.clone());

        log::info!("CFS configuration created: {}", cfs_configuration_name);

        cfs_configuration_value_vec.push(cfs_configuration.clone());

        // Print output
        if output_opt.is_some() && output_opt.unwrap().eq("json") {
            println!(
                "{}",
                serde_json::to_string_pretty(&cfs_configuration).unwrap()
            );
        } else {
            cfs_configuration_utils::print_table_struct(&cfs_configuration_value_vec);
        }
    }

    Ok(cfs_configuration_name_vec)
}
