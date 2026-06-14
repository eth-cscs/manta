//! Table and JSON renderers for `manta get summary`.

use anyhow::{Context, Error};
use comfy_table::{ContentArrangement, Table};

use crate::openapi_client::types::BackendSummary;

/// Print backend-summary rows in the requested format.
pub fn print(rows: &[BackendSummary], output: &str) -> Result<(), Error> {
  if output == "json" {
    println!(
      "{}",
      serde_json::to_string_pretty(rows)
        .context("Failed to serialize summary rows")?
    );
  } else {
    print_table(rows);
  }
  Ok(())
}

fn print_table(rows: &[BackendSummary]) {
  let mut table = Table::new();
  table.set_content_arrangement(ContentArrangement::Dynamic);
  table.set_header(vec![
    "Image ID",
    "Image name",
    "Image created",
    "Built-with configuration",
    "Config last updated",
    "Producing session",
    "Session configuration",
    "Session start",
    "BOS template",
  ]);
  for row in rows {
    table.add_row(vec![
      row.image_id.as_str(),
      row.name.as_str(),
      row.image_created.as_deref().unwrap_or("-"),
      row.configuration_name.as_deref().unwrap_or("-"),
      row.configuration_last_updated.as_deref().unwrap_or("-"),
      row.session_name.as_deref().unwrap_or("-"),
      row.session_configuration_name.as_deref().unwrap_or("-"),
      row.session_start_time.as_deref().unwrap_or("-"),
      row.bos_sessiontemplate.as_deref().unwrap_or("-"),
    ]);
  }
  println!("{table}");
}
