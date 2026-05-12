//! Table and JSON renderers for hardware inventory output.

use std::collections::{HashMap, HashSet};

use anyhow::{bail, Context, Error};
use comfy_table::{Cell, Color, Table};
use manta_backend_dispatcher::types::NodeSummary;
use serde_json::Value;

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

fn count_hw_components(
  info_iter: impl Iterator<Item = Option<String>>,
) -> (HashMap<String, usize>, HashSet<String>) {
  let mut counts: HashMap<String, usize> = HashMap::new();
  for info in info_iter.flatten() {
    counts.entry(info).and_modify(|q| *q += 1).or_insert(1);
  }
  let keys: HashSet<String> = counts.keys().cloned().collect();
  (counts, keys)
}

fn build_details_table(
  headers: &[String],
  rows: &[(String, HashMap<String, usize>)],
) -> Table {
  let mut all_cols: Vec<String> = [
    rows
      .iter()
      .flat_map(|(_, m)| m.keys().cloned())
      .collect::<Vec<_>>(),
    headers.to_vec(),
  ]
  .concat();
  all_cols.sort();
  all_cols.dedup();

  let mut table = Table::new();
  table.set_header([vec!["Node".to_string()], all_cols.clone()].concat());

  for (xname, counts) in rows {
    let mut row: Vec<Cell> = vec![
      Cell::new(xname).set_alignment(comfy_table::CellAlignment::Center),
    ];
    for col in &all_cols {
      if col.to_uppercase().contains("ERROR")
        && counts.get(col).is_some_and(|n| *n > 0)
      {
        let n = counts.get(col).copied().unwrap_or(0);
        row.push(
          Cell::new(format!("⚠️  ({})", n))
            .fg(Color::Yellow)
            .set_alignment(comfy_table::CellAlignment::Center),
        );
      } else if headers.contains(col) && counts.contains_key(col) {
        let n = counts.get(col).copied().unwrap_or(0);
        row.push(
          Cell::new(format!("✅ ({})", n))
            .fg(Color::Green)
            .set_alignment(comfy_table::CellAlignment::Center),
        );
      } else {
        row.push(
          Cell::new("❌").set_alignment(comfy_table::CellAlignment::Center),
        );
      }
    }
    table.add_row(row);
  }
  table
}

// ---------------------------------------------------------------------------
// Cluster renderers
// ---------------------------------------------------------------------------

fn print_to_terminal_cluster_hw_pattern(
  hsm_group_name: &str,
  pattern: HashMap<String, usize>,
) {
  println!(
    "{}:{}",
    hsm_group_name,
    pattern
      .iter()
      .map(|(hw, qty)| format!("{}:{}", hw, qty))
      .collect::<Vec<_>>()
      .join(":")
  );
}

fn print_table_summary(summary: &HashMap<String, usize>) {
  let mut table = Table::new();
  table.set_header(["HW Component", "Quantity"]);
  for (component, qty) in summary {
    table.add_row(vec![component.as_str(), qty.to_string().as_str()]);
  }
  println!("{table}");
}

fn print_table_details(node_summaries: &[NodeSummary]) {
  let mut rows: Vec<(String, HashMap<String, usize>)> = vec![];
  let mut proc_set: HashSet<String> = HashSet::new();
  let mut accel_set: HashSet<String> = HashSet::new();
  let mut mem_set: HashSet<String> = HashSet::new();
  let mut hsn_set: HashSet<String> = HashSet::new();

  for ns in node_summaries {
    let mut counts: HashMap<String, usize> = HashMap::new();

    let (c, k) =
      count_hw_components(ns.processors.iter().map(|p| p.info.clone()));
    proc_set.extend(k);
    counts.extend(c);

    let (c, k) =
      count_hw_components(ns.node_accels.iter().map(|a| a.info.clone()));
    accel_set.extend(k);
    counts.extend(c);

    let (c, k) = count_hw_components(ns.memory.iter().map(|m| {
      Some(m.info.clone().unwrap_or_else(|| "ERROR".to_string()))
    }));
    mem_set.extend(k);
    counts.extend(c);

    let (c, k) =
      count_hw_components(ns.node_hsn_nics.iter().map(|h| h.info.clone()));
    hsn_set.extend(k);
    counts.extend(c);

    rows.push((ns.xname.clone(), counts));
  }

  rows.sort_by(|a, b| a.0.cmp(&b.0));

  let headers = [
    Vec::from_iter(proc_set),
    Vec::from_iter(accel_set),
    Vec::from_iter(mem_set),
    Vec::from_iter(hsn_set),
  ]
  .concat();

  println!("{}", build_details_table(&headers, &rows));
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Print hardware cluster data in the requested format.
///
/// Accepted values: `"json"`, `"summary"`, `"details"`, `"pattern"`.
pub fn print_cluster(json: &Value, output: &str) -> Result<(), Error> {
  if output == "json" {
    println!(
      "{}",
      serde_json::to_string_pretty(json)
        .context("Failed to serialize hardware cluster")?
    );
    return Ok(());
  }

  let hsm_group_name =
    json["hsm_group_name"].as_str().unwrap_or("").to_string();
  let node_summaries: Vec<NodeSummary> =
    serde_json::from_value(json["node_summaries"].clone())
      .context("Failed to deserialize node summaries")?;

  match output {
    "details" => print_table_details(&node_summaries),
    "summary" => {
      let summary = crate::service::hardware::calculate_hsm_hw_component_summary(
        &node_summaries,
      );
      print_table_summary(&summary);
    }
    "pattern" => {
      let pattern =
        crate::service::hardware::get_cluster_hw_pattern(node_summaries);
      print_to_terminal_cluster_hw_pattern(&hsm_group_name, pattern);
    }
    other => bail!("unsupported output format '{}'", other),
  }
  Ok(())
}

/// Print hardware for an explicit list of nodes in the requested format.
///
/// Accepted values: `"table"` (per-node details table) or `"json"`.
pub fn print_nodes_list(json: &Value, output: &str) -> Result<(), Error> {
  if output == "json" {
    println!(
      "{}",
      serde_json::to_string_pretty(json)
        .context("Failed to serialize hardware nodes list")?
    );
    return Ok(());
  }

  let node_summaries: Vec<NodeSummary> =
    serde_json::from_value(json["node_summaries"].clone())
      .context("Failed to deserialize node summaries")?;
  print_table_details(&node_summaries);
  Ok(())
}

