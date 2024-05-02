use comfy_table::{Cell, Table};
use mesa::hsm::hw_components::NodeSummary;
use std::string::ToString;

/// Get nodes status/configuration for some nodes filtered by a HSM group.
pub async fn exec(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    xname: &str,
    type_artifact_opt: Option<&String>,
    output_opt: Option<&String>,
) {
    let mut node_hw_inventory = &mesa::hsm::hw_inventory::shasta::http_client::get_hw_inventory(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        xname,
    )
    .await
    .unwrap();

    // node_hw_inventory = node_hw_inventory.pointer("/Nodes/0").unwrap();

    node_hw_inventory = match node_hw_inventory.pointer("/Nodes/0") {
        Some(node_value) => node_value,
        None => {
            eprintln!(
                "ERROR - json section '/Node' missing in json response API for node '{}'",
                xname
            );
            std::process::exit(1);
        }
    };

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
