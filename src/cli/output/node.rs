use std::collections::HashMap;

use comfy_table::{Cell, ContentArrangement, Table};
use csm_rs::node::types::NodeDetails;

use crate::common::node_ops::string_vec_to_multi_line_string;

/// Print a formatted table of node details. When `wide`
/// is true, an extra column for kernel parameters is shown.
pub fn print_table(nodes_status: Vec<NodeDetails>, wide: bool) {
  let mut table = Table::new();
  table.set_content_arrangement(ContentArrangement::Dynamic);

  let mut header = vec![
    "XNAME",
    "NID",
    "HSM",
    "Power",
    "Runtime Config",
    "Config Status",
    "Enabled",
    "Error #",
    "Image ID",
  ];

  if wide {
    header.push("Kernel Params");
  }

  table.set_header(header);

  for node_status in nodes_status {
    let mut node_vec: Vec<String> = node_status
      .hsm
      .split(',')
      .map(|xname_str| xname_str.trim().to_string())
      .collect();
    node_vec.sort();

    let mut row = vec![
      Cell::new(node_status.xname),
      Cell::new(node_status.nid),
      Cell::new(string_vec_to_multi_line_string(Some(&node_vec), 1)),
      Cell::new(node_status.power_status),
      Cell::new(node_status.desired_configuration),
      Cell::new(node_status.configuration_status),
      Cell::new(node_status.enabled),
      Cell::new(node_status.error_count),
      Cell::new(node_status.boot_image_id),
    ];

    if wide {
      let kernel_params_vec: Vec<&str> =
        node_status.kernel_params.split_whitespace().collect();
      let cell_max_width = kernel_params_vec
        .iter()
        .map(|value| value.len())
        .max()
        .unwrap_or(0);

      let mut kernel_params_string: String = kernel_params_vec
        .first()
        .map(|s| s.to_string())
        .unwrap_or_default();
      let mut cell_width = kernel_params_string.len();

      for kernel_param in kernel_params_vec.iter().skip(1) {
        cell_width += kernel_param.len();

        if cell_width + kernel_param.len() >= cell_max_width {
          kernel_params_string.push('\n');
          cell_width = 0;
        } else {
          kernel_params_string.push(' ');
        }

        kernel_params_string.push_str(kernel_param);
      }

      row.push(Cell::new(kernel_params_string));
    }

    table.add_row(row);
  }

  println!("{table}");
}

/// Print a two-column summary table from a counter hashmap.
fn print_counter_table(
  header: &str,
  value_header: &str,
  counters: &HashMap<String, usize>,
) {
  let mut table = Table::new();
  table.set_header(vec![header, value_header]);
  for (name, count) in counters {
    table
      .load_preset(comfy_table::presets::ASCII_FULL_CONDENSED)
      .add_row(vec![
        Cell::new(name),
        Cell::new(count).set_alignment(comfy_table::CellAlignment::Center),
      ]);
  }
  println!("{table}");
}

/// Print aggregate summary tables showing counts by power
/// status, boot config, runtime config, and boot image.
pub fn print_summary(node_details_list: Vec<NodeDetails>) {
  let mut power_status_counters: HashMap<String, usize> = HashMap::new();
  let mut boot_configuration_counters: HashMap<String, usize> = HashMap::new();
  let mut runtime_configuration_counters: HashMap<String, usize> =
    HashMap::new();
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

  for power_status in
    ["FAILED", "ON", "OFF", "READY", "STANDBY", "UNCONFIGURED"]
  {
    table
      .load_preset(comfy_table::presets::ASCII_FULL_CONDENSED)
      .add_row(vec![
        Cell::new(power_status),
        Cell::new(power_status_counters.get(power_status).unwrap_or(&0))
          .set_alignment(comfy_table::CellAlignment::Center),
      ]);
  }

  println!("{table}");

  print_counter_table(
    "Boot configuration name",
    "Num nodes",
    &boot_configuration_counters,
  );

  print_counter_table("Boot image id", "Num nodes", &boot_image_counters);

  print_counter_table(
    "Runtime configuration name",
    "Num nodes",
    &runtime_configuration_counters,
  );
}
