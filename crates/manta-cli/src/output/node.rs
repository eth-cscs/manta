//! Table and JSON renderers for HSM node output.

use std::collections::HashMap;

use comfy_table::{Cell, ContentArrangement, Table};
use manta_shared::types::dto::NodeDetails;

use crate::common::multi_line::string_vec_to_multi_line_string;

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
        .map(std::string::ToString::to_string)
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

#[cfg(test)]
mod tests {
  //! Smoke tests for the public renderer entry points. The renderers
  //! delegate most layout to `comfy_table` — the value of these tests
  //! is catching panics from things like unwrap-on-None or mis-sized
  //! row construction, especially on edge inputs (empty list, single
  //! node, wide mode).

  use super::*;
  use manta_shared::types::dto::NodeDetails;

  fn empty_node() -> NodeDetails {
    NodeDetails {
      xname: String::new(),
      nid: String::new(),
      hsm: String::new(),
      power_status: String::new(),
      desired_configuration: String::new(),
      configuration_status: String::new(),
      enabled: String::new(),
      error_count: String::new(),
      boot_image_id: String::new(),
      kernel_params: String::new(),
      boot_configuration: String::new(),
    }
  }

  fn populated_node() -> NodeDetails {
    NodeDetails {
      xname: "x3000c0s1b0n0".to_string(),
      nid: "nid001313".to_string(),
      hsm: "zinal,compute".to_string(),
      power_status: "ON".to_string(),
      desired_configuration: "cos-2.3.101".to_string(),
      configuration_status: "configured".to_string(),
      enabled: "true".to_string(),
      error_count: "0".to_string(),
      boot_image_id: "abcd1234".to_string(),
      kernel_params: "ip=dhcp console=ttyS0,115200 crashkernel=512M"
        .to_string(),
      boot_configuration: "boot-cos".to_string(),
    }
  }

  #[test]
  fn print_table_on_empty_does_not_panic() {
    print_table(vec![], false);
    print_table(vec![], true);
  }

  #[test]
  fn print_table_on_single_node_does_not_panic() {
    print_table(vec![populated_node()], false);
  }

  #[test]
  fn print_table_wide_mode_does_not_panic() {
    // Wide mode adds a kernel-params column; the kernel-params
    // wrapping math runs on this code path only.
    print_table(vec![populated_node()], true);
  }

  #[test]
  fn print_table_wide_mode_handles_empty_kernel_params() {
    // Defensive: the wide-mode wrapping loop iterates over
    // whitespace-split tokens; an empty input should not panic.
    print_table(vec![empty_node()], true);
  }

  #[test]
  fn print_table_handles_node_with_multiple_hsm_groups() {
    // The hsm field is split on ',' and sorted before rendering;
    // this exercises that path explicitly.
    let mut node = populated_node();
    node.hsm = "zeta,alpha,gamma,beta".to_string();
    print_table(vec![node], false);
  }

  #[test]
  fn print_summary_on_empty_does_not_panic() {
    print_summary(vec![]);
  }

  #[test]
  fn print_summary_aggregates_across_multiple_nodes() {
    // Three nodes — two ON, one OFF — so the power-status counter
    // table has at least one row with count > 1.
    let mut n1 = populated_node();
    let mut n2 = populated_node();
    let mut n3 = populated_node();
    n1.power_status = "ON".to_string();
    n2.power_status = "ON".to_string();
    n3.power_status = "OFF".to_string();
    print_summary(vec![n1, n2, n3]);
  }
}
