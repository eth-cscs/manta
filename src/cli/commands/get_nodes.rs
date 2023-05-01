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
    silent_xname: bool,
) {
    // TODO:
    // How to get CFS session linked to node boot params and CFS session linked to BOA
    // 1) Get all nodes/xnames involved
    // 2) Get all nodes boot params
    // 3) Get all CFS sessions which ended successfully
    // 4) Get all BOS sessiontemplate
    // 4) Loop the nodes
    // 4.1) for each node get its boot params --> boot image
    // 4.2) find the CFS session which generated the image found in 4.1
    // 4.3) get the CFS configuration from the CFS session found in 4.2
    // 4.4) get BOS sessiontemplate for the node
    // 4.5) get all CFS configurations and match them with the BOS sessiontemplate in chronological
    //   order by CFS configuraton creation date
    //
    /* // Check HSM group name provided and configuration file
    let hsm_group_name = match hsm_group {
        None => cli_get_node.get_one::<String>("HSMGROUP"),
        Some(_) => hsm_group,
    }; */

    let hsm_groups_resp =
        hsm::http_client::get_hsm_groups(shasta_token, shasta_base_url, hsm_group_name).await;

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

    // println!("hsm_group_list:\n{:#?}", hsm_group_list);

    // Take all nodes for all hsm_groups found and put them in a Vec
    let mut hsm_groups_node_list: Vec<String> =
        hsm::utils::get_members_from_hsm_groups_serde_value(&hsm_group_list)
            .into_iter()
            .collect();

    hsm_groups_node_list.sort();

    let hsm_group_nodes_string = hsm_groups_node_list.join(",");

    let components_status = shasta::cfs::component::http_client::get_multiple_components(
        shasta_token,
        shasta_base_url,
        Some(&hsm_group_nodes_string),
        None,
        // None,
        /* None,
        None,
        None, */
    )
    .await
    .unwrap();

    // println!("components_status:\n{:#?}", components_status);

    /* let node_hsm_groups_map =
        hsm::utils::group_members_by_hsm_group_from_hsm_groups_serde_value(&hsm_group_list); */

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
    /* let bos_sessiontemplate_list = shasta::bos::template::http_client::get(
        shasta_token,
        shasta_base_url,
        hsm_group_name,
        None,
        None,
    )
    .await
    .unwrap(); */

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

    // match node with bot_sessiontemplate and put them in a list
    let mut node_details_list = Vec::new();

    for node in &hsm_groups_node_list {
        // let mut kernel_image_path_in_boot_params = None;
        // let mut manifest_image_path_in_bos_sessiontemplate = None;
        // let mut cfs_configuration_name = None;
        let mut node_details = Vec::new();

        // Find component details
        let component_details = components_status
            .iter()
            .find(|component_status| component_status["id"].as_str().unwrap().eq(node))
            .unwrap();

        let desired_configuration = component_details["desiredConfig"]
            .as_str()
            .unwrap_or_default();
        let configuration_status = component_details["configurationStatus"]
            .as_str()
            .unwrap_or_default();
        let enabled = component_details["enabled"].as_bool().unwrap_or_default();
        let error_count = component_details["errorCount"].as_i64().unwrap_or_default();
        // let tags = component_details["tags"].to_string();

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
        node_details.push(desired_configuration.to_string());
        node_details.push(configuration_status.to_string());
        node_details.push(enabled.to_string());
        node_details.push(error_count.to_string());
        // node_details.push(tags);

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

        // println!("node_boot_params:\n{:#?}", node_boot_params);

        let kernel_image_path_in_boot_params = Some(
            node_boot_params.unwrap()["kernel"]
                .as_str()
                .unwrap()
                .to_string()
                .trim_start_matches("s3://boot-images/")
                .trim_end_matches("/kernel")
                .to_string()
                .to_owned(),
        )
        .unwrap_or_default();

        node_details.push(kernel_image_path_in_boot_params);

        node_details_list.push(node_details.to_owned());

        /* // Loop CFS configuration list
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

            // because acording to CSM docs https://cray-hpe.github.io/docs-csm/en-12/operations/boot_orchestration/session_templates/ boot_set[].property can be
            // anything.
            for (boot_set_property, boot_set_value) in
                bos_sessiontemplate["boot_sets"].as_object().unwrap()
            {
                /* println!("boot_set_property {:#?}", boot_set_property);
                println!("boot_set_value {:#?}", boot_set_value); */

                if boot_set_value["node_list"]
                    .as_array()
                    .unwrap_or(&Vec::new())
                    .iter()
                    .any(|node_value| node_value.as_str().unwrap().eq(node))
                    || boot_set_value["node_groups"]
                        .as_array()
                        .unwrap_or(&Vec::new())
                        .iter()
                        .any(|hsm_value| {
                            node_hsm_groups_map
                                .get(node)
                                .unwrap()
                                .contains(&hsm_value.as_str().unwrap().to_string())
                        })
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
                        boot_set_value["path"]
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
        } */
    }

    if silent {
        println!(
            "{}",
            node_details_list
                .iter()
                .map(|node_details| node_details[1].clone())
                .collect::<Vec<String>>()
                .join(",")
        );
    } else if silent_xname {
        println!(
            "{}",
            node_details_list
                .iter()
                .map(|node_details| node_details[0].clone())
                .collect::<Vec<String>>()
                .join(",")
        );
    } else {
        // println!("node_details_list:\n{:#?}", node_details_list);

        node_ops::print_table(node_details_list);
    }
}
