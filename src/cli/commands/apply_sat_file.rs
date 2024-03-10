use core::time;
use std::{
    collections::HashMap,
    io::{self, Write},
};

use dialoguer::theme::ColorfulTheme;
use mesa::{
    cfs::{
        configuration::mesa::r#struct::cfs_configuration_response::{
            ApiError, CfsConfigurationResponse,
        },
        session::mesa::r#struct::CfsSessionGetResponse,
    },
    common::kubernetes,
    ims, {capmc, hsm},
};
use serde_yaml::Value;

use crate::common::{
    self,
    jwt_ops::get_claims_from_jwt_token,
    sat_file::{
        self, import_images_section_in_sat_file, validate_sat_file_configurations_section,
        validate_sat_file_images_section,
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
    gitea_token: &str,
    // tag: &str,
    do_not_reboot: bool,
) {
    let sat_file_yaml: Value = sat_file::render_jinja2_sat_file_yaml(
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

    // let sat_file_yaml: Value = serde_yaml::from_str(&file_content).unwrap();

    // Get hardware pattern from SAT YAML file
    let hardware_yaml_value_vec_opt = sat_file_yaml["hardware"].as_sequence();

    // Get CFS configurations from SAT YAML file
    let configuration_yaml_vec_opt = sat_file_yaml["configurations"].as_sequence();

    // Get inages from SAT YAML file
    let image_yaml_vec_opt = sat_file_yaml["images"].as_sequence();

    // Get inages from SAT YAML file
    let bos_session_template_yaml_vec_opt = sat_file_yaml["session_templates"].as_sequence();

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

    // VALIDATION
    validate_sat_file_configurations_section(
        configuration_yaml_vec_opt,
        image_yaml_vec_opt,
        bos_session_template_yaml_vec_opt,
    );

    // Get IMS recipes from CSM
    let ims_recipe_vec =
        mesa::ims::recipe::http_client::get(shasta_token, shasta_base_url, shasta_root_cert, None)
            .await
            .unwrap();

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
    }

    // validation of session_templates section in the SAT file is below
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

    // Process "hardware" section in SAT file

    log::info!("hardware pattern: {:?}", hardware_yaml_value_vec_opt);

    // Process "configurations" section in SAT file
    //
    let mut cfs_configuration_value_vec = Vec::new();

    let mut cfs_configuration_name_vec = Vec::new();

    for configuration_yaml in configuration_yaml_vec_opt.unwrap_or(&vec![]).iter() {
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

        let cfs_configuration_name = cfs_configuration.name.to_string();

        println!("CFS configuration '{}' created", cfs_configuration_name);

        cfs_configuration_name_vec.push(cfs_configuration_name.clone());

        cfs_configuration_value_vec.push(cfs_configuration.clone());
    }

    // Process "images" section in SAT file

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

    log::info!(
        "List of new image IDs: {:?}",
        cfs_session_created_hashmap.keys().collect::<Vec<&String>>()
    );

    // Process "session_templates" section in SAT file

    process_session_template_section_in_sat_file(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        ref_name_processed_hashmap,
        hsm_group_param_opt,
        hsm_group_available_vec,
        sat_file_yaml,
        // &tag,
        do_not_reboot,
    )
    .await;
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
                println!(
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
            }
        } else if let Some(image_name_substr_to_find) = session_template_yaml
            .get("image")
            .and_then(|image| image.get("ims").and_then(|ims| ims.get("name")))
        {
            // Validate image name (session_template.image.ims.name). Search in SAT file and CSM
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
                println!(
                    "Could not find image name '{}' in session_template '{}'. Exit",
                    image_name_substr_to_find.as_str().unwrap(),
                    session_template_yaml["name"].as_str().unwrap()
                );
            }
        } else {
            eprintln!(
                "Session template '{}' neither have 'image.ref_name' nor 'image.ims.name' values. Exit",
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
                        configuration_yaml.eq(configuration_to_find_value)
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

                configuration_found = mesa::cfs::configuration::shasta::http_client::get(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    Some(configuration_to_find),
                )
                .await
                .is_ok();

                if !configuration_found {
                    println!(
                        "Could not find configuration '{}' in session_template '{}'. Exit",
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
    hsm_group_param_opt: Option<&String>,
    hsm_group_available_vec: &Vec<String>,
    sat_file_yaml: Value,
    // tag: &str,
    do_not_reboot: bool,
) {
    let empty_vec = Vec::new();
    let bos_session_template_list_yaml = sat_file_yaml["session_templates"]
        .as_sequence()
        .unwrap_or(&empty_vec);

    for bos_session_template_yaml in bos_session_template_list_yaml {
        let image_details: ims::image::r#struct::Image = if let Some(bos_session_template_image) =
            bos_session_template_yaml.get("image")
        {
            if let Some(bos_session_template_image_ims) = bos_session_template_image.get("ims") {
                if let Some(bos_session_template_image_ims_name) =
                    bos_session_template_image_ims.get("name")
                {
                    let bos_session_template_image_name = bos_session_template_image_ims_name
                        .as_str()
                        .unwrap()
                        .to_string();

                    // Get base image details
                    mesa::ims::image::utils::get_fuzzy(
                        shasta_token,
                        shasta_base_url,
                        shasta_root_cert,
                        hsm_group_available_vec,
                        Some(&bos_session_template_image_name),
                        None,
                    )
                    .await
                    .unwrap()
                    .first()
                    .unwrap()
                    .0
                    .clone()
                } else {
                    eprintln!("ERROR: no 'image.ims.name' section in session_template.\nExit");
                    std::process::exit(1);
                }
            } else if let Some(bos_session_template_image_image_ref) =
                bos_session_template_image.get("image_ref")
            {
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
            } else if let Some(image_name_substring) = bos_session_template_image.as_str() {
                let image_name = image_name_substring;
                // let image_name = image_name_substring.replace("__DATE__", tag);

                // Backward compatibility
                // Get base image details
                log::info!("Looking for IMS image which name contains '{}'", image_name);

                let image_vec = mesa::ims::image::utils::get_fuzzy(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    hsm_group_available_vec,
                    Some(&image_name),
                    None,
                )
                .await
                .unwrap();

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
        let bos_session_template_configuration_name = bos_session_template_yaml["configuration"]
            .as_str()
            .unwrap()
            .to_string();

        // bos_session_template_configuration_name.replace("__DATE__", tag);

        log::info!(
            "Looking for CFS configuration with name: {}",
            bos_session_template_configuration_name
        );

        let cfs_configuration_vec_rslt = mesa::cfs::configuration::mesa::http_client::get(
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

        let ims_image_name = image_details.name.to_string();
        let ims_image_etag = image_details.link.as_ref().unwrap().etag.as_ref().unwrap();
        let ims_image_path = &image_details.link.as_ref().unwrap().path;
        let ims_image_type = &image_details.link.as_ref().unwrap().r#type;

        let bos_session_template_name = bos_session_template_yaml["name"]
            .as_str()
            .unwrap_or("")
            .to_string();

        // bos_session_template_name.replace("__DATE__", tag);

        let bos_session_template_hsm_groups: Vec<String> = if let Some(boot_sets_compute) =
            bos_session_template_yaml["bos_parameters"]["boot_sets"].get("compute")
        {
            boot_sets_compute["node_groups"]
                .as_sequence()
                .unwrap_or(&vec![])
                .iter()
                .map(|node| node.as_str().unwrap().to_string())
                .collect()
        } else if let Some(boot_sets_compute) =
            bos_session_template_yaml["bos_parameters"]["boot_sets"].get("uan")
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

        // Check HSM groups in YAML file session_templates.bos_parameters.boot_sets.compute.node_groups matches with
        // Check hsm groups in SAT file includes the hsm_group_param
        let hsm_group = if hsm_group_param_opt.is_some()
            && !bos_session_template_hsm_groups
                .iter()
                .any(|h_g| h_g.eq(hsm_group_param_opt.unwrap()))
        {
            eprintln!("HSM group in param does not matches with any HSM groups in SAT file under session_templates.bos_parameters.boot_sets.compute.node_groups section. Using HSM group in param as the default");
            hsm_group_param_opt.unwrap()
        } else {
            bos_session_template_hsm_groups.first().unwrap()
        };

        let create_bos_session_template_payload =
            mesa::bos::template::mesa::r#struct::request_payload::BosSessionTemplate::new_for_hsm_group(
                bos_session_template_configuration_name,
                bos_session_template_name,
                ims_image_name,
                ims_image_path.to_string(),
                ims_image_type.to_string(),
                ims_image_etag.to_string(),
                hsm_group,
            );

        let create_bos_session_template_resp = mesa::bos::template::shasta::http_client::post(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            &create_bos_session_template_payload,
        )
        .await;

        match create_bos_session_template_resp {
            Ok(bos_sessiontemplate) => println!(
                "BOS sessiontemplate name '{}' created",
                bos_sessiontemplate.as_str().unwrap()
            ),
            Err(error) => eprintln!(
                "ERROR: BOS session template creation failed.\nReason:\n{}\nExit",
                error
            ),
        }

        // Create BOS session. Note: reboot operation shuts down the nodes and they may not start
        // up... hence we will split the reboot into 2 operations shutdown and start

        if do_not_reboot {
            log::info!("Reboot canceled by user");
        } else {
            log::info!("Rebooting nodes");
            // Get nodes members of HSM group
            // Get HSM group details
            let hsm_group_details = hsm::group::shasta::http_client::get(
                shasta_token,
                shasta_base_url,
                shasta_root_cert,
                Some(hsm_group),
            )
            .await;

            // Get list of xnames in HSM group
            let nodes: Vec<String> = hsm_group_details.unwrap().first().unwrap()["members"]["ids"]
                .as_array()
                .unwrap()
                .iter()
                .map(|node| node.as_str().unwrap().to_string())
                .collect();

            // Create CAPMC operation shutdown
            let capmc_shutdown_nodes_resp = capmc::http_client::node_power_off::post_sync(
                shasta_token,
                shasta_base_url,
                shasta_root_cert,
                nodes.clone(),
                Some("Shut down cluster to apply changes".to_string()),
                true,
            )
            .await;

            log::debug!(
                "CAPMC shutdown nodes response:\n{:#?}",
                capmc_shutdown_nodes_resp
            );

            // Create BOS session operation start
            let create_bos_boot_session_resp = mesa::bos::session::shasta::http_client::post(
                shasta_token,
                shasta_base_url,
                shasta_root_cert,
                &create_bos_session_template_payload.name,
                "boot",
                Some(&nodes.join(",")),
            )
            .await;

            log::debug!(
                "Create BOS boot session response:\n{:#?}",
                create_bos_boot_session_resp
            );

            if create_bos_boot_session_resp.is_err() {
                eprintln!("Error creating BOS boot session. Exit");
                std::process::exit(1);
            }
        }

        // Audit
        let jwt_claims = get_claims_from_jwt_token(shasta_token).unwrap();

        log::info!(target: "app::audit", "User: {} ({}) ; Operation: Apply cluster", jwt_claims["name"].as_str().unwrap(), jwt_claims["preferred_username"].as_str().unwrap());
    }
}

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
            Some(&cfs_session.name.as_ref().unwrap().to_string()),
            Some(true),
        )
        .await
        .unwrap();
        /*
        mesa::cfs::session::mesa::utils::filter_by_hsm(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
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
