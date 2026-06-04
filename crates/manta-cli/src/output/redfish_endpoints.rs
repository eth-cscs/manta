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

#[cfg(test)]
mod tests {
  //! Smoke tests for the Redfish-endpoints renderer. The table
  //! path defensively `.unwrap_or("")` every field, so a missing
  //! key shouldn't crash — pinning that, plus the "unknown format
  //! falls through to table" regression-prevention pattern that
  //! caught the silent-fallthrough bug in `output/template.rs`.

  use super::*;
  use serde_json::json;

  #[test]
  fn print_empty_redfish_endpoints_array_succeeds() {
    let payload = json!({ "RedfishEndpoints": [] });
    assert!(print(&payload, "table").is_ok());
    assert!(print(&payload, "json").is_ok());
  }

  #[test]
  fn print_missing_redfish_endpoints_key_does_not_panic() {
    // `json["RedfishEndpoints"].as_array()` returns None — the
    // table branch's `if let Some(...)` skips, table is empty.
    let payload = json!({});
    assert!(print(&payload, "table").is_ok());
    assert!(print(&payload, "json").is_ok());
  }

  #[test]
  fn print_populated_table_does_not_panic() {
    let payload = json!({
      "RedfishEndpoints": [
        {
          "ID": "x3000c0s1b0",
          "Type": "NodeBMC",
          "FQDN": "x3000c0s1b0.management.zinal",
          "IPAddress": "10.0.0.1",
          "Enabled": true,
          "UUID": "abcd-1234",
        }
      ]
    });
    assert!(print(&payload, "table").is_ok());
  }

  #[test]
  fn print_endpoint_with_missing_fields_uses_empty_defaults() {
    // Each field's `.as_str().unwrap_or("")` (or `.as_bool()…`)
    // ensures a partial payload renders without panic.
    let payload = json!({
      "RedfishEndpoints": [
        { "ID": "x3000c0s1b0" }
      ]
    });
    assert!(print(&payload, "table").is_ok());
  }

  #[test]
  fn print_unknown_format_falls_back_to_table() {
    // Regression-prevention: same class of bug as the silent
    // fallthrough we fixed in `output/template.rs`. Any value
    // other than "json" should produce a table, not silently
    // emit nothing.
    let payload = json!({ "RedfishEndpoints": [] });
    assert!(print(&payload, "garbage").is_ok());
    assert!(print(&payload, "").is_ok());
  }
}
