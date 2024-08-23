use core::time;
use std::{
    collections::HashMap,
    io::{self, Write},
};

use dialoguer::theme::ColorfulTheme;
use mesa::{
    bos::{
        self,
        session::shasta::http_client::v2::{BosSession, Operation},
        template::mesa::r#struct::v2::{BootSet, BosSessionTemplate, Cfs},
    },
    cfs::{
        self,
        configuration::mesa::r#struct::cfs_configuration_response::v2::CfsConfigurationResponse,
        session::mesa::r#struct::v2::CfsSessionGetResponse,
    },
    common::{jwt_ops::get_claims_from_jwt_token, kubernetes},
    error::Error,
    hsm, ims,
};
use serde_yaml::Value;
use termion::color;

use crate::{
    cli::{commands::apply_hw_cluster_pin, process::validate_target_hsm_members},
    common::{
        self,
        sat_file::{
            self, import_images_section_in_sat_file, validate_sat_file_configurations_section,
            validate_sat_file_images_section, SatFile,
        },
    },
};

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
    prehook: Option<&String>,
    posthook: Option<&String>,
    image_only: bool,
    session_template_only: bool,
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

    let sat_template_file_yaml: Value = sat_file::render_jinja2_sat_file_yaml(
        &sat_file_content,
        values_file_content_opt.as_ref(),
        values_cli_opt,
    )
    // .as_mapping_mut()
    // .unwrap()
    .clone();

    let sat_template_file_string = serde_yaml::to_string(&sat_template_file_yaml).unwrap();

    let mut sat_template: SatFile = serde_yaml::from_str(&sat_template_file_string)
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

    if !image_only {
        println!(
            "{}#### This operation will reboot the nodes ####{}",
            color::Fg(color::Red),
            color::Fg(color::Reset),
        );
    }

    let process_sat_file = dialoguer::Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Please check the template above and confirm to proceed.")
        .interact()
        .unwrap();

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

    // Get inages from SAT YAML file
    let image_yaml_vec_opt = sat_template_file_yaml["images"].as_sequence();

    // Get inages from SAT YAML file
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
    let configuration_vec = cfs::configuration::mesa::http_client::get(
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
    //
    // Validate 'configurations' section
    validate_sat_file_configurations_section(
        configuration_yaml_vec_opt,
        image_yaml_vec_opt,
        bos_session_template_yaml_vec_opt,
    );

    // Validate 'images' section
    let image_validation_rslt = validate_sat_file_images_section(
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
    validate_sat_file_session_template_section(
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
                apply_hw_cluster_pin::exec(
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
            common::sat_file::create_cfs_configuration_from_sat_file(
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

        log::info!(
            "List of new image IDs: {:?}",
            cfs_session_created_hashmap.keys().collect::<Vec<&String>>()
        );
    }

    // Process "session_templates" section in SAT file
    //
    if image_only == false {
        process_session_template_section_in_sat_file(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            ref_name_processed_hashmap,
            hsm_group_param_opt,
            hsm_group_available_vec,
            sat_template_file_yaml,
            // &tag,
            do_not_reboot,
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

pub async fn validate_sat_file_session_template_section(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    image_yaml_vec_opt: Option<&Vec<Value>>,
    configuration_yaml_vec_opt: Option<&Vec<Value>>,
    session_template_yaml_vec_opt: Option<&Vec<Value>>,
    hsm_group_available_vec: &Vec<String>,
) {
    // Validate 'session_template' section in SAT file
    log::info!("Validate 'session_template' section in SAT file");
    for session_template_yaml in session_template_yaml_vec_opt.unwrap_or(&vec![]) {
        // Validate session_template
        let session_template_name = session_template_yaml["name"].as_str().unwrap();

        log::info!("Validate 'session_template' '{}'", session_template_name);

        // Validate user has access to HSM groups in 'session_template' section
        log::info!(
            "Validate 'session_template' '{}' HSM groups",
            session_template_name
        );

        let bos_session_template_hsm_groups: Vec<String> = if let Some(boot_sets_compute) =
            session_template_yaml["bos_parameters"]["boot_sets"].get("compute")
        {
            boot_sets_compute["node_groups"]
                .as_sequence()
                .unwrap_or(&vec![])
                .iter()
                .map(|node| node.as_str().unwrap().to_string())
                .collect()
        } else if let Some(boot_sets_compute) =
            session_template_yaml["bos_parameters"]["boot_sets"].get("uan")
        {
            boot_sets_compute["node_groups"]
                .as_sequence()
                .unwrap_or(&vec![])
                .iter()
                .map(|node| node.as_str().unwrap().to_string())
                .collect()
        } else {
            println!("No HSM group found in session_templates section in SAT file");
            std::process::exit(1);
        };

        for hsm_group in bos_session_template_hsm_groups {
            if !hsm_group_available_vec.contains(&hsm_group.to_string()) {
                eprintln!(
                        "HSM group '{}' in session_templates {} not allowed, List of HSM groups available {:?}. Exit",
                        hsm_group,
                        session_template_yaml["name"].as_str().unwrap(),
                        hsm_group_available_vec
                    );
                std::process::exit(1);
            }
        }

        // Validate boot image (session_template.image)
        log::info!(
            "Validate 'session_template' '{}' boot image",
            session_template_name
        );

        if let Some(ref_name_to_find) = session_template_yaml
            .get("image")
            .and_then(|image| image.get("image_ref"))
        {
            // Validate image_ref (session_template.image.image_ref). Search in SAT file for any
            // image with images[].ref_name
            log::info!(
                "Searching ref_name '{}' in SAT file",
                ref_name_to_find.as_str().unwrap(),
            );

            let image_ref_name_found = image_yaml_vec_opt.is_some_and(|image_vec| {
                image_vec.iter().any(|image| {
                    image
                        .get("ref_name")
                        .is_some_and(|ref_name| ref_name.eq(ref_name_to_find))
                })
            });

            if !image_ref_name_found {
                eprintln!(
                    "Could not find image ref '{}' in SAT file. Exit",
                    ref_name_to_find.as_str().unwrap()
                );
                std::process::exit(1);
            }
        } else if let Some(image_name_substr_to_find) = session_template_yaml
            .get("image")
            .and_then(|image| image.get("ims").and_then(|ims| ims.get("name")))
        {
            // VaVjlidate image name (session_template.image.ims.name). Search in SAT file and CSM
            log::info!(
                "Searching image name '{}' related to session template '{}' in SAT file",
                image_name_substr_to_find.as_str().unwrap(),
                session_template_yaml["name"].as_str().unwrap()
            );

            let mut image_found = image_yaml_vec_opt.is_some_and(|image_vec| {
                image_vec.iter().any(|image| {
                    image
                        .get("name")
                        .is_some_and(|name| name.eq(image_name_substr_to_find))
                })
            });

            if !image_found {
                // image not found in SAT file, looking in CSM
                log::warn!(
                    "Image name '{}' not found in SAT file, looking in CSM",
                    image_name_substr_to_find.as_str().unwrap()
                );
                log::info!(
                    "Searching image name '{}' related to session template '{}' in CSM",
                    image_name_substr_to_find.as_str().unwrap(),
                    session_template_yaml["name"].as_str().unwrap()
                );

                image_found = mesa::ims::image::utils::get_fuzzy(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    hsm_group_available_vec,
                    image_name_substr_to_find.as_str(),
                    Some(&1),
                )
                .await
                .is_ok();
            }

            if !image_found {
                eprintln!(
                    "Could not find image name '{}' in session_template '{}'. Exit",
                    image_name_substr_to_find.as_str().unwrap(),
                    session_template_yaml["name"].as_str().unwrap()
                );
                std::process::exit(1);
            }
        } else if let Some(image_id) = session_template_yaml
            .get("image")
            .and_then(|image| image.get("ims").and_then(|ims| ims.get("id")))
        {
            // Validate image id (session_template.image.ims.id). Search in SAT file and CSM
            log::info!(
                "Searching image id '{}' related to session template '{}' in CSM",
                image_id.as_str().unwrap(),
                session_template_yaml["name"].as_str().unwrap()
            );

            let image_found = mesa::ims::image::shasta::http_client::get(
                shasta_token,
                shasta_base_url,
                shasta_root_cert,
                image_id.as_str(),
            )
            .await
            .is_ok();

            if !image_found {
                eprintln!(
                    "Could not find image id '{}' in session_template '{}'. Exit",
                    image_id.as_str().unwrap(),
                    session_template_yaml["name"].as_str().unwrap()
                );
                std::process::exit(1);
            }
        } else if let Some(image_name_substr_to_find) = session_template_yaml.get("image") {
            // Backward compatibility
            // VaVjlidate image name (session_template.image.ims.name). Search in SAT file and CSM
            log::info!(
                "Searching image name '{}' related to session template '{}' in CSM - ('sessiontemplate' section in SAT file is outdated - switching to backward compatibility)",
                image_name_substr_to_find.as_str().unwrap(),
                session_template_yaml["name"].as_str().unwrap()
            );

            let image_found = mesa::ims::image::utils::get_fuzzy(
                shasta_token,
                shasta_base_url,
                shasta_root_cert,
                hsm_group_available_vec,
                image_name_substr_to_find.as_str(),
                Some(&1),
            )
            .await
            .is_ok();

            if !image_found {
                // image not found in SAT file, looking in CSM
                log::warn!(
                    "Image name '{}' not found in CSM. Exit",
                    image_name_substr_to_find.as_str().unwrap()
                );
                std::process::exit(1);
            }
        } else {
            eprintln!(
                "Session template '{}' must have one of these entries 'image.ref_name', 'image.ims.name' or 'image.ims.id' values. Exit",
                session_template_yaml["name"].as_str().unwrap(),
            );
            std::process::exit(1);
        }

        // Validate configuration
        log::info!(
            "Validate 'session_template' '{}' configuration",
            session_template_name
        );

        if let Some(configuration_to_find_value) = session_template_yaml.get("configuration") {
            let configuration_to_find = configuration_to_find_value.as_str().unwrap();

            log::info!(
                "Searching configuration name '{}' related to session template '{}' in CSM in SAT file",
                configuration_to_find,
                session_template_yaml["name"].as_str().unwrap()
            );

            let mut configuration_found =
                configuration_yaml_vec_opt.is_some_and(|configuration_yaml_vec| {
                    configuration_yaml_vec.iter().any(|configuration_yaml| {
                        configuration_yaml["name"].eq(configuration_to_find_value)
                    })
                });

            if !configuration_found {
                // CFS configuration in session_template not found in SAT file, searching in CSM
                log::warn!("Configuration not found in SAT file, looking in CSM");
                log::info!(
                    "Searching configuration name '{}' related to session_template '{}' in CSM",
                    configuration_to_find,
                    session_template_yaml["name"].as_str().unwrap()
                );

                configuration_found = cfs::configuration::shasta::http_client::v2::get(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    Some(configuration_to_find),
                )
                .await
                .is_ok();

                if !configuration_found {
                    eprintln!(
                        "ERROR - Could not find configuration '{}' in session_template '{}'. Exit",
                        configuration_to_find,
                        session_template_yaml["name"].as_str().unwrap(),
                    );
                    std::process::exit(1);
                }
            }
        } else {
            eprintln!(
                "Session template '{}' does not have 'configuration' value. Exit",
                session_template_yaml["name"].as_str().unwrap(),
            );
            std::process::exit(1);
        }
    }
}

pub async fn process_session_template_section_in_sat_file(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    ref_name_processed_hashmap: HashMap<String, String>,
    _hsm_group_param_opt: Option<&String>,
    hsm_group_available_vec: &Vec<String>,
    sat_file_yaml: Value,
    do_not_reboot: bool,
) {
    let empty_vec = Vec::new();
    let bos_session_template_list_yaml = sat_file_yaml["session_templates"]
        .as_sequence()
        .unwrap_or(&empty_vec);

    let mut bos_st_created_vec: Vec<String> = Vec::new();

    for bos_sessiontemplate_yaml in bos_session_template_list_yaml {
        let _bos_sessiontemplate: BosSessionTemplate =
            serde_yaml::from_value(bos_sessiontemplate_yaml.clone()).unwrap();

        let image_details: ims::image::r#struct::Image = if let Some(bos_sessiontemplate_image) =
            bos_sessiontemplate_yaml.get("image")
        {
            if let Some(bos_sessiontemplate_image_ims) = bos_sessiontemplate_image.get("ims") {
                // Get boot image to configure the nodes
                if let Some(bos_session_template_image_ims_name) =
                    bos_sessiontemplate_image_ims.get("name")
                {
                    // BOS sessiontemplate boot image defined by name
                    let bos_session_template_image_name = bos_session_template_image_ims_name
                        .as_str()
                        .unwrap()
                        .to_string();

                    // Get base image details
                    ims::image::utils::get_fuzzy(
                        shasta_token,
                        shasta_base_url,
                        shasta_root_cert,
                        hsm_group_available_vec,
                        Some(&bos_session_template_image_name),
                        Some(&1),
                    )
                    .await
                    .unwrap()
                    .first()
                    .unwrap()
                    .0
                    .clone()
                } else if let Some(bos_session_template_image_ims_id) =
                    bos_sessiontemplate_image_ims.get("id")
                {
                    // BOS sessiontemplate boot image defined by id
                    let bos_session_template_image_id = bos_session_template_image_ims_id
                        .as_str()
                        .unwrap()
                        .to_string();

                    // Get base image details
                    ims::image::mesa::http_client::get(
                        shasta_token,
                        shasta_base_url,
                        shasta_root_cert,
                        Some(&bos_session_template_image_id),
                    )
                    .await
                    .unwrap()
                    .first()
                    .unwrap()
                    .clone()
                } else {
                    eprintln!("ERROR: neither 'image.ims.name' nor 'image.ims.id' fields defined in session_template.\nExit");
                    std::process::exit(1);
                }
            } else if let Some(bos_session_template_image_image_ref) =
                bos_sessiontemplate_image.get("image_ref")
            {
                // BOS sessiontemplate boot image defined by image_ref
                let image_ref = bos_session_template_image_image_ref
                    .as_str()
                    .unwrap()
                    .to_string();

                let image_id = ref_name_processed_hashmap
                    .get(&image_ref)
                    .unwrap()
                    .to_string();

                // Get Image by id
                ims::image::mesa::http_client::get(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    Some(&image_id),
                )
                .await
                .unwrap()
                .first()
                .unwrap()
                .clone()
            } else if let Some(image_name_substring) = bos_sessiontemplate_image.as_str() {
                let image_name = image_name_substring;
                // let image_name = image_name_substring.replace("__DATE__", tag);

                // Backward compatibility
                // Get base image details
                log::info!("Looking for IMS image which name contains '{}'", image_name);

                let image_vec = ims::image::utils::get_fuzzy(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    hsm_group_available_vec,
                    Some(&image_name),
                    None,
                )
                .await
                .unwrap();

                // Validate/check if image exists
                if image_vec.is_empty() {
                    eprintln!(
                        "ERROR: Could not find an image which name contains '{}'. Exit",
                        image_name
                    );
                    std::process::exit(1);
                };

                image_vec.first().unwrap().0.clone()
            } else {
                eprintln!("ERROR: neither 'image.ims' nor 'image.image_ref' sections found in session_template.image.\nExit");
                std::process::exit(1);
            }
        } else {
            eprintln!("ERROR: no 'image' section in session_template.\nExit");
            std::process::exit(1);
        };

        log::info!("Image with name '{}' found", image_details.name);

        // Get CFS configuration to configure the nodes
        let bos_session_template_configuration_name = bos_sessiontemplate_yaml["configuration"]
            .as_str()
            .unwrap()
            .to_string();

        // bos_session_template_configuration_name.replace("__DATE__", tag);

        log::info!(
            "Looking for CFS configuration with name: {}",
            bos_session_template_configuration_name
        );

        let cfs_configuration_vec_rslt = cfs::configuration::mesa::http_client::get(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            Some(&bos_session_template_configuration_name),
        )
        .await;

        /* mesa::cfs::configuration::mesa::utils::filter(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            &mut cfs_configuration_vec,
            None,
            hsm_group_available_vec,
            None,
        )
        .await; */

        if cfs_configuration_vec_rslt.is_err() || cfs_configuration_vec_rslt.unwrap().is_empty() {
            eprintln!(
                "ERROR: BOS session template configuration not found in SAT file image list."
            );
            std::process::exit(1);
        }

        let _ims_image_name = image_details.name.to_string();
        let ims_image_etag = image_details.link.as_ref().unwrap().etag.as_ref().unwrap();
        let ims_image_path = &image_details.link.as_ref().unwrap().path;
        let ims_image_type = &image_details.link.as_ref().unwrap().r#type;

        let bos_sessiontemplate_name = bos_sessiontemplate_yaml["name"]
            .as_str()
            .unwrap_or("")
            .to_string();

        // bos_session_template_name.replace("__DATE__", tag);

        let mut boot_set_vec: HashMap<String, BootSet> = HashMap::new();

        for (parameter, boot_set) in bos_sessiontemplate_yaml["bos_parameters"]["boot_sets"]
            .as_mapping()
            .unwrap()
        {
            let kernel_parameters = boot_set["kernel_parameters"].as_str().unwrap();
            let arch_opt = boot_set["arch"].as_str().map(|value| value.to_string());

            let node_roles_groups_opt: Option<Vec<String>> = boot_set
                .get("node_roles_groups")
                .and_then(|node_roles_groups| {
                    node_roles_groups
                        .as_sequence()
                        .and_then(|node_role_groups| {
                            node_role_groups
                                .iter()
                                .map(|hsm_group_value| {
                                    hsm_group_value
                                        .as_str()
                                        .map(|hsm_group| hsm_group.to_string())
                                })
                                .collect()
                        })
                });

            // Validate/check user can create BOS sessiontemplates based on node roles. Users
            // with tenant role are not allowed to create BOS sessiontemplates based on node roles
            // however admin tenants are allowed to create BOS sessiontemplates based on node roles
            if !hsm_group_available_vec.is_empty()
                && node_roles_groups_opt
                    .clone()
                    .is_some_and(|node_roles_groups| !node_roles_groups.is_empty())
            {
                eprintln!("User type tenant can't user node roles in BOS sessiontemplate. Exit");
                std::process::exit(1);
            }

            let node_groups_opt: Option<Vec<String>> =
                boot_set.get("node_groups").and_then(|node_groups_value| {
                    node_groups_value.as_sequence().and_then(|node_group| {
                        node_group
                            .iter()
                            .map(|hsm_group_value| {
                                hsm_group_value
                                    .as_str()
                                    .map(|hsm_group| hsm_group.to_string())
                            })
                            .collect()
                    })
                });

            //TODO: Get rid of this by making sure CSM admins don't create HSM groups for system
            //wide operations instead of using roles
            let node_groups_opt = Some(mesa::hsm::group::hacks::filter_system_hsm_group_names(
                node_groups_opt.unwrap_or_default(),
            ));

            // Validate/check HSM groups in YAML file session_templates.bos_parameters.boot_sets.<parameter>.node_groups matches with
            // Check hsm groups in SAT file includes the hsm_group_param
            for node_group in node_groups_opt.clone().unwrap_or_default() {
                if !hsm_group_available_vec.contains(&node_group.to_string()) {
                    eprintln!("User does not have access to HSM group '{}' in SAT file under session_templates.bos_parameters.boot_sets.compute.node_groups section. Exit", node_group);
                    std::process::exit(1);
                }
            }

            // Validate user has access to the xnames in the BOS sessiontemplate
            let node_list_opt: Option<Vec<String>> =
                boot_set.get("node_list").and_then(|node_list_value| {
                    node_list_value.as_sequence().and_then(|node_list| {
                        node_list
                            .into_iter()
                            .map(|node_value_value| {
                                node_value_value
                                    .as_str()
                                    .map(|node_value| node_value.to_string())
                            })
                            .collect()
                    })
                });

            // Validate user has access to the list of nodes in BOS sessiontemplate
            if let Some(node_list) = &node_list_opt {
                validate_target_hsm_members(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    node_list.iter().map(|node| node.to_string()).collect(),
                )
                .await;
            }

            let cfs = Cfs {
                // clone_url: None,
                // branch: None,
                // commit: None,
                // playbook: None,
                configuration: Some(bos_session_template_configuration_name.clone()),
            };

            let rootfs_provider = Some("cpss3".to_string());
            let rootfs_provider_passthrough = boot_set["rootfs_provider_passthrough"]
                .as_str()
                .map(|value| value.to_string());

            let boot_set = BootSet {
                name: None,
                // boot_ordinal: Some(2),
                // shutdown_ordinal: None,
                path: Some(ims_image_path.to_string()),
                r#type: Some(ims_image_type.to_string()),
                etag: Some(ims_image_etag.to_string()),
                kernel_parameters: Some(kernel_parameters.to_string()),
                // network: Some("nmn".to_string()),
                node_list: node_list_opt,
                node_roles_groups: node_roles_groups_opt, // TODO: investigate whether this value can be a list
                // of nodes and if it is process it properly
                node_groups: node_groups_opt,
                rootfs_provider,
                rootfs_provider_passthrough,
                cfs: Some(cfs),
                arch: arch_opt,
            };

            boot_set_vec.insert(parameter.as_str().unwrap().to_string(), boot_set);
        }

        /* let create_bos_session_template_payload = BosSessionTemplate::new_for_hsm_group(
            bos_session_template_configuration_name,
            bos_session_template_name,
            ims_image_name,
            ims_image_path.to_string(),
            ims_image_type.to_string(),
            ims_image_etag.to_string(),
            hsm_group,
        ); */

        let cfs = Cfs {
            // clone_url: None,
            // branch: None,
            // commit: None,
            // playbook: None,
            configuration: Some(bos_session_template_configuration_name),
        };

        let create_bos_session_template_payload = BosSessionTemplate {
            // template_url: None,
            // name: Some(bos_sessiontemplate_name.clone()),
            name: None,
            description: None,
            // cfs_url: None,
            // cfs_branch: None,
            enable_cfs: Some(true),
            cfs: Some(cfs),
            // partition: None,
            boot_sets: Some(boot_set_vec),
            links: None,
            tenant: None,
        };

        let create_bos_session_template_resp = bos::template::shasta::http_client::v2::put(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            &create_bos_session_template_payload,
            // &create_bos_session_template_payload.name.as_ref().unwrap(),
            &bos_sessiontemplate_name,
        )
        .await;

        match create_bos_session_template_resp {
            Ok(bos_sessiontemplate) => {
                println!(
                    "BOS sessiontemplate name '{}' created",
                    bos_sessiontemplate_name
                );

                bos_st_created_vec.push(bos_sessiontemplate.name.unwrap())
            }
            Err(error) => eprintln!(
                "ERROR: BOS session template creation failed.\nReason:\n{}\nExit",
                error
            ),
        }
    }

    // Create BOS session. Note: reboot operation shuts down the nodes and they may not start
    // up... hence we will split the reboot into 2 operations shutdown and start

    if do_not_reboot {
        log::info!("Reboot canceled by user");
    } else {
        log::info!("Rebooting");

        for bos_st_name in bos_st_created_vec {
            log::info!(
                "Creating BOS session for BOS sessiontemplate '{}' to reboot",
                bos_st_name
            );

            // BOS session v1
            /* let create_bos_session_resp = bos::session::shasta::http_client::v1::post(
                shasta_token,
                shasta_base_url,
                shasta_root_cert,
                &bos_st_name,
                "reboot",
                None,
            )
            .await; */

            // BOS session v2
            let bos_session = BosSession {
                name: None,
                tenant: None,
                operation: Some(Operation::Reboot),
                template_name: bos_st_name.clone(),
                limit: None,
                stage: None,
                include_disabled: None,
                status: None,
                components: None,
            };

            let create_bos_session_resp = bos::session::shasta::http_client::v2::post(
                shasta_token,
                shasta_base_url,
                shasta_root_cert,
                bos_session,
            )
            .await;

            match create_bos_session_resp {
                Ok(_) => {
                    // log::info!("K8s job relates to BOS session v1 '{}'", bos_session["job"].as_str().unwrap());
                    println!(
                        "BOS session for BOS sessiontemplate '{}' created",
                        bos_st_name
                    )
                }
                Err(error) => eprintln!(
                    "ERROR: BOS session for BOS sessiontemplate '{}' creation failed.\nReason:\n{}\nExit",
                    bos_st_name,
                    error
                ),
            }

            let bos_sessiontemplate_vec = bos::template::mesa::http_client::get(
                shasta_token,
                shasta_base_url,
                shasta_root_cert,
                Some(&bos_st_name),
            )
            .await
            .unwrap();

            let bos_sessiontemplate = bos_sessiontemplate_vec.first().unwrap();

            let _ = if !bos_sessiontemplate.get_target_hsm().is_empty() {
                // Get list of XNAMES for all HSM groups
                let mut xnames = Vec::new();
                for hsm in bos_sessiontemplate.get_target_hsm().iter() {
                    xnames.append(
                        &mut hsm::group::utils::get_member_vec_from_hsm_group_name(
                            shasta_token,
                            shasta_base_url,
                            shasta_root_cert,
                            hsm,
                        )
                        .await,
                    );
                }

                xnames
            } else {
                // Get list of XNAMES
                bos_sessiontemplate.get_target_xname()
            };

            // power_reset_nodes::exec(
            //     shasta_token,
            //     shasta_base_url,
            //     shasta_root_cert,
            //     xnames,
            //     Some("Force BOS session reboot".to_string()),
            //     true,
            // )
            // .await;
        }
    }

    // Audit
    let jwt_claims = get_claims_from_jwt_token(shasta_token).unwrap();

    log::info!(target: "app::audit", "User: {} ({}) ; Operation: Apply cluster", jwt_claims["name"].as_str().unwrap(), jwt_claims["preferred_username"].as_str().unwrap());
}

// TODO: Document and move to mod bos/session/utils
pub async fn wait_bos_session_v2_to_complete_or_force_reboot(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    bos_session: &mesa::bos::session::shasta::http_client::v2::BosSession,
) -> Result<(), Error> {
    // Get nodes related to BOS session
    let bos_st = mesa::bos::template::mesa::http_client::get(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        Some(&bos_session.template_name),
    )
    .await
    .unwrap()
    .first()
    .cloned()
    .unwrap();

    let mut nodes = Vec::new();

    for (_, boot_set) in bos_st.boot_sets.unwrap() {
        if let Some(node_group_vec) = boot_set.node_groups {
            let mut xname_vec = mesa::hsm::group::utils::get_member_vec_from_hsm_name_vec(
                shasta_token,
                shasta_base_url,
                shasta_root_cert,
                node_group_vec.to_vec(),
            )
            .await;

            nodes.append(&mut xname_vec);
        }

        if let Some(mut node_vec) = boot_set.node_list {
            nodes.append(&mut node_vec);
        }

        // TODO: Add logic to process nodes by role (boot_set.nodes_roles_group). Only admins
        // should be able to do this or is TAPMS clever enough to break down roles per tenant???
    }

    // Get nodes power status
    // Wait till power state is what we expect

    Ok(())
}

// TODO: Document and move to mod cfs/session/utils
pub async fn wait_cfs_session_to_complete(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    cfs_session: &CfsSessionGetResponse,
) {
    // Monitor CFS image creation process ends
    let mut i = 0;
    let max = 1800; // Max ammount of attempts to check if CFS session has ended
    loop {
        let cfs_session_vec = mesa::cfs::session::mesa::http_client::get(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            None,
            None,
            None,
            Some(&cfs_session.name.as_ref().unwrap().to_string()),
            Some(true),
        )
        .await
        .unwrap();
        /*
        mesa::cfs::session::mesa::utils::filter_by_hsm(
            shasta_token,
            shasta_base_url,
            shasa_root_cert,
            &mut cfs_session_vec,
            hsm_group_available_vec,
            Some(&1),
        )
        .await; */

        if !cfs_session_vec.is_empty()
            && cfs_session_vec
                .first()
                .unwrap()
                .status
                .as_ref()
                .unwrap()
                .session
                .as_ref()
                .unwrap()
                .status
                .as_ref()
                .unwrap()
                .eq("complete")
            && i <= max
        {
            /* let cfs_session_aux: CfsSessionGetResponse =
            CfsSessionGetResponse::from_csm_api_json(
                cfs_session_value_vec.first().unwrap().clone(),
            ); */

            let cfs_session = cfs_session_vec.first().unwrap();

            if !cfs_session.is_success() {
                // CFS session failed
                eprintln!(
                    "ERROR creating CFS session '{}'",
                    cfs_session.name.as_ref().unwrap()
                );
                std::process::exit(1);
            } else {
                // CFS session succeeded
                // cfs_session_complete_vec.push(cfs_session.clone());

                log::info!(
                    "CFS session created: {}",
                    cfs_session.name.as_ref().unwrap()
                );

                break;
            }
        } else {
            print!("\x1B[2K");
            io::stdout().flush().unwrap();
            print!(
                "\rCFS session '{}' running. Checking again in 2 secs. Attempt {} of {}",
                cfs_session.name.as_ref().unwrap(),
                i + 1,
                max
            );
            io::stdout().flush().unwrap();

            tokio::time::sleep(time::Duration::from_secs(2)).await;
            std::io::Write::flush(&mut std::io::stdout()).unwrap();

            i += 1;
        }
    }
}
