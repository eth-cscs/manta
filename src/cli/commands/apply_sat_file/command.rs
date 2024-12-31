use std::collections::HashMap;

use dialoguer::theme::ColorfulTheme;
use mesa::{
    cfs::{
        self,
        configuration::http_client::v3::r#struct::cfs_configuration_response::CfsConfigurationResponse,
    },
    common::kubernetes,
    error::Error,
    hsm, ims,
};
use serde_yaml::Value;
use termion::color;

use crate::cli::commands::{apply_hw_cluster_pin, apply_sat_file::utils};

pub async fn exec(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    vault_base_url: &str,
    vault_secret_path: &str,
    vault_role_id: &str,
    k8s_api_url: &str,
    sat_file_content: String,
    values_file_content_opt: Option<String>,
    values_cli_opt: Option<Vec<String>>,
    hsm_group_param_opt: Option<&String>,
    hsm_group_available_vec: &Vec<String>,
    ansible_verbosity_opt: Option<u8>,
    ansible_passthrough_opt: Option<&String>,
    gitea_base_url: &str,
    gitea_token: &str,
    do_not_reboot: bool,
    watch_logs: bool,
    prehook: Option<&String>,
    posthook: Option<&String>,
    image_only: bool,
    session_template_only: bool,
    debug_on_failure: bool,
    dry_run: bool,
    assume_yes: bool,
) {
    // Validate Pre-hook
    if prehook.is_some() {
        match crate::common::hooks::check_hook_perms(prehook).await {
            Ok(_r) => println!(
                "Pre-hook script '{}' exists and is executable.",
                prehook.unwrap()
            ),
            Err(e) => {
                log::error!("{}. File: {}", e, &prehook.unwrap());
                std::process::exit(2);
            }
        };
    }

    // Validate Post-hook
    if posthook.is_some() {
        match crate::common::hooks::check_hook_perms(posthook).await {
            Ok(_) => println!(
                "Post-hook script '{}' exists and is executable.",
                posthook.unwrap()
            ),
            Err(e) => {
                log::error!("{}. File: {}", e, &posthook.unwrap());
                std::process::exit(2);
            }
        };
    }

    let sat_template_file_yaml: Value = utils::render_jinja2_sat_file_yaml(
        &sat_file_content,
        values_file_content_opt.as_ref(),
        values_cli_opt,
    )
    // .as_mapping_mut()
    // .unwrap()
    .clone();

    let sat_template_file_string = serde_yaml::to_string(&sat_template_file_yaml).unwrap();

    let mut sat_template: utils::SatFile = serde_yaml::from_str(&sat_template_file_string)
        .expect("Could not parse SAT template yaml file");

    // Filter either images or session_templates section according to user request
    //
    sat_template.filter(image_only, session_template_only);

    let sat_template_file_yaml: Value = serde_yaml::to_value(sat_template).unwrap();

    println!(
        "{}#### SAT file content ####{}\n{}",
        color::Fg(color::Blue),
        color::Fg(color::Reset),
        serde_yaml::to_string(&sat_template_file_yaml).unwrap(),
    );

    let process_sat_file = if !assume_yes {
        dialoguer::Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Please check the template above and confirm to proceed.")
            .interact()
            .unwrap()
    } else {
        true
    };

    // Run/process Pre-hook
    if prehook.is_some() {
        println!("Running the pre-hook '{}'", &prehook.unwrap());
        match crate::common::hooks::run_hook(prehook).await {
            Ok(_code) => log::debug!("Pre-hook script completed ok. RT={}", _code),
            Err(_error) => {
                log::error!("{}", _error);
                std::process::exit(2);
            }
        };
    }

    if process_sat_file {
        println!("Proceed and process SAT file");
    } else {
        println!("Operation canceled by user. Exit");
        std::process::exit(0);
    }

    // GET DATA
    //
    // Get data from SAT YAML file
    //
    // Get hardware pattern from SAT YAML file
    let hardware_yaml_value_vec_opt = sat_template_file_yaml["hardware"].as_sequence();

    // Get CFS configurations from SAT YAML file
    let configuration_yaml_vec_opt = sat_template_file_yaml["configurations"].as_sequence();

    // Get images from SAT YAML file
    let image_yaml_vec_opt = sat_template_file_yaml["images"].as_sequence();

    // Get images from SAT YAML file
    let bos_session_template_yaml_vec_opt =
        sat_template_file_yaml["session_templates"].as_sequence();

    // Get Cray/HPE product catalog
    //
    // Get k8s secrets
    let shasta_k8s_secrets = crate::common::vault::http_client::fetch_shasta_k8s_secrets(
        vault_base_url,
        vault_secret_path,
        vault_role_id,
    )
    .await;

    // Get k8s credentials needed to check HPE/Cray product catalog in k8s
    let kube_client = kubernetes::get_k8s_client_programmatically(k8s_api_url, shasta_k8s_secrets)
        .await
        .unwrap();

    // Get HPE product catalog from k8s
    let cray_product_catalog = kubernetes::get_configmap(kube_client, "cray-product-catalog")
        .await
        .unwrap();

    // TODO: multiple API calls to CSM sequentially
    //
    // Get data from CSM
    //
    // Get configurations from CSM
    let configuration_vec = cfs::configuration::http_client::v3::get(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        None,
    )
    .await
    .unwrap();

    // Get images from CSM
    let image_vec =
        ims::image::http_client::get_all(shasta_token, shasta_base_url, shasta_root_cert)
            .await
            .unwrap();

    // Get IMS recipes from CSM
    let ims_recipe_vec =
        ims::recipe::http_client::get(shasta_token, shasta_base_url, shasta_root_cert, None)
            .await
            .unwrap();

    // VALIDATION
    //
    // Validate 'configurations' section
    utils::validate_sat_file_configurations_section(
        configuration_yaml_vec_opt,
        image_yaml_vec_opt,
        bos_session_template_yaml_vec_opt,
    );

    // Validate 'images' section
    let image_validation_rslt = utils::validate_sat_file_images_section(
        image_yaml_vec_opt.unwrap_or(&Vec::new()),
        configuration_yaml_vec_opt.unwrap_or(&Vec::new()),
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

    // Validate 'session_template' section
    utils::validate_sat_file_session_template_section(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        image_yaml_vec_opt,
        configuration_yaml_vec_opt,
        bos_session_template_yaml_vec_opt,
        hsm_group_available_vec,
    )
    .await;

    // PROCESS SAT FILE
    //
    // Process "hardware" section in SAT file
    log::info!("hardware pattern: {:?}", hardware_yaml_value_vec_opt);

    // Process "clusters" section
    //
    if let Some(hw_component_pattern_vec) = hardware_yaml_value_vec_opt {
        for hw_component_pattern in hw_component_pattern_vec {
            let target_hsm_group_name = hw_component_pattern["target"].as_str().unwrap();
            let parent_hsm_group_name = hw_component_pattern["parent"].as_str().unwrap();

            if let Some(pattern) = hw_component_pattern
                .get("pattern")
                .and_then(|pattern_value| pattern_value.as_str())
            {
                log::info!("Processing hw component pattern for '{}' for target HSM group '{}' and parent HSM group '{}'", pattern, target_hsm_group_name, parent_hsm_group_name);
                // When applying a SAT file, I'm assuming the user doesn't want to create new HSM groups or delete empty parent hsm groups
                // But this could be changed.
                apply_hw_cluster_pin::command::exec(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    target_hsm_group_name,
                    parent_hsm_group_name,
                    pattern,
                    true,
                    false,
                    false,
                )
                .await;
            } else if let Some(nodes) = hw_component_pattern
                .get("nodespattern")
                .and_then(|pattern_value| pattern_value.as_str())
            {
                let hsm_group_members_vec: Vec<String> =
                    hsm::group::utils::get_member_vec_from_hsm_group_name(
                        shasta_token,
                        shasta_base_url,
                        shasta_root_cert,
                        target_hsm_group_name,
                    )
                    .await;
                let new_target_hsm_group_members_vec: Vec<String> = nodes
                    .split(',')
                    .filter(|node| !hsm_group_members_vec.contains(&node.to_string()))
                    .map(|node| node.to_string())
                    .collect();

                log::info!(
                    "Processing new nodes '{}' for target HSM group '{}'",
                    nodes,
                    target_hsm_group_name,
                );

                let _ = hsm::group::utils::update_hsm_group_members(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    target_hsm_group_name,
                    &hsm_group_members_vec,
                    &new_target_hsm_group_members_vec,
                )
                .await;
            }
        }
    }

    // Process "configurations" section in SAT file
    //
    log::info!("Process configurations section in SAT file");
    let mut cfs_configuration_value_vec = Vec::new();

    let mut cfs_configuration_name_vec = Vec::new();

    for configuration_yaml in configuration_yaml_vec_opt.unwrap_or(&vec![]).iter() {
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
                dry_run,
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

        println!("CFS configuration '{}' created", cfs_configuration_name);

        cfs_configuration_name_vec.push(cfs_configuration_name.clone());

        cfs_configuration_value_vec.push(cfs_configuration.clone());
    }

    // Process "images" section in SAT file
    //
    // List of image.ref_name already processed
    let mut ref_name_processed_hashmap: HashMap<String, String> = HashMap::new();

    if session_template_only == false {
        let cfs_session_created_hashmap: HashMap<String, serde_yaml::Value> =
            utils::import_images_section_in_sat_file(
                shasta_token,
                shasta_base_url,
                shasta_root_cert,
                vault_base_url,
                vault_secret_path,
                vault_role_id,
                k8s_api_url,
                &mut ref_name_processed_hashmap,
                image_yaml_vec_opt.unwrap_or(&Vec::new()).to_vec(),
                &cray_product_catalog,
                ansible_verbosity_opt,
                ansible_passthrough_opt,
                debug_on_failure,
                dry_run,
                watch_logs,
            )
            .await;

        log::info!(
            "Images created: {:?}",
            cfs_session_created_hashmap.keys().collect::<Vec<&String>>()
        );
    }

    // Process "session_templates" section in SAT file
    //
    if image_only == false {
        utils::process_session_template_section_in_sat_file(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            ref_name_processed_hashmap,
            hsm_group_param_opt,
            hsm_group_available_vec,
            sat_template_file_yaml,
            // &tag,
            do_not_reboot,
            dry_run,
        )
        .await;
    }

    // Run/process Post-hook
    if posthook.is_some() {
        println!("Running the post-hook '{}'", &posthook.unwrap());
        match crate::common::hooks::run_hook(posthook).await {
            Ok(_code) => log::debug!("Post-hook script completed ok. RT={}", _code),
            Err(_error) => {
                log::error!("{}", _error);
                std::process::exit(2);
            }
        };
    }
}
