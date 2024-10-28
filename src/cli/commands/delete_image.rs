pub mod command {
    

    use mesa::ims::image::{self, r#struct::Image};

    pub async fn exec(
        shasta_token: &str,
        shasta_base_url: &str,
        shasta_root_cert: &[u8],
        hsm_name_available_vec: Vec<String>,
        image_id_vec: &[&str],
        dry_run: bool,
    ) {
        log::info!(
            "Executing command to delete images: {}",
            image_id_vec.join(", "),
        );

        /* // VALIDATION
        // Check image is used to boot nodes
        // Get list of xnamne members for all HSM groups available to the user
        let xname_available_vec = mesa::hsm::group::utils::get_member_vec_from_hsm_name_vec(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            hsm_name_available_vec.clone(),
        )
        .await;

        // Get BOS sessiontemplates
        let mut bos_sessiontemplate_vec = mesa::bos::template::mesa::http_client::get_all(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
        )
        .await
        .unwrap();

        // Filter BOS sessiontemplates to the ones related to any HSM group the user has access to
        mesa::bos::template::mesa::utils::filter(
            &mut bos_sessiontemplate_vec,
            &hsm_name_available_vec.as_slice(),
            &xname_available_vec,
            None,
        );

        // Get list of image ids from the BOS sessiontemplate list filtered
        let image_available_vec: Vec<String> = bos_sessiontemplate_vec
            .iter()
            .flat_map(|bos_st| bos_st.get_image_vec())
            .collect();

        // Filter input list of images ids with the ones the user has access to from the BOS
        // sessiontemplates related to HSM groups the user has access to
        let image_id_vec: Vec<String> = image_id_vec
            .iter()
            .filter(|&&image_id| image_available_vec.contains(&image_id.to_string()))
            .cloned()
            .collect();

        // Get all boot parameters for all xnames available to the user
        let boot_param_vec_rslt = mesa::bss::bootparameters::http_client::get_raw(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            // &xname_available_vec,
            &[],
        )
        .await;

        let boot_param_vec = match boot_param_vec_rslt {
            Ok(boot_param_vec) => boot_param_vec,
            Err(e) => {
                eprintln!(
                    "ERROR - could not fetch boot parameters.\nReason:\n{:#?}",
                    e
                );
                std::process::exit(1);
            }
        };

        //================================
        /* let hsm_group_all_vec =
            mesa::hsm::group::http_client::get_all(shasta_token, shasta_base_url, shasta_root_cert)
                .await
                .unwrap();
        let mut xname_all_vec: Vec<String> = hsm_group_all_vec
            .iter()
            .flat_map(|hsm_group| {
                hsm_group
                    .members
                    .as_ref()
                    .unwrap()
                    .ids
                    .as_ref()
                    .unwrap()
                    .clone()
            })
            .collect();
        xname_all_vec.sort();
        xname_all_vec.dedup();

        let mut xname_with_bootparameters_vec: Vec<String> = Vec::new();
        let mut num_boot_params_without_hosts = 0;
        for boot_param in &boot_param_vec {
            if boot_param.hosts.eq(&vec!["".to_string()]) || boot_param.hosts.is_empty() {
                println!(
                    "DEBUG something wrong with this boot params because its hosts is empty????:\n{:#?}",
                    boot_param
                );
                // println!("hosts: {:#?}", boot_param.hosts);
                num_boot_params_without_hosts += 1;
            }

            xname_with_bootparameters_vec.extend(boot_param.hosts.clone());
        }
        xname_with_bootparameters_vec.sort();
        xname_with_bootparameters_vec.dedup();

        let xname_without_bootparameters_vec: Vec<String> = xname_all_vec
            .iter()
            .filter(|&xname| !xname_with_bootparameters_vec.contains(xname))
            .cloned()
            .collect();
        let xname_non_cn_with_bootparameters_vec: Vec<String> = xname_with_bootparameters_vec
            .iter()
            .filter(|&xname| !xname_all_vec.contains(xname))
            .cloned()
            .collect();
        println!("DEBUG - total number of nodes: {}", xname_all_vec.len());
        println!(
            "DEBUG - number of nodes with boot parameters: {}",
            xname_with_bootparameters_vec.len()
        );
        println!(
            "DEBUG - number of nodes without boot parameters: {}",
            xname_without_bootparameters_vec.len()
        );
        println!(
            "DEBUG - number of non compute nodes with boot parameters: {}",
            xname_non_cn_with_bootparameters_vec.len()
        );
        println!(
            "DEBUG - non compute nodes with boot parameters: {}",
            xname_non_cn_with_bootparameters_vec.len()
        );
        println!(
            "DEBUG - number of boot params without hosts: {}",
            num_boot_params_without_hosts
        );
        println!(
            "DEBUG - total number of boot params: {}",
            boot_param_vec.len()
        ); */
        //================================

        let mut image_xnames_boot_map: HashMap<String, Vec<String>> = HashMap::new();

        for boot_param in &boot_param_vec {
            let mut host_vec = boot_param.hosts.clone();
            let boot_image = boot_param.get_boot_image();

            if image_id_vec.contains(&boot_image.as_str()) && !boot_param.hosts.is_empty() {
                /* println!("DEBUG - image id: {}", boot_image);
                println!("DEBUG - hosts: {:?}", host_vec); */
                // println!("DEBUG - boot parameter:\n{:#?}", boot_param);
                image_xnames_boot_map
                    .entry(boot_image)
                    .and_modify(|xname_vec| xname_vec.append(&mut host_vec))
                    .or_insert(host_vec);
            }
        }

        if !image_xnames_boot_map.is_empty() {
            eprintln!(
                "ERROR - The following images could not be deleted since they boot nodes.\n{:#?}",
                image_xnames_boot_map
            );
            std::process::exit(1);
        } */

        let mut image_vec: Vec<Image> =
            image::mesa::http_client::get_all(shasta_token, shasta_base_url, shasta_root_cert)
                .await
                .unwrap();

        let image_detail_tuple_vec: Vec<(Image, String, String, bool)> = image::utils::filter(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            &mut image_vec,
            &hsm_name_available_vec,
            None,
        )
        .await;

        // VALIDATE
        // Check images user wants to delete are not being used to boot nodes
        let mut image_xnames_boot_map = Vec::new();
        for image_details_tuple in image_detail_tuple_vec {
            let image_id = image_details_tuple.0.name;
            if image_details_tuple.3 && image_id_vec.contains(&image_id.as_str()) {
                image_xnames_boot_map.push(image_id);
            }
        }

        // Exit if any image id user wants to delete is used to boot nodes
        if !image_xnames_boot_map.is_empty() {
            eprintln!(
                "ERROR - The following images could not be deleted since they boot nodes.\n{:#?}",
                image_xnames_boot_map
            );
            std::process::exit(1);
        }

        for image_id in image_id_vec {
            if dry_run {
                eprintln!("Dry-run enabled. No changes persisted into the system");
            } else {
                let _ = image::shasta::http_client::delete(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    &image_id,
                )
                .await;
            }
        }

        println!("Images deleted:\n{:#?}", image_id_vec);
    }
}
