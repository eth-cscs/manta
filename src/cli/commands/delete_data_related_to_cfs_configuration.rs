use core::time;
use std::collections::HashMap;

use std::io::{self, Write};

use chrono::NaiveDateTime;
use comfy_table::Table;
use dialoguer::{theme::ColorfulTheme, Confirm};
use csm_rs::bss::bootparameters::BootParameters;
use csm_rs::cfs::configuration::csm_rs::r#struct::cfs_configuration_response::v2::CfsConfigurationResponse;
use csm_rs::{bos, cfs};
use serde_json::Value;

use crate::{
    cli::commands::delete_data_related_to_cfs_configuration,
    common::node_ops::get_node_vec_booting_image,
};

pub async fn delete_data_related_cfs_configuration(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    hsm_name_available_vec: Vec<String>,
    configuration_name_opt: Option<&String>,
    configuration_name_pattern: Option<&String>,
    since_opt: Option<NaiveDateTime>,
    until_opt: Option<NaiveDateTime>,
    yes: &bool,
) {
    /* if !hsm_name_available_vec.contains(hsm_group_name_opt.unwrap()) {
        eprintln!(
            "No access to HSM group {}. Exit",
            hsm_group_name_opt.unwrap()
        );
        std::process::exit(1);
    } */

    // COLLECT SITE WIDE DATA FOR VALIDATION
    //

    // Check CFS configurations to delete not used as a desired configuration
    //
    // Get all CFS components in CSM
    let cfs_components: Vec<Value> =
        csm_rs::cfs::component::shasta::http_client::v3::get_multiple_components(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            None,
            None,
        )
        .await
        .unwrap();

    // Check images related to CFS configurations to delete are not used to boot nodes. For
    // this we need to get images from both CFS session and BOS sessiontemplate because CSCS staff
    //
    // Get all BSS boot params
    let boot_param_vec: Vec<BootParameters> = csm_rs::bss::bootparameters::http_client::get(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        &vec![],
    )
    .await
    .unwrap();

    // Get all CFS configurations in CSM
    let mut cfs_configuration_vec: Vec<CfsConfigurationResponse> =
        cfs::configuration::csm_rs::http_client::get(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            None,
        )
        .await
        .unwrap();

    // Filter CFS configurations related to HSM group, configuration name or configuration name
    // pattern
    cfs::configuration::csm_rs::utils::filter(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        &mut cfs_configuration_vec,
        configuration_name_pattern.map(|elem| elem.as_str()),
        &hsm_name_available_vec,
        None,
    )
    .await;

    // Filter CFS configurations based on user input (date range or configuration name)
    if let (Some(since), Some(until)) = (since_opt, until_opt) {
        cfs_configuration_vec.retain(|cfs_configuration| {
            let date = chrono::DateTime::parse_from_rfc3339(&cfs_configuration.last_updated)
                .unwrap()
                .naive_utc();

            since <= date && date < until
        });
    } else if let Some(cfs_configuration_name) = configuration_name_opt {
        cfs_configuration_vec.retain(|cfs_configuration| {
            cfs_configuration
                .name
                .eq_ignore_ascii_case(cfs_configuration_name)
        });
    }

    // Get list CFS configuration names
    let mut cfs_configuration_name_vec = cfs_configuration_vec
        .iter()
        .map(|configuration_value| configuration_value.name.clone())
        .collect::<Vec<String>>();

    // Check images related to CFS configurations to delete are not used to boot nodes. For
    // this we need to get images from both CFS session and BOS sessiontemplate because CSCS staff
    // deletes all CFS sessions every now and then
    //
    // Get all BOS session templates
    let mut bos_sessiontemplate_value_vec = csm_rs::bos::template::csm_rs::http_client::get(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        None,
    )
    .await
    .unwrap();

    // Filter BOS sessiontemplate related to a HSM group
    csm_rs::bos::template::csm_rs::utils::filter(
        &mut bos_sessiontemplate_value_vec,
        &hsm_name_available_vec,
        &Vec::new(),
        // cfs_configuration_name_opt.map(|elem| elem.as_str()),
        None,
    )
    .await;

    // Filter BOS sessiontemplate containing /configuration/name field
    bos_sessiontemplate_value_vec.retain(|bos_sessiontemplate| {
        cfs_configuration_name_vec.contains(
            bos_sessiontemplate
                .cfs
                .as_ref()
                .unwrap()
                .configuration
                .as_ref()
                .unwrap(),
        )
    });

    log::debug!(
        "BOS sessiontemplate filtered by HSM and configuration name:\n{:#?}",
        bos_sessiontemplate_value_vec
    );

    // Get CFS configurations related with BOS sessiontemplate
    let cfs_configuration_name_from_bos_sessiontemplate_value_iter = bos_sessiontemplate_value_vec
        .iter()
        .map(|bos_sessiontemplate_value| {
            bos_sessiontemplate_value
                .cfs
                .as_ref()
                .unwrap()
                .configuration
                .as_ref()
                .unwrap()
        });

    // Check images related to CFS configurations to delete are not used to boot nodes. For
    // this we need to get images from both CFS session and BOS sessiontemplate because CSCS staff
    // deletes all CFS sessions every now and then
    //
    // Get all CFS sessions
    let mut cfs_session_vec = csm_rs::cfs::session::csm_rs::http_client::get(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        None,
        None,
        None,
        None,
        None,
    )
    .await
    .unwrap();

    // Filter CFS sessions related to a HSM group
    // NOTE: Admins (pa-admin) are the only ones who can delete generic sessions
    let keep_generic_sessions = csm_rs::common::jwt_ops::is_user_admin(shasta_token).unwrap();

    csm_rs::cfs::session::csm_rs::utils::filter_by_hsm(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        &mut cfs_session_vec,
        &hsm_name_available_vec,
        None,
        keep_generic_sessions,
    )
    .await;

    // Filter CFS sessions containing /configuration/name field
    cfs_session_vec.retain(|cfs_session_value| {
        cfs_configuration_name_vec.contains(
            cfs_session_value
                .configuration
                .as_ref()
                .unwrap()
                .name
                .as_ref()
                .unwrap(),
        )
    });

    // Get CFS configurations related with CFS sessions
    let cfs_configuration_name_from_cfs_sessions =
        cfs_session_vec.iter().map(|cfs_session_value| {
            cfs_session_value
                .configuration
                .as_ref()
                .unwrap()
                .name
                .as_ref()
                .unwrap()
        });

    // Get list of CFS configuration names related to CFS sessions and BOS sessiontemplates
    cfs_configuration_name_vec = cfs_configuration_name_from_bos_sessiontemplate_value_iter
        .chain(cfs_configuration_name_from_cfs_sessions)
        .cloned()
        .collect::<Vec<String>>();
    cfs_configuration_name_vec.sort();
    cfs_configuration_name_vec.dedup();

    // Get list of CFS configuration serde values related to CFS sessions and BOS
    // sessiontemplates
    cfs_configuration_vec.retain(|cfs_configuration_value| {
        cfs_configuration_name_vec.contains(&cfs_configuration_value.name)
    });

    // Get image ids from CFS sessions related to CFS configuration to delete
    let image_id_from_cfs_session_vec =
        cfs::session::csm_rs::utils::get_image_id_from_cfs_session_vec(&cfs_session_vec);

    /* // Get image ids from BOS session template related to CFS configuration to delete
    // NOTE: This assumes runtime configuration and boot image configuration are the same
    // NOTE: DON'T DELETE IMAGES FROM BOS SESSIONTEMPLATE BASED ON CONFGURATION NAME SINCE BOOT
    // IMAGE MAY HABE BEEN CREATED USING A DIFFERENT CONFIGURATION
    let image_id_from_bos_sessiontemplate_vec =
        bos::template::shasta::utils::get_image_id_from_bos_sessiontemplate_vec(
            &bos_sessiontemplate_value_vec,
        ); */

    /* // Combine image ids from CFS session and BOS session template
    let mut image_id_vec: Vec<&str> = [
        image_id_from_cfs_session_vec
            .iter()
            .map(|elem| elem.as_str())
            .collect::<Vec<&str>>(),
        image_id_from_bos_sessiontemplate_vec
            .iter()
            .map(|elem| elem.as_str())
            .collect::<Vec<&str>>(),
    ]
    .concat(); */
    let mut image_id_vec = image_id_from_cfs_session_vec
        .iter()
        .map(|elem| elem.as_str())
        .collect::<Vec<&str>>();

    image_id_vec.sort();
    image_id_vec.dedup();

    log::info!("Image IDs found: {:?}", image_id_vec);

    // Filter list of image ids by removing the ones that does not exists. This is because we
    // currently image id list contains the values from CFS session and BOS sessiontemplate
    // which does not means the image still exists (the image perse could have been deleted
    // previously and the CFS session and BOS sessiontemplate not being cleared)
    let mut image_id_filtered_vec: Vec<&str> = Vec::new();
    for image_id in image_id_vec {
        if !csm_rs::ims::image::csm_rs::http_client::get(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            // &hsm_name_available_vec,
            Some(image_id),
        )
        .await
        .unwrap_or(vec![])
        .is_empty()
        {
            log::info!("Artifact for image ID {} exists", image_id);
            image_id_filtered_vec.push(image_id);
        } else {
            log::info!("Artifact for image ID {} does NOT exists", image_id);
            image_id_filtered_vec.push(image_id);
        }
    }

    image_id_vec = image_id_filtered_vec;

    log::info!(
                "Image id related to CFS sessions and/or BOS sessiontemplate related to CFS configurations filtered by user input: {:?}",
                image_id_vec
            );

    log::info!("Image ids to delete: {:?}", image_id_vec);

    // Get list of CFS session name, CFS configuration name and image id for CFS sessions which
    // created an image
    let cfs_session_cfs_configuration_image_id_tuple_vec: Vec<(String, String, String)> =
        cfs_session_vec
            .iter()
            .filter(|cfs_session| cfs_session.get_first_result_id().is_some())
            .map(|cfs_session| {
                (
                    cfs_session.name.clone().unwrap(),
                    cfs_session
                        .get_configuration_name()
                        .as_ref()
                        .unwrap()
                        .clone(),
                    cfs_session
                        .get_first_result_id()
                        .unwrap_or("".to_string())
                        .clone(),
                )
            })
            .collect();

    // Get list of BOS sessiontemplate name, CFS configuration name and image ids for compute nodes
    let mut bos_sessiontemplate_cfs_configuration_image_id_tuple_vec: Vec<(&str, &str, &str)> =
        Vec::new();

    for bos_sessiontemplate_value in &bos_sessiontemplate_value_vec {
        let cfs_session_name: &str = bos_sessiontemplate_value.name.as_ref().unwrap();
        let cfs_configuration_name: &String = bos_sessiontemplate_value
            .cfs
            .as_ref()
            .unwrap()
            .configuration
            .as_ref()
            .unwrap();

        for boot_set_prop in bos_sessiontemplate_value
            .boot_sets
            .as_ref()
            .unwrap()
            .values()
        {
            let image_id = if let Some(image_path_value) = boot_set_prop.path.as_ref() {
                image_path_value
                    .strip_prefix("s3://boot-images/")
                    .unwrap()
                    .strip_suffix("/manifest.json")
                    .unwrap()
            } else {
                ""
            };

            bos_sessiontemplate_cfs_configuration_image_id_tuple_vec.push((
                cfs_session_name,
                cfs_configuration_name,
                image_id,
            ));
        }
    }

    // Group image ids by CFS configuration names
    let mut cfs_configuration_image_id: HashMap<&str, Vec<&str>> = HashMap::new();

    for (_, cfs_configuration, image_id) in
        &bos_sessiontemplate_cfs_configuration_image_id_tuple_vec
    {
        cfs_configuration_image_id
            .entry(cfs_configuration)
            .and_modify(|image_vec| image_vec.push(image_id))
            .or_insert(vec![image_id]);
    }

    for (_, cfs_configuration, image_id) in &cfs_session_cfs_configuration_image_id_tuple_vec {
        cfs_configuration_image_id
            .entry(cfs_configuration)
            .and_modify(|image_vec| image_vec.push(image_id))
            .or_insert(vec![image_id]);
    }

    // VALIDATION
    //
    let mut cfs_configuration_name_used_to_configure_nodes_vec: Vec<&str> = Vec::new();
    let mut image_id_used_to_boot_nodes_vec: Vec<&str> = Vec::new();

    // We can't allow any data deletion operation which can jeopardize the system stability,
    // therefore we will filter the list of the CFS configurations and Images used to configure or boot nodes
    for (cfs_configuration_name, mut image_id_vec) in cfs_configuration_image_id {
        let mut nodes_using_cfs_configuration_as_dessired_configuration_vec = cfs_components
            .iter()
            .filter(|cfs_component| {
                cfs_component["desired_config"]
                    .as_str()
                    .unwrap()
                    .eq(cfs_configuration_name)
            })
            .map(|cfs_component| cfs_component["id"].as_str().unwrap())
            .collect::<Vec<&str>>();

        if !nodes_using_cfs_configuration_as_dessired_configuration_vec.is_empty() {
            cfs_configuration_name_used_to_configure_nodes_vec.push(cfs_configuration_name);

            nodes_using_cfs_configuration_as_dessired_configuration_vec.sort();

            eprintln!(
                    "CFS configuration '{}' can't be deleted. Reason:\nCFS configuration '{}' used as desired configuration for nodes: {}",
                    cfs_configuration_name, cfs_configuration_name, nodes_using_cfs_configuration_as_dessired_configuration_vec.join(", "));
        }

        image_id_vec.dedup();

        for image_id in &image_id_vec {
            let node_vec = get_node_vec_booting_image(image_id, &boot_param_vec);

            if !node_vec.is_empty() {
                image_id_used_to_boot_nodes_vec.push(image_id);
                eprintln!(
                    "Image '{}' used to boot nodes: {}",
                    image_id,
                    node_vec.join(", ")
                );
            }
        }
    }

    // Get final list of CFS configuration serde values related to CFS sessions and BOS
    // sessiontemplates and excluding the CFS sessions to keep (in case user decides to
    // force the deletion operation)
    cfs_configuration_vec.retain(|cfs_configuration_value| {
        !cfs_configuration_name_used_to_configure_nodes_vec
            .contains(&cfs_configuration_value.name.as_str())
    });

    let cfs_session_cfs_configuration_image_id_tuple_filtered_vec: Vec<(String, String, String)>;
    let bos_sessiontemplate_cfs_configuration_image_id_tuple_filtered_vec: Vec<(&str, &str, &str)>;

    // EVALUATE IF NEED TO CONTINUE.
    // CHECK IF ANY CFS CONFIGURAION OR IMAGE IS CURRENTLY USED TO CONFIGURE OR BOOT NODES
    if !cfs_configuration_name_used_to_configure_nodes_vec.is_empty()
        || !image_id_used_to_boot_nodes_vec.is_empty()
    {
        // There are CFS configuraions or Images currently used by nodes. Better to be safe and
        // stop the process
        eprintln!("Either images or configurations used by other clusters/nodes. Exit");
        std::process::exit(1);
    } else {
        // We are safe to delete, none of the data selected for deletion is currently used as
        // neither configure nor boot the nodes
        cfs_configuration_name_vec.retain(|cfs_configuration_name| {
            !cfs_configuration_name_used_to_configure_nodes_vec
                .contains(&cfs_configuration_name.as_str())
        });

        image_id_vec.retain(|image_id| !image_id_used_to_boot_nodes_vec.contains(image_id));

        cfs_session_cfs_configuration_image_id_tuple_filtered_vec =
            cfs_session_cfs_configuration_image_id_tuple_vec
                .iter()
                .filter(|(_, cfs_configuration_name, image_id)| {
                    !cfs_configuration_name_used_to_configure_nodes_vec
                        .contains(&cfs_configuration_name.as_str())
                        && !image_id_used_to_boot_nodes_vec.contains(&image_id.as_str())
                })
                .cloned()
                .collect();

        bos_sessiontemplate_cfs_configuration_image_id_tuple_filtered_vec =
            bos_sessiontemplate_cfs_configuration_image_id_tuple_vec
                .into_iter()
                .filter(|(_, cfs_configuration_name, image_id)| {
                    !cfs_configuration_name_used_to_configure_nodes_vec
                        .contains(cfs_configuration_name)
                        && !image_id_used_to_boot_nodes_vec.contains(image_id)
                })
                .collect();
    }

    // EXIT IF THERE IS NO DATA TO DELETE
    if cfs_configuration_name_vec.is_empty()
        && image_id_vec.is_empty()
        && cfs_session_cfs_configuration_image_id_tuple_filtered_vec.is_empty()
        && bos_sessiontemplate_cfs_configuration_image_id_tuple_filtered_vec.is_empty()
    {
        print!("Nothing to delete.");
        if configuration_name_opt.is_some() {
            print!(
                " Could not find information related to CFS configuration '{}'",
                configuration_name_opt.unwrap()
            );
        }
        if since_opt.is_some() && until_opt.is_some() {
            print!(
                " Could not find information between dates {} and {}",
                since_opt.unwrap(),
                until_opt.unwrap()
            );
        }
        // print!(" in HSM '{}'. Exit", hsm_group_name_opt.unwrap());
        io::stdout().flush().unwrap();

        std::process::exit(0);
    }

    // PRINT SUMMARY/DATA TO DELETE
    //
    println!("CFS sessions to delete:");

    let mut cfs_session_table = Table::new();

    cfs_session_table.set_header(vec!["Name", "Configuration", "Image ID"]);

    for cfs_session in &cfs_session_vec {
        cfs_session_table.add_row(vec![
            cfs_session.name.as_ref().unwrap_or(&"".to_string()),
            &cfs_session.get_configuration_name().unwrap_or_default(),
            &cfs_session.get_first_result_id().unwrap_or_default(),
        ]);
    }

    println!("{cfs_session_table}");

    println!("BOS sessiontemplates to delete:");

    let mut bos_sessiontemplate_table = Table::new();

    bos_sessiontemplate_table.set_header(vec!["Name", "Configuration", "Image ID"]);

    for bos_sessiontemplate_tuple in
        &bos_sessiontemplate_cfs_configuration_image_id_tuple_filtered_vec
    {
        bos_sessiontemplate_table.add_row(vec![
            bos_sessiontemplate_tuple.0,
            bos_sessiontemplate_tuple.1,
            bos_sessiontemplate_tuple.2,
        ]);
    }

    println!("{bos_sessiontemplate_table}");

    println!("CFS configurations to delete:");

    let mut cfs_configuration_table = Table::new();

    cfs_configuration_table.set_header(vec!["Name", "Last Update"]);

    for cfs_configuration_value in cfs_configuration_vec {
        cfs_configuration_table.add_row(vec![
            cfs_configuration_value.name,
            cfs_configuration_value.last_updated,
        ]);
    }

    println!("{cfs_configuration_table}");

    println!("Images to delete:");

    let mut image_id_table = Table::new();

    image_id_table.set_header(vec!["Image ID"]);

    for image_id in &image_id_vec {
        image_id_table.add_row(vec![image_id]);
    }

    println!("{image_id_table}");

    // ASK USER FOR CONFIRMATION
    //
    if !*yes {
        if Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Please revew the data above and confirm to delete:")
            .interact()
            .unwrap()
        {
            println!("Continue");
        } else {
            println!("Cancelled by user. Aborting.");
            std::process::exit(0);
        }
    }

    // DELETE DATA
    //
    delete_data_related_to_cfs_configuration::delete(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        cfs_configuration_name_vec
            .iter()
            .map(|cfs_config| cfs_config.as_str())
            .collect(),
        image_id_vec,
        // &cfs_components,
        cfs_session_cfs_configuration_image_id_tuple_filtered_vec
            .iter()
            .map(|(session, _, _)| session.as_str())
            .collect(),
        bos_sessiontemplate_cfs_configuration_image_id_tuple_filtered_vec
            .into_iter()
            .map(|(sessiontemplate, _, _)| sessiontemplate)
            .collect(),
        // &boot_param_vec,
    )
    .await;
}

/// Deletes CFS configuration, CFS session, BOS sessiontemplate, BOS session and images related to
/// a CFS configuration. This method is safe. It checks if CFS configuration to delete is assigned
/// to a CFS component as a 'desired configuration' and also checks if image related to CFS
/// configuration is used as a boot image of any node in the system.
pub async fn delete(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    cfs_configuration_name_vec: Vec<&str>,
    image_id_vec: Vec<&str>,
    cfs_session_name_vec: Vec<&str>,
    bos_sessiontemplate_name_vec: Vec<&str>,
) {
    // DELETE DATA
    //
    // DELETE IMAGES
    for image_id in image_id_vec {
        log::info!("Deleting IMS image '{}'", image_id);
        let image_deleted_value_rslt = csm_rs::ims::image::shasta::http_client::delete(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            image_id,
        )
        .await;

        // process api response
        match image_deleted_value_rslt {
            Ok(_) => println!("IMS image deleted: {}", image_id),
            Err(error) => {
                if error.status().unwrap().eq(&404) {
                    eprintln!(
                        "Artifact related to image id '{}' not found. Continue",
                        image_id
                    );
                }
            }
        }
    }

    // DELETE BOS SESSIONS
    let bos_session_vec = bos::session::shasta::http_client::v2::get(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        None,
    )
    .await
    .unwrap();

    // Match BOS SESSIONS with the BOS SESSIONTEMPLATE RELATED
    for bos_session in bos_session_vec {
        let bos_session_id = &bos_session.name.unwrap();
        log::info!("Deleting BOS sesion '{}'", bos_session_id);

        if bos_sessiontemplate_name_vec.contains(&bos_session.template_name.as_str()) {
            bos::session::shasta::http_client::v2::delete(
                shasta_token,
                shasta_base_url,
                shasta_root_cert,
                &bos_session_id,
            )
            .await
            .unwrap();

            println!(
                "BOS session deleted: {}",
                bos_session_id // For some reason CSM API to delete a BOS
                               // session does not returns the BOS session
                               // ID in the payload...
            );
        } else {
            log::debug!("Ignoring BOS session template {}", bos_session_id);
        }
    }

    // DELETE CFS SESSIONS
    let max_attempts = 5;
    for cfs_session_name in cfs_session_name_vec {
        log::info!("Deleting IMS image '{}'", cfs_session_name);
        let mut counter = 0;
        loop {
            let deletion_rslt = cfs::session::shasta::http_client::v2::delete(
                shasta_token,
                shasta_base_url,
                shasta_root_cert,
                cfs_session_name,
            )
            .await;

            if deletion_rslt.is_err() && counter <= max_attempts {
                log::warn!("Could not delete CFS session {} attempt {} of {}, trying again in 2 seconds...", cfs_session_name, counter, max_attempts);
                tokio::time::sleep(time::Duration::from_secs(2)).await;
                counter += 1;
            } else if deletion_rslt.is_err() && counter > max_attempts {
                eprintln!(
                    "ERROR deleting CFS session {}, please delete it manually.",
                    cfs_session_name,
                );
                log::debug!("ERROR:\n{:#?}", deletion_rslt.unwrap_err());
                break;
            } else {
                println!("CfS session deleted: {}", cfs_session_name);
                break;
            }
        }
    }

    // DELETE BOS SESSIONTEMPLATES
    let max_attempts = 5;
    for bos_sessiontemplate_name in bos_sessiontemplate_name_vec {
        log::info!(
            "Deleting BOS sessiontemplate '{}'",
            bos_sessiontemplate_name
        );
        let mut counter = 0;
        loop {
            let deletion_rslt = csm_rs::bos::template::shasta::http_client::v2::delete(
                shasta_token,
                shasta_base_url,
                shasta_root_cert,
                bos_sessiontemplate_name,
            )
            .await;

            if deletion_rslt.is_err() && counter <= max_attempts {
                log::warn!("Could not delete BOS sessiontemplate {} attempt {} of {}, trying again in 2 seconds...", bos_sessiontemplate_name, counter, max_attempts);
                tokio::time::sleep(time::Duration::from_secs(2)).await;
                counter += 1;
            } else if deletion_rslt.is_err() && counter > max_attempts {
                eprintln!(
                    "ERROR deleting BOS sessiontemplate {}, please delete it manually.",
                    bos_sessiontemplate_name,
                );
                log::debug!("ERROR:\n{:#?}", deletion_rslt.unwrap_err());
                break;
            } else {
                println!("BOS sessiontemplate deleted: {}", bos_sessiontemplate_name);
                break;
            }
        }
    }

    // DELETE CFS CONFIGURATIONS
    let max_attempts = 5;
    for cfs_configuration in cfs_configuration_name_vec {
        log::info!("Deleting CFS configuration '{}'", cfs_configuration);
        let mut counter = 0;
        loop {
            let deletion_rslt = cfs::configuration::shasta::http_client::v2::delete(
                shasta_token,
                shasta_base_url,
                shasta_root_cert,
                cfs_configuration,
            )
            .await;

            if deletion_rslt.is_err() && counter <= max_attempts {
                log::warn!("Could not delete CFS configuration {} attempt {} of {}, trying again in 2 seconds...", cfs_configuration, counter, max_attempts);
                tokio::time::sleep(time::Duration::from_secs(2)).await;
                counter += 1;
            } else if deletion_rslt.is_err() && counter > max_attempts {
                eprintln!(
                    "ERROR deleting CFS configuration {}, please delete it manually.",
                    cfs_configuration,
                );
                log::debug!("ERROR:\n{:#?}", deletion_rslt.unwrap_err());
                break;
            } else {
                println!("CFS configuration deleted: {}", cfs_configuration);
                break;
            }
        }
    }
}
