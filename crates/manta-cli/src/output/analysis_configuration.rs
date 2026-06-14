//! Table and JSON renderers for `manta get analysis configuration`.

use anyhow::{Context, Error};
use comfy_table::{ContentArrangement, Table};

use crate::openapi_client::types::ConfigurationAnalysis;

pub fn print(
  rows: &[ConfigurationAnalysis],
  output: &str,
) -> Result<(), Error> {
  if output == "json" {
    println!(
      "{}",
      serde_json::to_string_pretty(rows)
        .context("Failed to serialize configuration-analysis rows")?
    );
  } else {
    print_table(rows);
  }
  Ok(())
}

fn print_table(rows: &[ConfigurationAnalysis]) {
  let mut table = Table::new();
  table.set_content_arrangement(ContentArrangement::Dynamic);
  table.set_header(vec![
    "Configuration",
    "Last updated",
    "Safe to delete",
  ]);
  for row in rows {
    table.add_row(vec![
      row.name.as_str(),
      row.last_updated.as_str(),
      if row.safe_to_delete { "yes" } else { "no" },
    ]);
  }
  println!("{table}");
  println!("{} rows", rows.len());
}
