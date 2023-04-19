use termion::color;

use crate::{
    common::node_ops,
    shasta::{self, hsm},
};

/// Get nodes status/configuration for some nodes filtered by a HSM group.
///
pub async fn exec(
    // hsm_group: Option<&String>,
    // cli_get_node: &ArgMatches,
    shasta_token: &str,
    shasta_base_url: &str,
    hsm_group_name: Option<&String>,
    silent: bool,
) {
    /* // Check HSM group name provided and configuration file
    let hsm_group_name = match hsm_group {
        None => cli_get_node.get_one::<String>("HSMGROUP"),
        Some(_) => hsm_group,
    }; */

    let hsm_groups_resp = hsm::http_client::get_hsm_groups(
        shasta_token,
        shasta_base_url,
        Some(hsm_group_name.unwrap()),
    )
    .await;

    // println!("hsm_groups_resp: {:#?}", hsm_groups_resp);

    let hsm_group_list = if hsm_groups_resp.is_err() {
        eprintln!(
            "No HSM group {}{}{} found!",
            color::Fg(color::Red),
            hsm_group_name.unwrap(),
            color::Fg(color::Reset)
        );
        std::process::exit(0);
    } else {
        hsm_groups_resp.unwrap()
    };

    // Take all nodes for all hsm_groups found and put them in a Vec
    let mut hsm_groups_node_list: Vec<String> =
        hsm::utils::get_members_from_hsm_groups_serde_value(&hsm_group_list)
            .into_iter()
            .collect();

    hsm_groups_node_list.sort();

    // println!("hsm_groups_nodes: {:?}", hsm_groups_nodes);

    let node_hsm_groups_map =
        hsm::utils::get_members_and_hsm_from_hsm_groups_serde_value(&hsm_group_list);

    // println!("node_hsm_groups_map:\n{:#?}", node_hsm_groups_map);

    // Get boot params
    let nodes_boot_params_list = shasta::bss::http_client::get_boot_params(
        shasta_token,
        shasta_base_url,
        &hsm_groups_node_list,
    )
    .await
    .unwrap();

    // Get all BOS session templates for HSM group
    let bos_sessiontemplate_list = shasta::bos::template::http_client::get(
        shasta_token,
        shasta_base_url,
        hsm_group_name,
        None,
        None,
    )
    .await
    .unwrap();

    // Get all CFS configurations so we can link CFS configuration name with its counterpart in the
    // BOS sessiontemplate, we are doing this because BOS sessiontemplate does not have
    // creation/update time hence I can't sort by date to loop and find out most recent BOS
    // sessiontemplate per node. Joining CFS configuration and BOS sessiontemplate will help to
    // this
    let mut cfs_configuration_list = shasta::cfs::configuration::http_client::get(
        shasta_token,
        shasta_base_url,
        None,
        None,
        None,
    )
    .await
    .unwrap();

    // reverse list in order to have most recent CFS configuration lastUpdate values at front
    cfs_configuration_list.reverse();

    // println!("bos_sessiontemplate_list:\n{:#?}", bos_sessiontemplate_list);

    // Get nodes details (nids) from HSM
    let nodes_hsm_info_resp = hsm::http_client::get_components_status(
        shasta_token,
        shasta_base_url,
        hsm_groups_node_list.clone(),
    )
    .await
    .unwrap();

    // match node with bot_sessiontemplate and put them in a list
    let mut node_details_list = Vec::new();

    for node in &hsm_groups_node_list {
        let mut kernel_image_path_in_boot_params = None;
        let mut manifest_image_path_in_bos_sessiontemplate = None;
        let mut cfs_configuration_name = None;
        let mut node_details = Vec::new();
        // Get power status
        // node_power_status = get_node_power_status(node, &nodes_power_status_resp);
        let node_hsm_info = nodes_hsm_info_resp["Components"]
            .as_array()
            .unwrap()
            .iter()
            .find(|&component| component["ID"].as_str().unwrap().eq(node))
            .unwrap();

        let node_power_status = node_hsm_info["State"]
            .as_str()
            .unwrap()
            .to_string()
            .to_uppercase();

        let node_nid = format!(
            "nid{:0>6}",
            node_hsm_info["NID"].as_u64().unwrap().to_string()
        );

        node_details.push(node.to_string());
        node_details.push(node_nid);
        node_details.push(node_power_status);

        // Get node boot params (these are the boot params of the nodes with the image the node
        // boot with). The image in the BOS sessiontemplate may be different I don't know why. Need
        // to investigate
        let node_boot_params = nodes_boot_params_list.iter().find(|&node_boot_param| {
            node_boot_param["hosts"]
                .as_array()
                .unwrap()
                .iter()
                .map(|host_value| host_value.as_str().unwrap())
                .any(|host| host.eq(node))
        });

        // Loop CFS configuration list
        for cfs_configuration in &cfs_configuration_list {
            /* println!(
                "Processing CFS configuration {} with last update date {}",
                cfs_configuration["name"].as_str().unwrap(),
                cfs_configuration["lastUpdated"]
            ); */
            let bos_sessiontemplate_option =
                bos_sessiontemplate_list.iter().find(|bos_sessiontemplate| {
                    bos_sessiontemplate
                        .pointer("/cfs/configuration")
                        .unwrap()
                        .as_str()
                        .unwrap()
                        .eq(cfs_configuration["name"].as_str().unwrap())
                });

            if bos_sessiontemplate_option.is_none() {
                continue;
            }

            let bos_sessiontemplate = bos_sessiontemplate_option.unwrap();

            // for bos_sessiontemplate in &bos_sessiontemplate_list {
            for boot_set_property in ["uan", "compute"] {
                /* println!(
                    "Comparing node {} in hsm groups {:?} with bos_sessointemplate {:#?}",
                    node,
                    node_hsm_groups_map.get(node),
                    bos_sessiontemplate
                ); */
                if bos_sessiontemplate
                    .pointer(&("/boot_sets/".to_owned() + boot_set_property + "/node_list"))
                    .unwrap_or(&serde_json::Value::Array(Vec::new()))
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|node_value| node_value.as_str().unwrap().eq(node))
                    || bos_sessiontemplate
                        .pointer(&("/boot_sets/".to_owned() + boot_set_property + "/node_groups"))
                        .unwrap_or(&serde_json::Value::Array(Vec::new()))
                        .as_array()
                        .unwrap()
                        .iter()
                        .any(|hsm_value| {
                            node_hsm_groups_map
                                .get(node)
                                .unwrap()
                                .contains(&hsm_value.as_str().unwrap().to_string())
                        })
                // Check /boot_sets/<property>/node_groups contains any hsm group linked to "node"
                {
                    kernel_image_path_in_boot_params = Some(
                        node_boot_params.unwrap()["kernel"]
                            .as_str()
                            .unwrap()
                            .to_string()
                            .trim_start_matches("s3://boot-images/")
                            .trim_end_matches("/kernel")
                            .to_string()
                            .to_owned(),
                    );

                    manifest_image_path_in_bos_sessiontemplate = Some(
                        bos_sessiontemplate
                            .pointer(&("/boot_sets/".to_owned() + boot_set_property + "/path"))
                            .unwrap()
                            .as_str()
                            .unwrap()
                            .to_string()
                            .trim_start_matches("s3://boot-images/")
                            .trim_end_matches("/manifest.json")
                            .to_string()
                            .to_owned(),
                    );
                    cfs_configuration_name = Some(
                        bos_sessiontemplate
                            .pointer("/cfs/configuration")
                            .unwrap()
                            .as_str()
                            .unwrap()
                            .to_string(),
                    );
                }
            }

            if manifest_image_path_in_bos_sessiontemplate.is_some()
                && cfs_configuration_name.is_some()
            {
                /* println!(
                    "\nnode:\n{}\ncfs_configuration:\n{:#?}\nbos_sessiontemplate\n{:#?}\n",
                    node, cfs_configuration, bos_sessiontemplate
                ); */
                node_details.push(kernel_image_path_in_boot_params.to_owned().unwrap());
                node_details.push(cfs_configuration_name.to_owned().unwrap());
                node_details.push(
                    manifest_image_path_in_bos_sessiontemplate
                        .to_owned()
                        .unwrap(),
                );

                node_details_list.push(node_details.to_owned());

                // println!("{:?}", combo);

                break;
            }
        }
    }

    if silent {
        println!("{}", node_details_list.iter().map(|node_details| node_details[1].clone()).collect::<Vec<String>>().join(","));
    } else {
        node_ops::print_table(node_details_list);
    }
}
