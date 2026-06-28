//! Renderer for [`NodeDetails`] (HSM node + power + CFS state).
//!
//! Called by `manta get nodes`. Supported output formats:
//! **table** — the per-node detail table (with an extra
//! kernel-parameter column when `--wide` is set, soft-wrapped to
//! [`KERNEL_PARAMS_WRAP_WIDTH`]) — and a **summary** view that
//! aggregates counts by power status, boot configuration, runtime
//! configuration, and boot image. JSON is emitted by the dispatcher
//! directly off the wire type.

use std::collections::HashMap;

use anyhow::{bail, Context, Result};
use crate::openapi_client::types::NodeDetails;
use comfy_table::{Cell, ContentArrangement, Table};
use manta_shared::types::cluster_status;
use manta_shared::types::dto::NodeDetails as SharedNodeDetails;

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
      row.push(Cell::new(wrap_kernel_params(&node_status.kernel_params)));
    }

    table.add_row(row);
  }

  println!("{table}");
}

/// Target wrap width for the wide-mode kernel-params column. Picked
/// so a typical 80-column terminal still has room for the other nine
/// columns; comfy-table's `ContentArrangement::Dynamic` will narrow
/// further when the rest of the row needs space.
const KERNEL_PARAMS_WRAP_WIDTH: usize = 60;

/// Word-wrap a whitespace-separated `kernel_params` string into lines
/// of at most ~[`KERNEL_PARAMS_WRAP_WIDTH`] columns. Each token stays
/// intact — only the inter-token space is replaced with a newline when
/// adding the next token (plus its leading space) would overflow the
/// current line.
fn wrap_kernel_params(kernel_params: &str) -> String {
  let mut out = String::new();
  let mut line_len = 0;
  for token in kernel_params.split_whitespace() {
    if out.is_empty() {
      out.push_str(token);
      line_len = token.len();
      continue;
    }
    // +1 for the separator (space) that joins the token to the line.
    if line_len + 1 + token.len() > KERNEL_PARAMS_WRAP_WIDTH {
      out.push('\n');
      out.push_str(token);
      line_len = token.len();
    } else {
      out.push(' ');
      out.push_str(token);
      line_len += 1 + token.len();
    }
  }
  out
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

/// Render a node-details list according to the flags passed by the CLI.
///
/// Called by both `dispatch/get/nodes::exec` and
/// `dispatch/get/group_nodes::exec`; the only behavioural difference
/// between the two callers is the `xnames_only` flag (`false` for
/// `nodes`, true when `--xnames-only-one-line` is passed for
/// `group-nodes`).
///
/// # Errors
///
/// Returns an error if JSON serialisation fails or `output_opt` holds
/// an unrecognised value.
pub fn render_node_details(
  list: Vec<NodeDetails>,
  nids_only: bool,
  xnames_only: bool,
  summary_status: bool,
  output_opt: Option<&str>,
) -> Result<()> {
  if summary_status {
    // cluster_status helpers live in manta-shared and consume the
    // shared NodeDetails type. Both types are wire-identical, so
    // round-tripping through JSON is the lightest conversion.
    let shared: Vec<SharedNodeDetails> =
      serde_json::from_value(serde_json::to_value(&list)?)?;
    println!("{}", cluster_status::compute_summary_status(&shared));
  } else if nids_only {
    let node_nid_list: Vec<String> =
      list.iter().map(|nd| nd.nid.clone()).collect();

    if output_opt == Some("json") {
      println!(
        "{}",
        serde_json::to_string(&node_nid_list)
          .context("Failed to serialize node NID list")?
      );
    } else {
      println!("{}", node_nid_list.join(","));
    }
  } else if xnames_only {
    let node_xname_list: Vec<String> =
      list.iter().map(|nd| nd.xname.clone()).collect();

    if output_opt == Some("json") {
      println!(
        "{}",
        serde_json::to_string(&node_xname_list)
          .context("Failed to serialize node xname list")?
      );
    } else {
      println!("{}", node_xname_list.join(","));
    }
  } else {
    match output_opt {
      Some("json") => {
        println!(
          "{}",
          serde_json::to_string_pretty(&list)
            .context("Failed to serialize node details")?
        );
      }
      Some("summary") => {
        print_summary(list);
      }
      Some("table-wide") => {
        print_table(list, true);
      }
      Some("table") => {
        print_table(list, false);
      }
      _ => {
        bail!("Output value not recognized or missing");
      }
    }
  }

  Ok(())
}

#[cfg(test)]
mod tests {
  //! Smoke tests for the public renderer entry points. The renderers
  //! delegate most layout to `comfy_table` — the value of these tests
  //! is catching panics from things like unwrap-on-None or mis-sized
  //! row construction, especially on edge inputs (empty list, single
  //! node, wide mode).

  use super::*;
  use crate::openapi_client::types::NodeDetails;

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
  fn wrap_kernel_params_empty_input_yields_empty_output() {
    assert_eq!(wrap_kernel_params(""), "");
  }

  #[test]
  fn wrap_kernel_params_single_token_does_not_wrap() {
    assert_eq!(wrap_kernel_params("ip=dhcp"), "ip=dhcp");
  }

  #[test]
  fn wrap_kernel_params_joins_short_tokens_with_spaces() {
    // Three short tokens easily fit on one 60-column line.
    let input = "ip=dhcp console=ttyS0,115200 crashkernel=512M";
    let out = wrap_kernel_params(input);
    assert_eq!(out, input);
    assert!(!out.contains('\n'));
  }

  #[test]
  fn wrap_kernel_params_breaks_when_line_would_overflow() {
    // Each token is 25 chars; "tok1 tok2" = 51, "tok1 tok2 tok3" = 77,
    // so the wrap should land between tok2 and tok3 (after 51 chars,
    // before exceeding 60).
    let tok = "x".repeat(25);
    let input = format!("{tok} {tok} {tok}");
    let out = wrap_kernel_params(&input);
    let lines: Vec<&str> = out.split('\n').collect();
    assert_eq!(lines.len(), 2, "expected exactly one break in: {out:?}");
    assert_eq!(lines[0], format!("{tok} {tok}"));
    assert_eq!(lines[1], tok);
  }

  #[test]
  fn wrap_kernel_params_oversized_token_lives_on_its_own_line() {
    // A token longer than the wrap width can't fit anywhere; it lands
    // on a fresh line and stays intact rather than getting truncated.
    let big = "x".repeat(KERNEL_PARAMS_WRAP_WIDTH + 10);
    let input = format!("ip=dhcp {big} console=ttyS0");
    let out = wrap_kernel_params(&input);
    let lines: Vec<&str> = out.split('\n').collect();
    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0], "ip=dhcp");
    assert_eq!(lines[1], big);
    assert_eq!(lines[2], "console=ttyS0");
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
