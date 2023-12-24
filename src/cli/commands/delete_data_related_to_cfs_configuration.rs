use core::time;
use std::{collections::HashMap, thread};

use chrono::NaiveDateTime;
use comfy_table::Table;
use dialoguer::{theme::ColorfulTheme, Confirm};
use mesa::{bos, cfs};
use serde_json::Value;

use crate::{
    cli::commands::delete_data_related_to_cfs_configuration,
    common::node_ops::get_node_vec_booting_image,
};

pub async fn delete_data_related_cfs_configuration(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    hsm_group_name_opt: Option<&String>,
    hsm_name_available_vec: Vec<String>,
    cfs_configuration_name_opt: Option<&String>,
    since_opt: Option<NaiveDateTime>,
    until_opt: Option<NaiveDateTime>,
    force: &bool,
) {
    // COLLECT SITE WIDE DATA FOR VALIDATION
    //

    // Check dessired configuration not using any CFS configuration to delete
    //
    // Get all CFS components in CSM
    let cfs_components = mesa::cfs::component::shasta::http_client::get_multiple_components(
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
    let boot_param_vec = mesa::bss::http_client::get_boot_params(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        &[],
    )
    .await
    .unwrap();

    // Get all CFS configurations in CSM
    let mut cfs_configuration_value_vec = cfs::configuration::shasta::http_client::get(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        None,
    )
    .await
    .unwrap();

    // Filter CFS configurations based on user input
    if let (Some(since), Some(until)) = (since_opt, until_opt) {
        cfs_configuration_value_vec.retain(|cfs_configuration_value| {
            let date = chrono::DateTime::parse_from_rfc3339(
                cfs_configuration_value["lastUpdated"].as_str().unwrap(),
            )
            .unwrap()
            .naive_utc();

            since <= date && date < until
        });
    } else if let Some(cfs_configuration_name) = cfs_configuration_name_opt {
        cfs_configuration_value_vec.retain(|cfs_configuration_value| {
            cfs_configuration_value["name"]
                .as_str()
                .unwrap()
                .eq_ignore_ascii_case(cfs_configuration_name)
        });
    }

    // Get list CFS configuration names
    let mut cfs_configuration_name_vec = cfs_configuration_value_vec
        .iter()
        .map(|configuration_value| configuration_value["name"].as_str().unwrap())
        .collect::<Vec<&str>>();

    // Check images related to CFS configurations to delete are not used to boot nodes. For
    // this we need to get images from both CFS session and BOS sessiontemplate because CSCS staff
    // deletes all CFS sessions every now and then
    //
    // Get all BOS session templates
    let bos_sessiontemplate_value_vec = mesa::bos::template::shasta::http_client::get(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        &hsm_name_available_vec,
        None,
        Some(cfs_configuration_name_vec.clone()),
        None,
    )
    .await
    .unwrap();

    // Filter BOS sessiontemplates related to CFS configurations to be deleted
    //
    // Filter BOS sessiontemplate containing /cfs/configuration field
    /* bos_sessiontemplate_value_vec.retain(|bos_sessiontemplate_value| {
        cfs_configuration_name_vec.contains(
            &bos_sessiontemplate_value
                .pointer("/cfs/configuration")
                .unwrap()
                .as_str()
                .unwrap(),
        )
    }); */

    // Get CFS configurations related with BOS sessiontemplate
    let cfs_configuration_name_from_bos_sessiontemplate_value_iter = bos_sessiontemplate_value_vec
        .iter()
        .map(|bos_sessiontemplate_value| {
            bos_sessiontemplate_value
                .pointer("/cfs/configuration")
                .unwrap()
                .as_str()
                .unwrap()
        });

    // Check images related to CFS configurations to delete are not used to boot nodes. For
    // this we need to get images from both CFS session and BOS sessiontemplate because CSCS staff
    // deletes all CFS sessions every now and then
    //
    // Get all CFS sessions
    let mut cfs_session_value_vec = mesa::cfs::session::shasta::http_client::get(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        None,
        None,
    )
    .await
    .unwrap();

    mesa::cfs::session::shasta::http_client::filter(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        &mut cfs_session_value_vec,
        &hsm_name_available_vec,
        None,
    )
    .await;

    // Filter CFS sessions containing /configuration/name field
    cfs_session_value_vec.retain(|cfs_session_value| {
        cfs_configuration_name_vec.contains(
            &cfs_session_value
                .pointer("/configuration/name")
                .unwrap()
                .as_str()
                .unwrap(),
        )
    });

    // Get CFS configurations related with CFS sessions
    let cfs_configuration_name_from_cfs_sessions =
        cfs_session_value_vec.iter().map(|cfs_session_value| {
            cfs_session_value
                .pointer("/configuration/name")
                .unwrap()
                .as_str()
                .unwrap()
        });

    // Get list of CFS configuration names related to CFS sessions and BOS sessiontemplates
    cfs_configuration_name_vec = cfs_configuration_name_from_bos_sessiontemplate_value_iter
        .chain(cfs_configuration_name_from_cfs_sessions)
        .collect::<Vec<&str>>();
    cfs_configuration_name_vec.sort();
    cfs_configuration_name_vec.dedup();

    // Get list of CFS configuration serde values related to CFS sessions and BOS
    // sessiontemplates
    cfs_configuration_value_vec.retain(|cfs_configuration_value| {
        cfs_configuration_name_vec.contains(&cfs_configuration_value["name"].as_str().unwrap())
    });

    // Get image ids from CFS sessions and BOS sessiontemplate related to CFS configuration to delete
    let image_id_from_cfs_session_vec =
        cfs::session::shasta::utils::get_image_id_from_cfs_session_vec(&cfs_session_value_vec);

    // Get image ids from BOS session template related to CFS configuration to delete
    let image_id_from_bos_sessiontemplate_vec =
        bos::template::shasta::utils::get_image_id_from_bos_sessiontemplate_vec(
            &bos_sessiontemplate_value_vec,
        );

    // Combine image ids from CFS session and BOS session template
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
    .concat();

    image_id_vec.sort();
    image_id_vec.dedup();

    // Filter list of image ids by removing the ones that does not exists. This is because we
    // currently image id list contains the values from CFS session and BOS sessiontemplate
    // which does not means the image still exists (the image perse could have been deleted
    // previously and the CFS session and BOS sessiontemplate not being cleared)
    let mut image_id_filtered_vec: Vec<&str> = Vec::new();
    for image_id in image_id_vec {
        if !mesa::ims::image::http_client::get_struct(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            // &hsm_name_available_vec,
            Some(image_id),
            None,
            None,
        )
        .await
        .unwrap()
        .is_empty()
        {
            log::info!("Image ID {} exists", image_id);
            image_id_filtered_vec.push(image_id);
        }
    }

    image_id_vec = image_id_filtered_vec;

    log::info!(
                "Image id related to CFS sessions and/or BOS sessiontemplate related to CFS configurations filtered by user input: {:?}",
                image_id_vec
            );

    log::info!("Image ids to delete: {:?}", image_id_vec);

    // Get list of CFS session name, CFS configuration name and image id
    let cfs_session_cfs_configuration_image_id_tuple_vec: Vec<(&str, &str, &str)> =
        cfs_session_value_vec
            .iter()
            .map(|cfs_session_value| {
                (
                    cfs_session_value["name"].as_str().unwrap(),
                    cfs_session_value
                        .pointer("/configuration/name")
                        .unwrap()
                        .as_str()
                        .unwrap(),
                    cfs_session_value
                        .pointer("/status/artifacts/0/result_id")
                        .and_then(|result_id| result_id.as_str())
                        .unwrap_or(""),
                )
            })
            .collect();

    // Get list of BOS sessiontemplate name, CFS configuration name and image ids for compute nodes
    let mut bos_sessiontemplate_cfs_configuration_image_id_tuple_vec: Vec<(&str, &str, &str)> =
        Vec::new();

    for bos_sessiontemplate_value in &bos_sessiontemplate_value_vec {
        let cfs_session_name = bos_sessiontemplate_value["name"].as_str().unwrap();
        let cfs_configuration_name = bos_sessiontemplate_value
            .pointer("/cfs/configuration")
            .unwrap()
            .as_str()
            .unwrap();
        for (_, boot_set_prop) in bos_sessiontemplate_value["boot_sets"].as_object().unwrap() {
            let image_id = if let Some(image_path_value) = boot_set_prop.get("path") {
                image_path_value
                    .as_str()
                    .unwrap()
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
    let mut cfs_configuration_name_to_keep_vec: Vec<&str> = Vec::new();
    let mut image_id_to_keep_vec: Vec<&str> = Vec::new();

    for (cfs_configuration_name, image_id_vec) in cfs_configuration_image_id {
        let mut nodes_using_cfs_configuration_as_dessired_configuration_vec = cfs_components
            .iter()
            .filter(|cfs_component| {
                cfs_component["desiredConfig"]
                    .as_str()
                    .unwrap()
                    .eq(cfs_configuration_name)
            })
            .map(|cfs_component| cfs_component["id"].as_str().unwrap())
            .collect::<Vec<&str>>();
        if !nodes_using_cfs_configuration_as_dessired_configuration_vec.is_empty() {
            cfs_configuration_name_to_keep_vec.push(cfs_configuration_name);

            nodes_using_cfs_configuration_as_dessired_configuration_vec.sort();

            eprintln!(
                    "CFS configuration '{}' can't be deleted. Reason:\nCFS configuration '{}' used as desired configuration for nodes: {}",
                    cfs_configuration_name, cfs_configuration_name, nodes_using_cfs_configuration_as_dessired_configuration_vec.join(", "));
        }

        for image_id in &image_id_vec {
            let node_vec = get_node_vec_booting_image(image_id, &boot_param_vec);

            if !node_vec.is_empty() {
                image_id_to_keep_vec.push(image_id);
                eprintln!(
                    "Image '{}' used to boot nodes: {}",
                    image_id,
                    node_vec.join(", ")
                );
            }
        }
    }

    /* println!("---------------------------------------");
    println!(
        "DEBUG - cfs configuration: {:?}",
        cfs_configuration_name_vec
    );
    println!(
        "DEBUG - cfs configuration to keep: {:?}",
        cfs_configuration_name_to_keep_vec
    );
    println!("DEBUG - image ids: {:?}", image_id_vec);
    println!("DEBUG - image ids to keep: {:?}", image_id_to_keep_vec);
    println!(
        "DEBUG - cfs session, cfs configuration, image id: {:?}",
        cfs_session_cfs_configuration_image_id_tuple_vec
    );
    println!(
        "DEBUG - bos sessiontemplate, cfs configuration, image id: {:?}",
        bos_sessiontemplate_cfs_configuration_image_id_tuple_vec
    ); */

    // Get final list of CFS configuration serde values related to CFS sessions and BOS
    // sessiontemplates and excluding the CFS sessions to keep (in case user decides to
    // force the deletion operation)
    cfs_configuration_value_vec.retain(|cfs_configuration_value| {
        !cfs_configuration_name_to_keep_vec
            .contains(&cfs_configuration_value["name"].as_str().unwrap())
    });

    let cfs_session_cfs_configuration_image_id_tuple_filtered_vec: Vec<(&str, &str, &str)>;
    let bos_sessiontemplate_cfs_configuration_image_id_tuple_filtered_vec: Vec<(&str, &str, &str)>;

    if !cfs_configuration_name_to_keep_vec.is_empty() || !image_id_to_keep_vec.is_empty() {
        if *force {
            cfs_configuration_name_vec.retain(|cfs_configuration_name| {
                !cfs_configuration_name_to_keep_vec.contains(cfs_configuration_name)
            });

            image_id_vec.retain(|image_id| !image_id_to_keep_vec.contains(image_id));

            cfs_session_cfs_configuration_image_id_tuple_filtered_vec =
                cfs_session_cfs_configuration_image_id_tuple_vec
                    .into_iter()
                    .filter(|(_, cfs_configuration_name, image_id)| {
                        !cfs_configuration_name_to_keep_vec.contains(cfs_configuration_name)
                            && !image_id_to_keep_vec.contains(image_id)
                    })
                    .collect();

            bos_sessiontemplate_cfs_configuration_image_id_tuple_filtered_vec =
                bos_sessiontemplate_cfs_configuration_image_id_tuple_vec
                    .into_iter()
                    .filter(|(_, cfs_configuration_name, image_id)| {
                        !cfs_configuration_name_to_keep_vec.contains(cfs_configuration_name)
                            && !image_id_to_keep_vec.contains(image_id)
                    })
                    .collect();
        } else {
            // User don't want to force and there are cfs configurations or images used in the
            // system. EXIT
            eprintln!("Exit");
            std::process::exit(1);
        }
    } else {
        cfs_session_cfs_configuration_image_id_tuple_filtered_vec = Vec::new();
        bos_sessiontemplate_cfs_configuration_image_id_tuple_filtered_vec = Vec::new();
    }

    println!("---------------------------------------");
    println!(
        "DEBUG - cfs configuration name: {:?}",
        cfs_configuration_name_vec
    );
    println!("DEBUG - image id: {:?}", image_id_vec);
    println!(
        "DEBUG - cfs session cfs configuration image id tuple filtered: {:?}",
        cfs_session_cfs_configuration_image_id_tuple_filtered_vec
    );
    println!(
        "DEBUG - bos sessiontempalte cfs configuration image id tuple filtered: {:?}",
        bos_sessiontemplate_cfs_configuration_image_id_tuple_filtered_vec
    );

    /* // VALIDATION
    //
    // Process CFS configurations to delete one by one
    let mut cfs_configuration_to_keep_vec: Vec<&str> = Vec::new();
    for cfs_configuration_name in &cfs_configuration_name_vec {
        // Check dessired configuration not using any CFS configuration to delete
        let mut nodes_using_cfs_configuration_as_dessired_configuration_vec =
            cfs_components
                .iter()
                .filter(|cfs_component| {
                    cfs_component["desiredConfig"]
                        .as_str()
                        .unwrap()
                        .eq(*cfs_configuration_name)
                })
                .map(|cfs_component| cfs_component["id"].as_str().unwrap())
                .collect::<Vec<&str>>();

        if !nodes_using_cfs_configuration_as_dessired_configuration_vec.is_empty() {
            cfs_configuration_to_keep_vec.push(cfs_configuration_name);

            nodes_using_cfs_configuration_as_dessired_configuration_vec.sort();

            eprintln!(
            "CFS configuration {} can't be deleted. Reason:\nCFS configuration {} used as desired configuration for nodes: {}",
            cfs_configuration_name, cfs_configuration_name, nodes_using_cfs_configuration_as_dessired_configuration_vec.join(", "));
        }
    }

    // for cfs_configuration_name in &cfs_configuration_name_vec {
    // Check images related to CFS configurations to delete are not used to boot nodes. For
    // this we need to get images from both CFS session and BOS sessiontemplate because CSCS staff
    let mut image_id_to_keep_vec: Vec<&str> = Vec::new();

    // Check images related to CFS configurations to delete are not used to boot nodes. For
    // this we need to get images from both CFS session and BOS sessiontemplate because CSCS staff
    let mut boot_image_node_vec: Vec<(&str, Vec<String>)> = Vec::new();

    for image_id in &image_id_vec {
        let nodes = get_node_vec_booting_image(image_id, &boot_param_vec);

        if !nodes.is_empty() {
            boot_image_node_vec.push((image_id, nodes));
        }
    }

    if !boot_image_node_vec.is_empty() {
        // cfs_configuration_to_keep_vec.push(cfs_configuration_name);

        image_id_to_keep_vec.extend(
            boot_image_node_vec
                .iter()
                .flat_map(|(_, nodes)| nodes)
                .collect(),
        );

        eprintln!(
            "Image based on CFS configuration {} can't be deleted. Reason:",
            cfs_configuration_name
        );
        for (image_id, node_vec) in boot_image_node_vec {
            eprintln!("Image id {} used to boot nodes:\n{:?}", image_id, node_vec);
        }
        std::process::exit(1);
    }
    // }

    if !cfs_configuration_to_keep_vec.is_empty() || !image_id_to_keep_vec.is_empty() {
        if *force {
            cfs_configuration_name_vec.retain(|cfs_configuration_name| {
                !cfs_configuration_to_keep_vec.contains(cfs_configuration_name)
            });
            image_id_vec.retain(|image_id| !image_id_to_keep_vec.contains(image_id));
        } else {
            // User don't want to force and there are cfs configurations or images used in the
            // system. EXIT
            std::process::exit(1);
        }
    } */

    // EVALUATE IF NEED TO CONTINUE. EXIT IF THERE IS NO DATA TO DELETE
    //
    if cfs_configuration_name_vec.is_empty()
        && image_id_vec.is_empty()
        && cfs_session_cfs_configuration_image_id_tuple_filtered_vec.is_empty()
        && bos_sessiontemplate_cfs_configuration_image_id_tuple_filtered_vec.is_empty()
    {
        print!("Nothing to delete.");
        if cfs_configuration_name_opt.is_some() {
            print!(
                " Could not find information related to CFS configuration '{}'",
                cfs_configuration_name_opt.unwrap()
            );
        }
        if since_opt.is_some() && until_opt.is_some() {
            print!(
                " Could not find information between dates {} and {}",
                since_opt.unwrap(),
                until_opt.unwrap()
            );
        }
        print!(" in HSM '{}'. Exit", hsm_group_name_opt.unwrap());

        std::process::exit(0);
    }

    // PRINT SUMMARY/DATA TO DELETE
    //
    println!("CFS sessions to delete:");

    let mut cfs_session_table = Table::new();

    cfs_session_table.set_header(vec!["Name", "Configuration", "Image ID"]);

    for cfs_session_tuple in &cfs_session_cfs_configuration_image_id_tuple_filtered_vec {
        cfs_session_table.add_row(vec![
            cfs_session_tuple.0,
            cfs_session_tuple.1,
            cfs_session_tuple.2,
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

    for cfs_configuration_value in &cfs_configuration_value_vec {
        cfs_configuration_table.add_row(vec![
            cfs_configuration_value["name"].as_str().unwrap(),
            cfs_configuration_value["lastUpdated"].as_str().unwrap(),
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
    if !*force {
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

    /* println!("---------------------------------------");
    println!(
        "DEBUG - cfs configuration name: {:?}",
        cfs_configuration_name_vec
    );
    println!("DEBUG - image id: {:?}", image_id_vec);
    println!(
        "DEBUG - cfs session cfs configuration image id tuple filtered: {:?}",
        cfs_session_cfs_configuration_image_id_tuple_filtered_vec
    );
    println!(
        "DEBUG - bos sessiontempalte cfs configuration image id tuple filtered: {:?}",
        bos_sessiontemplate_cfs_configuration_image_id_tuple_filtered_vec
    );
    println!("DEBUG - EXIT");
    std::process::exit(0); */

    // DELETE DATA
    //
    delete_data_related_to_cfs_configuration::delete(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        &cfs_configuration_name_vec,
        &image_id_vec,
        // &cfs_components,
        &cfs_session_cfs_configuration_image_id_tuple_filtered_vec
            .into_iter()
            .map(|(session, _, _)| session)
            .collect(),
        &bos_sessiontemplate_cfs_configuration_image_id_tuple_filtered_vec
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
    cfs_configuration_name_vec: &Vec<&str>,
    image_id_vec: &Vec<&str>,
    cfs_session_name_vec: &Vec<&str>,
    bos_sessiontemplate_name_vec: &Vec<&str>,
) {
    // DELETE DATA
    //
    // DELETE IMAGES
    for image_id in image_id_vec {
        let image_deleted_value_rslt = mesa::ims::image::http_client::delete(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            image_id,
        )
        .await;

        // process api response
        match image_deleted_value_rslt {
            Ok(_) => println!("Image deleted: {}", image_id),
            Err(error) => {
                eprintln!("ERROR:\n{:#?}", error);
                let error_response = serde_json::from_str::<Value>(&error.to_string()).unwrap();
                eprintln!("ERROR:\n{:#?}", error_response);
                // std::process::exit(0);
                if error_response["status"].as_u64().unwrap() == 404 {
                    eprintln!("Image {} not found. Continue", image_id);
                }
            }
        }
    }

    // DELETE BOS SESSIONS
    let bos_session_id_value_vec = mesa::bos::session::shasta::http_client::get(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        None,
    )
    .await
    .unwrap();

    // Match BOS SESSIONS with the BOS SESSIONTEMPLATE RELATED
    for bos_session_id_value in bos_session_id_value_vec {
        let bos_session_value = mesa::bos::session::shasta::http_client::get(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            Some(bos_session_id_value.as_str().unwrap()),
        )
        .await
        .unwrap();

        if !bos_session_value.is_empty()
            && bos_sessiontemplate_name_vec.contains(
                &bos_session_value.first().unwrap()["templateName"]
                    .as_str()
                    .unwrap(),
            )
        {
            mesa::bos::session::shasta::http_client::delete(
                shasta_token,
                shasta_base_url,
                shasta_root_cert,
                bos_session_id_value.as_str().unwrap(),
            )
            .await
            .unwrap();

            println!(
                "BOS session deleted: {}",
                bos_session_id_value.as_str().unwrap() // For some reason CSM API to delete a BOS
                                                       // session does not returns the BOS session
                                                       // ID in the payload...
            );
        } else {
            log::info!(
                "Could not find BOS session template related to BOS session {} - Possibly related to a different HSM group or BOS session template was deleted?",
                bos_session_id_value.as_str().unwrap()
            );
        }
    }

    // DELETE CFS SESSIONS
    let max_attempts = 5;
    for cfs_session_name in cfs_session_name_vec {
        let mut counter = 0;
        loop {
            let deletion_rslt = mesa::cfs::session::shasta::http_client::delete(
                shasta_token,
                shasta_base_url,
                shasta_root_cert,
                cfs_session_name,
            )
            .await;

            /* println!(
                "CFS session deleted: {}",
                cfs_session_value["name"].as_str().unwrap()
            ); */
            if deletion_rslt.is_err() && counter <= max_attempts {
                log::warn!("Could not delete CFS session {} attempt {} of {}, trying again in 2 seconds...", cfs_session_name, counter, max_attempts);
                thread::sleep(time::Duration::from_secs(2));
                counter += 1;
            } else if deletion_rslt.is_err() && counter > max_attempts {
                eprint!(
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
        let mut counter = 0;
        loop {
            let deletion_rslt = mesa::bos::template::shasta::http_client::delete(
                shasta_token,
                shasta_base_url,
                shasta_root_cert,
                bos_sessiontemplate_name,
            )
            .await;

            /* match deletion_rslt {
                Ok(_) => println!(
                    "BOS sessiontemplate deleted: {}",
                    bos_sessiontemplate["name"].as_str().unwrap()
                ),
                Err(error) => {
                    let response_error = serde_json::from_str::<Value>(&error.to_string());
                    log::error!("{:#?}", response_error);
                }
            } */

            if deletion_rslt.is_err() && counter <= max_attempts {
                log::warn!("Could not delete BOS sessiontemplate {} attempt {} of {}, trying again in 2 seconds...", bos_sessiontemplate_name, counter, max_attempts);
                thread::sleep(time::Duration::from_secs(2));
                counter += 1;
            } else if deletion_rslt.is_err() && counter > max_attempts {
                eprint!(
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
        let mut counter = 0;
        loop {
            let deletion_rslt = cfs::configuration::shasta::http_client::delete(
                shasta_token,
                shasta_base_url,
                shasta_root_cert,
                cfs_configuration,
            )
            .await;

            if deletion_rslt.is_err() && counter <= max_attempts {
                log::warn!("Could not delete CFS configuration {} attempt {} of {}, trying again in 2 seconds...", cfs_configuration, counter, max_attempts);
                thread::sleep(time::Duration::from_secs(2));
                counter += 1;
            } else if deletion_rslt.is_err() && counter > max_attempts {
                eprint!(
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
