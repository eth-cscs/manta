use core::time;
use std::{ops::Deref, path::PathBuf, thread};

// use clap::ArgMatches;
use mesa::{
    mesa::cfs::{
        configuration::get_put_payload::CfsConfigurationResponse,
        session::get_response_struct::CfsSessionGetResponse,
    },
    shasta::{
        bos::{self, template},
        capmc,
        cfs::session,
        hsm,
        ims::image,
    },
};
use serde_yaml::Value;

use crate::common::jwt_ops::get_claims_from_jwt_token;

use super::apply_image;

pub async fn exec(
    vault_base_url: &str,
    vault_secret_path: &str,
    vault_role_id: &str,
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    path_file: &PathBuf,
    hsm_group_param_opt: Option<&String>,
    hsm_group_available_vec_opt: Option<&[String]>,
    ansible_verbosity_opt: Option<&String>,
    ansible_passthrough_opt: Option<&String>,
    k8s_api_url: &str,
    watch_logs: Option<&bool>,
    tag: String,
    output_opt: Option<&String>,
) {
    let file_content = std::fs::read_to_string(path_file).unwrap();
    let sat_file_yaml: Value = serde_yaml::from_str(&file_content).unwrap();

    let empty_vec = Vec::new();
    let bos_session_template_list_yaml = sat_file_yaml["session_templates"]
        .as_sequence()
        .unwrap_or(&empty_vec);

    // VALIDATION
    // Check HSM groups in session_templates in SAT file section matches the HSM group in Manta configuration file
    // This is a bit messy... images section in SAT file valiidation is done inside apply_image::exec but the
    // validation of session_templates section in the SAT file is below
    if let Some(hsm_group_available_vec) = hsm_group_available_vec_opt {
        for bos_session_template_yaml in bos_session_template_list_yaml {
            let bos_session_template_hsm_groups: Vec<String> = if let Some(boot_sets_compute) =
                bos_session_template_yaml["bos_parameters"]["boot_sets"].get("compute")
            {
                boot_sets_compute["node_groups"]
                    .as_sequence()
                    .unwrap_or(&Vec::new())
                    .iter()
                    .map(|node| node.as_str().unwrap().to_string())
                    .collect()
            } else if let Some(boot_sets_compute) =
                bos_session_template_yaml["bos_parameters"]["boot_sets"].get("uan")
            {
                boot_sets_compute["node_groups"]
                    .as_sequence()
                    .unwrap_or(&Vec::new())
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
                        bos_session_template_yaml["name"].as_str().unwrap(),
                        hsm_group_available_vec
                    );
                    std::process::exit(-1);
                }
            }
        }
    } else {
        println!("No HSM groups user has access defined, please check with your Alps sys admin this. Exit");
        std::process::exit(1);
    }

    // Create CFS configuration and image
    let (cfs_configuration_vec, cfs_session_vec) = apply_image::exec(
        vault_base_url,
        vault_secret_path,
        vault_role_id,
        path_file,
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        ansible_verbosity_opt,
        ansible_passthrough_opt,
        watch_logs,
        &tag,
        hsm_group_available_vec_opt,
        k8s_api_url,
        output_opt,
    )
    .await;

    let mut cfs_session_complete_vec: Vec<CfsSessionGetResponse> = Vec::new();
    let mut cfs_configuration_from_session_complete_vec: Vec<CfsConfigurationResponse> = Vec::new();
    // let mut cfs_session_result_id_list = Vec::new();

    for (cfs_configuration, cfs_session) in cfs_configuration_vec.iter().zip(cfs_session_vec.iter())
    {
        // Monitor CFS image creation process ends
        let cfs_session_name = cfs_session.name.clone().unwrap();

        let mut i = 0;
        let max = 1800; // Max ammount of attempts to check if CFS session has ended
        loop {
            let cfs_session_value_vec_rslt = session::http_client::get(
                shasta_token,
                shasta_base_url,
                shasta_root_cert,
                None,
                Some(&cfs_session_name),
                Some(&1),
                Some(true),
            )
            .await;

            if cfs_session_value_vec_rslt
                .as_ref()
                .is_ok_and(|cfs_session_vec| {
                    !cfs_session_vec.is_empty()
                        && cfs_session_vec.first().unwrap()["status"]["session"]["status"]
                            .eq("complete")
                })
                && i <= max
            {
                let cfs_session_aux: CfsSessionGetResponse =
                    CfsSessionGetResponse::from_csm_api_json(
                        cfs_session_value_vec_rslt.unwrap().first().unwrap().clone(),
                    );

                cfs_session_complete_vec.push(cfs_session_aux);

                cfs_configuration_from_session_complete_vec.push(cfs_configuration.clone());

                break;
            } else {
                print!(
                    "\rCFS session '{}' running. Checking again in 2 secs. Attempt {} of {}",
                    cfs_session_name, // TODO: remove this clone
                    i + 1,
                    max
                );

                thread::sleep(time::Duration::from_secs(2));
                std::io::Write::flush(&mut std::io::stdout()).unwrap();

                i += 1;
            }
        }
    }

    println!(); // Don't delete we do need to print an empty line here for the previous waiting CFS
    // session message

    // Create BOS sessiontemplate

    let empty_vec = Vec::new();
    let bos_session_template_list_yaml = sat_file_yaml["session_templates"]
        .as_sequence()
        .unwrap_or(&empty_vec);

    for bos_session_template_yaml in bos_session_template_list_yaml {
        let mut bos_session_template_image_name = bos_session_template_yaml["image"]
            .as_str()
            .unwrap_or("")
            .to_string();

        bos_session_template_image_name = bos_session_template_image_name.replace("__DATE__", &tag);

        let cfs_session_detail_opt = cfs_session_complete_vec.iter().find(|cfs_session_detail| {
            cfs_session_detail
                .name
                .clone()
                .unwrap()
                .eq(&bos_session_template_image_name)
        });

        if cfs_session_detail_opt.is_none() {
            eprintln!("ERROR: BOS session template image not found in SAT file image list.");
            std::process::exit(1);
        }

        let cfs_session_detail = cfs_session_detail_opt.unwrap().clone();

        let bos_session_template_configuration_name = bos_session_template_yaml["configuration"]
            .as_str()
            .unwrap()
            .to_string()
            .replace("__DATE__", &tag);

        let cfs_configuration_detail_opt =
            cfs_configuration_from_session_complete_vec
                .iter()
                .find(|cfs_configuration_detail| {
                    cfs_configuration_detail
                        .name
                        .eq(&bos_session_template_configuration_name)
                });

        if cfs_configuration_detail_opt.is_none() {
            eprintln!(
                "ERROR: BOS session template configuration not found in SAT file image list."
            );
            std::process::exit(1);
        }

        // Get image details
        let image_id = cfs_session_detail
            .status
            .unwrap()
            .artifacts
            .unwrap()
            .into_iter()
            .next()
            .unwrap()
            .result_id
            .unwrap();

        let image_detail_vec = image::http_client::get(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            None,
            Some(&image_id),
            None,
            None,
        )
        .await
        .unwrap_or(Vec::new());

        log::debug!("IMS image response:\n{:#?}", image_detail_vec);

        let ims_image_name = image_detail_vec.first().unwrap()["name"]
            .as_str()
            .unwrap()
            .to_string();
        let ims_image_etag = image_detail_vec.first().unwrap()["link"]["etag"]
            .as_str()
            .unwrap()
            .to_string();
        let ims_image_path = image_detail_vec.first().unwrap()["link"]["path"]
            .as_str()
            .unwrap()
            .to_string();
        let ims_image_type = image_detail_vec.first().unwrap()["link"]["type"]
            .as_str()
            .unwrap()
            .to_string();

        let bos_session_template_name = bos_session_template_yaml["name"]
            .as_str()
            .unwrap_or("")
            .to_string()
            .replace("__DATE__", &tag);

        let bos_session_template_hsm_groups: Vec<String> = if let Some(boot_sets_compute) =
            bos_session_template_yaml["bos_parameters"]["boot_sets"].get("compute")
        {
            boot_sets_compute["node_groups"]
                .as_sequence()
                .unwrap_or(&Vec::new())
                .iter()
                .map(|node| node.as_str().unwrap().to_string())
                .collect()
        } else if let Some(boot_sets_compute) =
            bos_session_template_yaml["bos_parameters"]["boot_sets"].get("uan")
        {
            boot_sets_compute["node_groups"]
                .as_sequence()
                .unwrap_or(&Vec::new())
                .iter()
                .map(|node| node.as_str().unwrap().to_string())
                .collect()
        } else {
            println!("No HSM group found in session_templates section in SAT file");
            std::process::exit(1);
        };

        // let cfs_configuration_name = bos_session_template_yaml["configuration"]
        //     .as_str()
        //     .unwrap_or("")
        //     .to_string();

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
            bos::template::BosTemplateRequest::new_for_hsm_group(
                bos_session_template_configuration_name,
                bos_session_template_name,
                ims_image_name,
                ims_image_path,
                ims_image_type,
                ims_image_etag,
                hsm_group,
            );

        log::debug!(
            "create BOS session template payload:\n{:#?}",
            create_bos_session_template_payload
        );

        let create_bos_session_template_resp = template::http_client::post(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            &create_bos_session_template_payload,
        )
        .await;

        log::debug!(
            "Create BOS session template response:\n{:#?}",
            create_bos_session_template_resp
        );

        if create_bos_session_template_resp.is_err() {
            eprintln!("BOS session template creation failed");
            std::process::exit(1);
        }

        // Create BOS session. Note: reboot operation shuts down the nodes and don't bring them back
        // up... hence we will split the reboot into 2 operations shutdown and start

        // Get nodes members of HSM group
        // Get HSM group details
        let hsm_group_details = hsm::http_client::get_hsm_group(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            hsm_group,
        )
        .await;

        log::debug!("HSM group response:\n{:#?}", hsm_group_details);

        // Get list of xnames in HSM group
        let nodes: Vec<String> = hsm_group_details.unwrap()["members"]["ids"]
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
        let create_bos_boot_session_resp = bos::session::http_client::post(
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

        // Audit
        let jwt_claims = get_claims_from_jwt_token(shasta_token).unwrap();

        log::info!(target: "app::audit", "User: {} ({}) ; Operation: Apply cluster", jwt_claims["name"].as_str().unwrap(), jwt_claims["preferred_username"].as_str().unwrap());
    }
}
