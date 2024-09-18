use std::collections::HashMap;

use comfy_table::{Cell, Table};
use mesa::{bss::bootparameters::BootParameters, node::r#struct::NodeDetails};

pub fn print_table(nodes_status: Vec<NodeDetails>) {
    let mut table = Table::new();

    table.set_header(vec![
        "XNAME",
        "NID",
        "HSM",
        "Power Status",
        "Runtime Configuration",
        "Configuration Status",
        "Enabled",
        "Error Count",
        "Image Configuration",
        "Image ID",
    ]);

    for node_status in nodes_status {
        table.add_row(vec![
            Cell::new(node_status.xname),
            Cell::new(node_status.nid),
            Cell::new(node_status.hsm),
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

pub fn print_table_wide(nodes_status: Vec<NodeDetails>) {
    let mut table = Table::new();

    table.set_header(vec![
        "XNAME",
        "NID",
        "HSM",
        "Power Status",
        "Runtime Configuration",
        "Configuration Status",
        "Enabled",
        "Error Count",
        "Image Configuration",
        "Image ID",
        "Kernel Params",
    ]);

    for node_status in nodes_status {
        let kernel_params_vec: Vec<&str> = node_status.kernel_params.split_whitespace().collect();
        let cell_max_width = kernel_params_vec
            .iter()
            .map(|value| value.len())
            .max()
            .unwrap_or(0);

        let mut kernel_params_string: String = kernel_params_vec[0].to_string();
        let mut cell_width = kernel_params_string.len();

        for kernel_param in kernel_params_vec.iter().skip(1) {
            cell_width += kernel_param.len();

            if cell_width + kernel_param.len() >= cell_max_width {
                kernel_params_string.push_str("\n");
                cell_width = 0;
            } else {
                kernel_params_string.push_str(" ");
            }

            kernel_params_string.push_str(kernel_param);
        }

        table.add_row(vec![
            Cell::new(node_status.xname),
            Cell::new(node_status.nid),
            Cell::new(node_status.hsm),
            Cell::new(node_status.power_status),
            Cell::new(node_status.desired_configuration),
            Cell::new(node_status.configuration_status),
            Cell::new(node_status.enabled),
            Cell::new(node_status.error_count),
            Cell::new(node_status.boot_configuration),
            Cell::new(node_status.boot_image_id),
            Cell::new(kernel_params_string),
        ]);
    }

    println!("{table}");
}

pub fn print_summary(node_details_list: Vec<NodeDetails>) {
    let mut power_status_counters: HashMap<String, usize> = HashMap::new();
    let mut boot_configuration_counters: HashMap<String, usize> = HashMap::new();
    let mut runtime_configuration_counters: HashMap<String, usize> = HashMap::new();
    let mut boot_image_counters: HashMap<String, usize> = HashMap::new();

    for node in node_details_list {
        power_status_counters
            .entry(node.power_status)
            .and_modify(|power_status_counter| *power_status_counter += 1)
            .or_insert(1);

        boot_configuration_counters
            .entry(node.boot_configuration)
            .and_modify(|power_status_counter| *power_status_counter += 1)
            .or_insert(1);

        runtime_configuration_counters
            .entry(node.desired_configuration)
            .and_modify(|power_status_counter| *power_status_counter += 1)
            .or_insert(1);

        boot_image_counters
            .entry(node.boot_image_id)
            .and_modify(|power_status_counter| *power_status_counter += 1)
            .or_insert(1);
    }

    let mut table = Table::new();

    table.set_header(vec!["Power status", "Num nodes"]);

    for power_status in ["FAILED", "ON", "OFF", "READY", "STANDBY", "UNCONFIGURED"] {
        table
            .load_preset(comfy_table::presets::ASCII_FULL_CONDENSED)
            .add_row(vec![
                Cell::new(power_status),
                Cell::new(power_status_counters.get(power_status).unwrap_or(&0))
                    .set_alignment(comfy_table::CellAlignment::Center),
            ]);
    }

    println!("{table}");

    let mut table = Table::new();

    table.set_header(vec!["Boot configuration name", "Num nodes"]);

    for (config_name, counter) in boot_configuration_counters {
        table
            .load_preset(comfy_table::presets::ASCII_FULL_CONDENSED)
            .add_row(vec![
                Cell::new(config_name),
                Cell::new(counter).set_alignment(comfy_table::CellAlignment::Center),
            ]);
    }

    println!("{table}");

    let mut table = Table::new();

    table.set_header(vec!["Boot image id", "Num nodes"]);

    for (image_id, counter) in boot_image_counters {
        table
            .load_preset(comfy_table::presets::ASCII_FULL_CONDENSED)
            .add_row(vec![
                Cell::new(image_id),
                Cell::new(counter).set_alignment(comfy_table::CellAlignment::Center),
            ]);
    }

    println!("{table}");

    let mut table = Table::new();

    table.set_header(vec!["Runtime configuration name", "Num nodes"]);

    for (config_name, counter) in runtime_configuration_counters {
        table
            .load_preset(comfy_table::presets::ASCII_FULL_CONDENSED)
            .add_row(vec![
                Cell::new(config_name),
                Cell::new(counter).set_alignment(comfy_table::CellAlignment::Center),
            ]);
    }

    println!("{table}");
}

pub fn nodes_to_string_format_one_line(nodes: Option<&Vec<String>>) -> String {
    if let Some(nodes_content) = nodes {
        nodes_to_string_format_discrete_columns(nodes, nodes_content.len() + 1)
    } else {
        "".to_string()
    }
}

pub fn nodes_to_string_format_discrete_columns(
    nodes: Option<&Vec<String>>,
    num_columns: usize,
) -> String {
    let mut members: String;

    match nodes {
        Some(nodes) if !nodes.is_empty() => {
            members = nodes[0].clone(); // take first element

            for (i, _) in nodes.iter().enumerate().skip(1) {
                // iterate for the rest of the list
                if i % num_columns == 0 {
                    // breaking the cell content into multiple lines (only 2 xnames per line)

                    members.push_str(",\n");
                } else {
                    members.push(',');
                }

                members.push_str(&nodes[i]);
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
