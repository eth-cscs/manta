use comfy_table::{Cell, Table};
use mesa::{bss::bootparameters::BootParameters, node::r#struct::NodeDetails};
use serde_json::Value;

pub fn print_table(nodes_status: Vec<NodeDetails>) {
    let mut table = Table::new();

    table.set_header(vec![
        "XNAME",
        "NID",
        "Power Status",
        "Runtime Configuration",
        "Configuration Status",
        "Enabled",
        "Error Count",
        // "Tags",
        "Image Configuration",
        "Image ID",
    ]);

    for node_status in nodes_status {
        table.add_row(vec![
            Cell::new(node_status.xname),
            Cell::new(node_status.nid),
            Cell::new(node_status.power_status),
            Cell::new(node_status.desired_configuration),
            Cell::new(node_status.configuration_status),
            Cell::new(node_status.enabled),
            Cell::new(node_status.error_count),
            Cell::new(node_status.boot_configuration),
            Cell::new(node_status.boot_image_id),
        ]);
    }

    println!("{table}");
}

pub fn nodes_to_string_format_one_line(nodes: Option<&Vec<Value>>) -> String {
    if let Some(nodes_content) = nodes {
        nodes_to_string_format_discrete_columns(nodes, nodes_content.len() + 1)
    } else {
        "".to_string()
    }
}

pub fn nodes_to_string_format_discrete_columns(
    nodes: Option<&Vec<Value>>,
    num_columns: usize,
) -> String {
    let mut members: String;

    match nodes {
        Some(nodes) if !nodes.is_empty() => {
            members = nodes[0].as_str().unwrap().to_string(); // take first element

            for (i, _) in nodes.iter().enumerate().skip(1) {
                // iterate for the rest of the list
                if i % num_columns == 0 {
                    // breaking the cell content into multiple lines (only 2 xnames per line)

                    members.push_str(",\n");
                } else {
                    members.push(',');
                }

                members.push_str(nodes[i].as_str().unwrap());
            }
        }
        _ => members = "".to_string(),
    }

    members
}

/// Given a list of boot params, this function returns the list of hosts booting an image_id
pub fn get_node_vec_booting_image(
    image_id: &str,
    boot_param_vec: &[BootParameters],
) -> Vec<String> {
    let mut node_booting_image_vec = boot_param_vec
        .iter()
        .cloned()
        .filter(|boot_param| boot_param.get_boot_image().eq(image_id))
        .flat_map(|boot_param| boot_param.hosts)
        .collect::<Vec<_>>();

    node_booting_image_vec.sort();

    node_booting_image_vec
}
