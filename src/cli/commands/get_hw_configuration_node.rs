use comfy_table::{Cell, Table};
use mesa::hsm::hw_components::NodeSummary;
use std::string::ToString;

use termion::color;

/// Get nodes status/configuration for some nodes filtered by a HSM group.
pub async fn exec(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    hsm_group_name: Option<&String>,
    xname: &str,
    type_artifact_opt: Option<&String>,
    output_opt: Option<&String>,
) {
    let hsm_groups_resp = mesa::hsm::group::shasta::http_client::get_hsm_group_vec(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        hsm_group_name,
    )
    .await;

    let hsm_group_list = if let Ok(hsm_groups) = hsm_groups_resp {
        hsm_groups
    } else {
        eprintln!(
            "No HSM group {}{}{} found!",
            color::Fg(color::Red),
            hsm_group_name.unwrap(),
            color::Fg(color::Reset)
        );
        std::process::exit(0);
    };

    if hsm_group_list.is_empty() {
        println!("No HSM group found");
        std::process::exit(0);
    }

    // Take all nodes for all hsm_groups found and put them in a Vec
    let mut hsm_groups_node_list: Vec<String> =
        mesa::hsm::group::shasta::utils::get_member_vec_from_hsm_group_value_vec(&hsm_group_list)
            .into_iter()
            .collect();

    hsm_groups_node_list.sort();

    let mut node_hw_inventory = &mesa::hsm::hw_inventory::shasta::http_client::get_hw_inventory(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        xname,
    )
    .await
    .unwrap();

    node_hw_inventory = node_hw_inventory.pointer("/Nodes/0").unwrap();

    if let Some(type_artifact) = type_artifact_opt {
        node_hw_inventory = &node_hw_inventory
            .as_array()
            .unwrap()
            .iter()
            .find(|&node| node["ID"].as_str().unwrap().eq(xname))
            .unwrap()[type_artifact];
    }

    let node_summary = NodeSummary::from_csm_value(node_hw_inventory.clone());

    if output_opt.is_some() && output_opt.unwrap().eq("json") {
        println!("{}", serde_json::to_string_pretty(&node_summary).unwrap());
    } else {
        print_table(&[node_summary].to_vec());
    }
}

pub fn print_table(node_summary_vec: &Vec<NodeSummary>) {
    let mut table = Table::new();

    table.set_header(vec![
        "Node XName",
        "Component XName",
        "Component Type",
        "Component Info",
    ]);

    for node_summary in node_summary_vec {
        for processor in &node_summary.processors {
            table.add_row(vec![
                Cell::new(node_summary.xname.clone()),
                Cell::new(processor.xname.clone()),
                Cell::new(processor.r#type.clone()),
                Cell::new(
                    processor
                        .info
                        .clone()
                        .unwrap_or("*** Missing info".to_string()),
                ),
            ]);
        }

        for memory in &node_summary.memory {
            table.add_row(vec![
                Cell::new(node_summary.xname.clone()),
                Cell::new(memory.xname.clone()),
                Cell::new(memory.r#type.clone()),
                Cell::new(
                    memory
                        .info
                        .clone()
                        .unwrap_or("*** Missing info".to_string()),
                ),
            ]);
        }

        for node_accel in &node_summary.node_accels {
            table.add_row(vec![
                Cell::new(node_summary.xname.clone()),
                Cell::new(node_accel.xname.clone()),
                Cell::new(node_accel.r#type.clone()),
                Cell::new(
                    node_accel
                        .clone()
                        .info
                        .unwrap_or("*** Missing info".to_string()),
                ),
            ]);
        }

        for node_hsn_nic in &node_summary.node_hsn_nics {
            table.add_row(vec![
                Cell::new(node_summary.xname.clone()),
                Cell::new(node_hsn_nic.xname.clone()),
                Cell::new(node_hsn_nic.r#type.clone()),
                Cell::new(
                    node_hsn_nic
                        .clone()
                        .info
                        .unwrap_or("*** Missing info".to_string()),
                ),
            ]);
        }
    }

    println!("{table}");
}
