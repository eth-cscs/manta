//! Table and JSON renderers for hardware inventory output.

use anyhow::{Context, Error};
use comfy_table::{ContentArrangement, Table};
use serde_json::Value;

fn node_summary_row(ns: &Value) -> Vec<String> {
  let xname = ns["xname"].as_str().unwrap_or("").to_string();

  let count = |key: &str| -> String {
    ns[key]
      .as_array()
      .map(|a| a.len().to_string())
      .unwrap_or_default()
  };

  let info = |key: &str| -> String {
    ns[key]
      .as_array()
      .map(|a| {
        a.iter()
          .filter_map(|e| e["info"].as_str())
          .collect::<Vec<_>>()
          .join("\n")
      })
      .unwrap_or_default()
  };

  vec![
    xname,
    count("processors"),
    info("processors"),
    count("memory"),
    count("node_accels"),
    count("node_hsn_nics"),
  ]
}

fn make_table() -> Table {
  let mut table = Table::new();
  table.set_content_arrangement(ContentArrangement::Dynamic);
  table.set_header(vec!["XNAME", "CPUs", "CPU Info", "Memory Modules", "GPUs", "HSN NICs"]);
  table
}

fn print_cluster_table(json: &Value) {
  let mut table = make_table();
  if let Some(summaries) = json["node_summaries"].as_array() {
    for ns in summaries {
      table.add_row(node_summary_row(ns));
    }
  }
  println!("{table}");
}

fn print_node_table(json: &Value) {
  let mut table = make_table();
  if let Some(ns) = json.get("node_summary") {
    table.add_row(node_summary_row(ns));
  }
  println!("{table}");
}

/// Print hardware cluster data in the requested format (`"json"` or `"table"`).
pub fn print_cluster(json: &Value, format: &str) -> Result<(), Error> {
  if format == "json" {
    println!(
      "{}",
      serde_json::to_string_pretty(json)
        .context("Failed to serialize hardware cluster")?
    );
  } else {
    print_cluster_table(json);
  }
  Ok(())
}

/// Print hardware node data in the requested format (`"json"` or `"table"`).
pub fn print_node(json: &Value, format: &str) -> Result<(), Error> {
  if format == "json" {
    println!(
      "{}",
      serde_json::to_string_pretty(json)
        .context("Failed to serialize hardware node")?
    );
  } else {
    print_node_table(json);
  }
  Ok(())
}
