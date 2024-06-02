use super::apply_sat_file;

#[deprecated(since = "1.28.2", note = "Please use `apply_sat_file` instead")]
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
    apply_sat_file::exec(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        vault_base_url,
        vault_secret_path,
        vault_role_id,
        k8s_api_url,
        sat_file_content,
        values_file_content_opt,
        values_cli_opt,
        hsm_group_param_opt,
        hsm_group_available_vec,
        ansible_verbosity_opt,
        ansible_passthrough_opt,
        gitea_token,
        do_not_reboot,
        None,
        None,
    )
    .await;
}

/* pub async fn validate_sat_file_session_template_section(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    bos_session_template_list_yaml: Option<&Vec<Value>>,
    image_yaml_vec_opt: Option<&Vec<Value>>,
    configuration_yaml_vec_opt: Option<&Vec<Value>>,
    hsm_group_available_vec: &Vec<String>,
) {
    // Validate sessiontemplate section in SAT file
    for bos_session_template_yaml in bos_session_template_list_yaml.unwrap_or(&vec![]) {
        // Validate user has access to HSM groups in sessiontemplate
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
            println!("No HSM group found in session_templates section in SAT file. Exit");
            std::process::exit(1);
        };

        for hsm_group in bos_session_template_hsm_groups {
            if !hsm_group_available_vec.contains(&hsm_group.to_string()) {
                println!(
                        "HSM group '{}' in session_templates {} not allowed, List of HSM groups available {:?}. Exit",
                        hsm_group,
                        bos_session_template_yaml["name"].as_str().unwrap(),
                        hsm_group_available_vec
                    );
                std::process::exit(1);
            }
        }

        // Validate image
        let image_name_opt: Option<&str> = bos_session_template_yaml["image"].as_str();
        if let Some(image_name) = image_name_opt {
            let image_found = image_yaml_vec_opt
                .unwrap()
                .iter()
                .any(|image_yaml_value| image_yaml_value["name"].as_str().unwrap().eq(image_name));

            if !image_found {
                log::info!(
                    "Image '{}' linked to session_template section not found in SAT file",
                    image_name
                );
                if mesa::ims::image::utils::get_fuzzy(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    hsm_group_available_vec,
                    image_name_opt,
                    Some(1).as_ref(),
                )
                .await
                .is_err()
                {
                    log::info!(
                        "Image '{}' in session_template section not found in CSM. Exit",
                        image_name
                    );
                    eprintln!(
                        "Image '{}' in session_template section not found in CSM. Exit",
                        image_name
                    );
                    std::process::exit(1);
                }
            }
        } else {
            eprintln!("Image in session_templates section missing. Exit");
            std::process::exit(1);
        }

        // Validate configuration
        let configuration_name_opt: Option<&str> =
            bos_session_template_yaml["configuration"].as_str();
        if let Some(configuration_name) = configuration_name_opt {
            let configuration_found =
                configuration_yaml_vec_opt
                    .unwrap()
                    .iter()
                    .any(|configuration_yaml_value| {
                        configuration_yaml_value["name"]
                            .as_str()
                            .unwrap()
                            .eq(configuration_name)
                    });

            if !configuration_found {
                log::info!(
                    "Configuration '{}' in session_template section not found in SAT file",
                    configuration_name
                );
                if mesa::cfs::configuration::shasta::http_client::v3::get(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    configuration_name_opt,
                )
                .await
                .is_err()
                {
                    log::info!(
                        "Configuration '{}' in session_template section not found in CSM.",
                        configuration_name
                    );
                    eprintln!(
                        "Configuration '{}' in session_template section not found in CSM.",
                        configuration_name
                    );
                    std::process::exit(1);
                }
            }
        } else {
            eprintln!("Configuration in session_template section missing. Exit");
            std::process::exit(1);
        }
    }
} */

/* pub async fn process_session_template_section_in_sat_file(
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

        let mut boot_param_vec = Vec::new();

        for (property, boot_set) in bos_session_template_yaml["bos_parameters"]["boot_sets"]
            .as_mapping()
            .unwrap()
        {
            let kernel_params = boot_set["kernel_parameters"].as_str().unwrap();
            let arch = boot_set["arch"].as_str().unwrap();

            let bos_session_template_hsm_groups: Vec<String> = boot_set["node_groups"]
                .as_sequence()
                .unwrap_or(&vec![])
                .iter()
                .map(|node| node.as_str().unwrap().to_string())
                .collect();

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

            boot_param_vec.insert(property, element)
        }

        let create_bos_session_template_payload = BosSessionTemplate::new_for_hsm_group(
            bos_session_template_configuration_name,
            bos_session_template_name,
            ims_image_name,
            ims_image_path.to_string(),
            ims_image_type.to_string(),
            ims_image_etag.to_string(),
            hsm_group,
            kernel_params,
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
                bos_sessiontemplate.name
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
} */

/* pub async fn wait_cfs_session_to_complete(
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
} */
