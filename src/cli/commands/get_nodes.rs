use clap::ArgMatches;
use termion::color;

use crate::{common::node_ops, shasta::{hsm, self}};

pub async fn exec(
    hsm_group: Option<&String>,
    cli_get_node: &ArgMatches,
    shasta_token: &str,
    shasta_base_url: &str,
) {
    // Check HSM group name provided y configuration file
    let hsm_group_name = match hsm_group {
        None => cli_get_node.get_one::<String>("HSMGROUP"),
        Some(_) => hsm_group,
    };

    let hsm_group_resp =
        hsm::http_client::get_hsm_group(shasta_token, shasta_base_url, hsm_group_name.unwrap())
            .await;

    // println!("hsm_groups: {:?}", hsm_groups);

    let hsm_group = if hsm_group_resp.is_err() {
        eprintln!(
            "No HSM group {}{}{} found!",
            color::Fg(color::Red),
            hsm_group_name.unwrap(),
            color::Fg(color::Reset)
        );
        std::process::exit(0);
    } else {
        hsm_group_resp.unwrap()
    };

    // Take all nodes for all hsm_groups found and put them in a Vec
    let hsm_groups_nodes: Vec<String> = hsm::utils::get_members_ids_from_serde_value(&hsm_group);

    // Get all BOS session templates for HSM group
    let mut bos_sessiontemplates = shasta::bos::template::http_client::get(
        shasta_token,
        shasta_base_url,
        hsm_group_name.map(|x| &**x),
        None,
        None,
    )
    .await
    .unwrap();

    bos_sessiontemplates.reverse();

    let compute_hsm_group_bos_sessiontemplate = bos_sessiontemplates
        .iter()
        .find(|&bos_sessiontemplate| bos_sessiontemplate.pointer("/boot_sets/compute").is_some());

    let uan_hsm_group_bos_sessiontemplate = bos_sessiontemplates
        .iter()
        .find(|&bos_sessiontemplate| bos_sessiontemplate.pointer("/boot_sets/uan").is_some());

    // Get nodes details (nids) from HSM
    let nodes_hsm_info_resp = hsm::http_client::get_components_status(
        shasta_token,
        shasta_base_url,
        hsm_groups_nodes.to_vec(),
    )
    .await
    .unwrap();

    // match node with bot_sessiontemplate and put them in a list
    let mut nodes_details = Vec::new();

    for node in &hsm_groups_nodes {
        let mut image_id = None;
        let mut cfs_configuration_name = None;
        let mut combo = Vec::new();
        // Get power status
        // node_power_status = get_node_power_status(node, &nodes_power_status_resp);
        let node_details = nodes_hsm_info_resp["Components"]
            .as_array()
            .unwrap()
            .iter()
            .find(|&component| component["ID"].as_str().unwrap().eq(node))
            .unwrap();

        let node_power_status = node_details["State"]
            .as_str()
            .unwrap()
            .to_string()
            .to_uppercase();

        let node_nid = format!(
            "nid{:0>6}",
            node_details["NID"].as_u64().unwrap().to_string()
        );

        combo.push(node.to_string());
        combo.push(node_nid);
        combo.push(node_power_status);
        // Set image and CFS configuration
        for bos_sessiontemplate in &bos_sessiontemplates {
            // find bos_sesstiontemplate in /boot_sets/compute/node_list
            if bos_sessiontemplate
                .pointer("/boot_sets/compute/node_list")
                .unwrap_or(&serde_json::Value::Array(Vec::new()))
                .as_array()
                .unwrap()
                .iter()
                .any(|node_value| node_value.as_str().unwrap().eq(node))
            {
                image_id = Some(
                    bos_sessiontemplate
                        .pointer("/boot_sets/compute/path")
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
                combo.push(image_id.to_owned().unwrap());
                combo.push(cfs_configuration_name.to_owned().unwrap());

                nodes_details.push(combo.to_owned());
                break;
            // find bos_sessiontemplate in /boot_sets/uan/node_list
            } else if bos_sessiontemplate
                .pointer("/boot_sets/uan/node_list")
                .unwrap_or(&serde_json::Value::Array(Vec::new()))
                .as_array()
                .unwrap()
                .iter()
                .any(|node_value| node_value.as_str().unwrap().eq(node))
            {
                image_id = Some(
                    bos_sessiontemplate
                        .pointer("/boot_sets/compute/path")
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
                combo.push(image_id.to_owned().unwrap());
                combo.push(cfs_configuration_name.to_owned().unwrap());

                nodes_details.push(combo.to_owned());
                break;
            }
        }
        // No bos_sessointemplate found in node_list param, using the most recent
        // bos_sessiontemplate for node_groups
        if image_id.is_none() && cfs_configuration_name.is_none() {
            combo.push(
                compute_hsm_group_bos_sessiontemplate
                    .unwrap()
                    .pointer("/boot_sets/compute/path")
                    .unwrap()
                    .as_str()
                    .unwrap()
                    .to_string()
                    .trim_start_matches("s3://boot-images/")
                    .trim_end_matches("/manifest.json")
                    .to_string()
                    .to_owned(),
            );
            combo.push(
                bos_sessiontemplates
                    .first()
                    .unwrap()
                    .pointer("/cfs/configuration")
                    .unwrap()
                    .as_str()
                    .unwrap()
                    .to_string(),
            );
            nodes_details.push(combo);
        }
    }

    node_ops::print_table(nodes_details);
}
