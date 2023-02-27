use clap::ArgMatches;

use std::collections::HashMap;

use termion::color;

use crate::shasta::{bss, capmc, cfs, hsm};

use crate::common::node_ops;

pub async fn exec(
    hsm_group: Option<&String>,
    cli_get_node: &ArgMatches,
    shasta_token: &String,
    shasta_base_url: &String,
) -> () {
    // Check if HSM group name provided y configuration file
    let hsm_group_name = match hsm_group {
        None => cli_get_node.get_one::<String>("HSMGROUP"),
        Some(_) => hsm_group,
    };

    let hsm_group_resp =
        hsm::http_client::get_hsm_group(&shasta_token, &shasta_base_url, hsm_group_name.unwrap())
            .await;

    // println!("hsm_groups: {:?}", hsm_groups);

    let hsm_group;

    // Exit if no hsm groups found
    if hsm_group_resp.is_err() {
        eprintln!(
            "No HSM group {}{}{} found!",
            color::Fg(color::Red),
            hsm_group_name.unwrap(),
            color::Fg(color::Reset)
        );
        std::process::exit(0);
    } else {
        hsm_group = hsm_group_resp.unwrap();
    }

    // Take all nodes for all hsm_groups found and put them in a Vec
    let hsm_groups_nodes: Vec<String> = hsm::utils::get_member_ids(&hsm_group);
    //            let hsm_groups_nodes: Vec<String> = hsm_group["members"]["ids"]
    //                .as_array()
    //                .unwrap_or(&Vec::new())
    //                .iter()
    //                .map(|xname| xname.as_str().unwrap().to_string())
    //                .collect();

    // Get node most recent CFS session with target image
    // Get all CFS sessions matching hsm_group value
    let mut cfs_sessions = cfs::session::http_client::get(
        &shasta_token,
        &shasta_base_url,
        hsm_group_name,
        None,
        None,
        Some(true),
    )
    .await
    .unwrap();

    // Sort CFS sessions by start time
    cfs_sessions.sort_by(|a, b| {
        a["status"]["session"]["startTime"] // don't be tempted to use completionTime
            // because this field is NULL is CFS session
            // failed. Plus lastest CFS session should be
            // considered the latest to start since is the
            // last one ran by the user (with latest
            // changes)
            .as_str()
            .unwrap()
            .cmp(b["status"]["session"]["startTime"].as_str().unwrap())
    });

    // println!("cfs_sessions: {:#?}", cfs_sessions);

    // Filter CFS sessions with target = "image" and succeded = "true"
    let cfs_sessions_target_image: Vec<_> = cfs_sessions
        .iter()
        .filter(|cfs_session| {
            cfs_session["target"]["definition"]
                .as_str()
                .unwrap()
                .eq("image")
                && cfs_session["status"]["session"]["succeeded"]
                    .as_str()
                    .unwrap()
                    .eq("true")
        })
        .collect();

    // println!("cfs_sessions_target_image: {:#?}", cfs_sessions_target_image);

    // Get most recent CFS session with target = "image" and succeded = "true"
    let most_recent_cfs_session = cfs_sessions_target_image
        [cfs_sessions_target_image.len().saturating_sub(1)..]
        .to_vec()
        .iter()
        .next()
        .unwrap();

    // Get CFS configurations for HSM group
    let most_recent_cfs_configurations = crate::shasta::cfs::configuration::http_client::get(
        shasta_token,
        shasta_base_url,
        hsm_group_name,
        None,
        Some(&1),
    )
    .await
    .unwrap();

    // Get BOS session templates for HSM group
    let most_recent_bos_sessiontemplates = crate::shasta::bos::template::http_client::get(
        shasta_token,
        shasta_base_url,
        hsm_group_name,
        None,
        Some(&1),
    )
    .await
    .unwrap();

    let image_id;

    // Check whether most recent CFS configuration is used in the most recent BOS sessiontemplate
    // If False ==> No way image in node will use the most recent CFS configuration
    // if True ==> check boot params on each node and see if image_id matches on both most recent
    // BOS sessiontemplate and node boot params
    if most_recent_bos_sessiontemplates.iter().next().unwrap()["cfs"]["configuration"]
        .as_str()
        .unwrap()
        != most_recent_cfs_configurations.iter().next().unwrap()["name"]
            .as_str()
            .unwrap()
    {
        // Most recent BOS sessiontemplate does not use most recelt CFS configuration ==> image_id
        // = None
        image_id = None;
    } else {
        // Most recent BOS sessiontemplate uses most recent CFS configuration ==>
        // Extract image id from session
        image_id = Some(
            most_recent_bos_sessiontemplates.iter().next().unwrap()["boot_sets"]["compute"]["path"]
                .as_str()
                .unwrap()
                .to_string()
                .trim_start_matches("s3://boot-images/")
                .trim_end_matches("/manifest.json")
                .to_string(),
        );

        log::info!(
            "Image_id from most recent CFS session (target = image and successful = true): {}",
            image_id.to_owned().unwrap()
        );
    }

    // Correlate last/most recent CFS session with CFS configuration with BOS sessiontemplate --> This
    // will correlate the image id related to the CFS most recent

    // Correlate last/most recent BOS sessontemplate with CFS configuration --> This will indicate
    // whether the most recent image was created according to the most recent CFS configuration

    // TODO: the image_id from the most recent CFS session is no longer representative of the image
    // related to that configuration/session, this is because the script we use which uses SAT which overwrites the image after creation `sudo sat bootprep run -s --overwrite-configs --overwrite-images --overwrite-templates ${file}`, this new image has a different ID which no longer matches the result_id from the session. This means we need to list all images and get the one with name <HSM GROUP>-cos-template-<DATE> with date being the most recent one...????

    // Get nodes details
    let nodes_status =
        get_nodes_details(&shasta_token, &shasta_base_url, image_id, &hsm_groups_nodes).await;

    // shasta::hsm::utils::print_table(hsm_groups);
    node_ops::print_table(nodes_status);
}

pub async fn get_nodes_details(
    shasta_token: &String,
    shasta_base_url: &String,
    image_id: Option<String>,
    xnames: &Vec<String>,
) -> Vec<Vec<String>> {
    let mut nodes_status: Vec<Vec<String>> = Vec::new();

    // Get power node status from capmc
    let nodes_power_status_resp =
        capmc::http_client::node_power_status::post(&shasta_token, &shasta_base_url, xnames)
            .await
            .unwrap();

    // Get nodes boot params
    let nodes_boot_params =
        bss::http_client::get_boot_params(shasta_token, shasta_base_url, xnames)
            .await
            .unwrap();

    // println!("nodes_boot_params: {:#?}", nodes_boot_params);

    let mut nodes_images: HashMap<String, String> = HashMap::new();

    // Create dictionary of xname and image_id
    for node_boot_params in nodes_boot_params {
        let nodes: Vec<String> = node_boot_params["hosts"]
            .as_array()
            .unwrap()
            .iter()
            .map(|node| node.as_str().unwrap().to_string())
            .collect();

        let image_id = node_boot_params["kernel"]
            .as_str()
            .unwrap()
            .to_string()
            .trim_start_matches("s3://boot-images/")
            .trim_end_matches("/kernel")
            .to_string();

        for node in nodes {
            nodes_images.insert(node, image_id.clone());
        }
    }

    //    println!("node_images dictionary: {:#?}", nodes_images);

    // Group nodes by power status
    let nodes_on: Vec<String> = nodes_power_status_resp["on"]
        .as_array()
        .unwrap_or(&Vec::new())
        .iter()
        .map(|xname| xname.as_str().unwrap().to_string())
        .collect();
    let nodes_off: Vec<String> = nodes_power_status_resp["off"]
        .as_array()
        .unwrap_or(&Vec::new())
        .iter()
        .map(|xname| xname.as_str().unwrap().to_string())
        .collect();
    let nodes_disabled: Vec<String> = nodes_power_status_resp["disabled"]
        .as_array()
        .unwrap_or(&Vec::new())
        .iter()
        .map(|xname| xname.as_str().unwrap().to_string())
        .collect();
    let nodes_ready: Vec<String> = nodes_power_status_resp["ready"]
        .as_array()
        .unwrap_or(&Vec::new())
        .iter()
        .map(|xname| xname.as_str().unwrap().to_string())
        .collect();
    let nodes_standby: Vec<String> = nodes_power_status_resp["standby"]
        .as_array()
        .unwrap_or(&Vec::new())
        .iter()
        .map(|xname| xname.as_str().unwrap().to_string())
        .collect();

    //    println!(
    //        "List nodes power status:\nON: {:?}\nOFF: {:?}\nDISABLED: {:?}\nREADY: {:?}\nSTANDBY: {:?}",
    //        nodes_on, nodes_off, nodes_disabled, nodes_ready, nodes_standby
    //    );

    // Merge nodes power status with boot params
    for xname in xnames {
        let mut node_status: Vec<String> = vec![xname.to_string()];

        // Get node power status
        let node_power_status = if nodes_on.contains(xname) {
            "ON".to_string()
        } else if nodes_off.contains(xname) {
            "OFF".to_string()
        } else if nodes_disabled.contains(xname) {
            "DISABLED".to_string()
        } else if nodes_ready.contains(xname) {
            "READY".to_string()
        } else if nodes_standby.contains(xname) {
            "STANDBY".to_string()
        } else {
            "N/A".to_string()
        };

        node_status.push(node_power_status);

        // Get node boot param image
        let node_image_id = nodes_images.get(&xname.to_string()).unwrap().to_string();

        node_status.push(node_image_id.clone());

        // Has node latest image?
        if image_id.is_some() {
            node_status.push(node_image_id.eq(&image_id.clone().unwrap()).to_string());
        } else {
            node_status.push("N/A".to_string());
        }

        nodes_status.push(node_status);
    }

    // println!("nodes_status: {:#?}", nodes_status);

    nodes_status
}
