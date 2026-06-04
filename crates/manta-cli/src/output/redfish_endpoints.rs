//! Table and JSON renderers for Redfish endpoint output.

use anyhow::{Context, Error};
use comfy_table::{ContentArrangement, Table};
use serde_json::Value;

fn print_table(json: &Value) {
  let mut table = Table::new();
  table.set_content_arrangement(ContentArrangement::Dynamic);
  table.set_header(vec!["ID", "Type", "FQDN", "IP Address", "Enabled", "UUID"]);

  if let Some(endpoints) = json["RedfishEndpoints"].as_array() {
    for ep in endpoints {
      table.add_row(vec![
        ep["ID"].as_str().unwrap_or("").to_string(),
        ep["Type"].as_str().unwrap_or("").to_string(),
        ep["FQDN"].as_str().unwrap_or("").to_string(),
        ep["IPAddress"].as_str().unwrap_or("").to_string(),
        ep["Enabled"]
          .as_bool()
          .map(|b| b.to_string())
          .unwrap_or_default(),
        ep["UUID"].as_str().unwrap_or("").to_string(),
      ]);
    }
  }

  println!("{table}");
}

/// Print Redfish endpoints in the requested format (`"json"` or `"table"`).
pub fn print(json: &Value, format: &str) -> Result<(), Error> {
  if format == "json" {
    println!(
      "{}",
      serde_json::to_string_pretty(json)
        .context("Failed to serialize redfish endpoints")?
    );
  } else {
    print_table(json);
  }
  Ok(())
}
