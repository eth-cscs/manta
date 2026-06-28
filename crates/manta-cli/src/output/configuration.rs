//! Renderer for [`CfsConfigurationResponse`] (CFS Ansible
//! configurations).
//!
//! Called by `manta get configuration`. Supported output formats:
//! **table only** in this module — the dispatcher emits JSON itself
//! when `-o json` is set, since the wire type is already serializable
//! and bypasses this renderer entirely. Each row pairs a
//! configuration with its `safe_to_delete` verdict sourced from
//! `/analysis/configurations`.

use chrono::{DateTime, Local};
use comfy_table::Table;
use manta_shared::types::dto::CfsConfigurationResponse;

use manta_shared::common::DATETIME_FORMAT;

/// Print CFS configurations as a formatted table. Each row pairs a
/// configuration with its `safe_to_delete` verdict from the server.
pub fn print_table_struct(rows: &[(CfsConfigurationResponse, bool)]) {
  let mut table = Table::new();

  table.set_header(vec![
    "Config Name",
    "Last updated",
    "Layers",
    "Safe to delete",
  ]);

  for (cfs_configuration, safe_to_delete) in rows {
    let mut layers: String = String::new();

    if let Some(first_layer) = cfs_configuration.layers.first() {
      let layers_json = &cfs_configuration.layers;

      layers = format!(
        "Name:     {}\nPlaybook: {}\nCommit:   {}",
        first_layer.name.as_ref().unwrap_or(&String::new()),
        first_layer.playbook,
        first_layer.commit.as_deref().unwrap_or("Not defined"),
      );

      for layer in layers_json.iter().skip(1) {
        layers = format!(
          "{}\n\nName:     {}\nPlaybook: {}\nCommit:   {}",
          layers,
          layer.name.as_ref().unwrap_or(&String::new()),
          layer.playbook,
          layer.commit.as_deref().unwrap_or("Not defined"),
        );
      }
    }

    table.add_row(vec![
      cfs_configuration.name.clone(),
      cfs_configuration
        .last_updated
        .clone()
        .parse::<DateTime<Local>>()
        .map_or_else(
          |_| cfs_configuration.last_updated.clone(),
          |dt| dt.format(DATETIME_FORMAT).to_string(),
        ),
      layers,
      if *safe_to_delete { "yes" } else { "no" }.to_string(),
    ]);
  }

  println!("{table}");
}

#[cfg(test)]
mod tests {
  //! Smoke tests for the CFS configuration renderer. The renderer
  //! reaches into multiple `Option` fields (layer name, commit)
  //! with `unwrap_or` fallbacks, and parses `last_updated` with a
  //! non-strict fallback (raw string when the format is unknown).
  //! Tests pin the happy path + edge cases. Test data is built via
  //! JSON deserialization since the inner `Layer` type isn't
  //! re-exported through `manta_shared` and we don't want to add a
  //! direct `manta-backend-dispatcher` dev-dep just for tests.

  use super::*;
  use serde_json::json;

  fn from_json(value: serde_json::Value) -> CfsConfigurationResponse {
    serde_json::from_value(value).unwrap()
  }

  #[test]
  fn print_empty_list_does_not_panic() {
    print_table_struct(&[]);
  }

  #[test]
  fn print_config_with_no_layers_does_not_panic() {
    // The `if let Some(first_layer)` branch is skipped; layers
    // column ends up empty.
    let cfg = from_json(json!({
      "name": "cfg-empty",
      "last_updated": "2026-06-04T12:00:00Z",
      "layers": [],
    }));
    print_table_struct(&[(cfg, true)]);
  }

  #[test]
  fn print_config_with_single_layer_does_not_panic() {
    let cfg = from_json(json!({
      "name": "cfg-one",
      "last_updated": "2026-06-04T12:00:00Z",
      "layers": [{
        "name": "ss11",
        "clone_url": "https://example.com/repo.git",
        "playbook": "site.yml",
        "commit": "abc123",
      }],
    }));
    print_table_struct(&[(cfg, true)]);
  }

  #[test]
  fn print_config_with_multiple_layers_does_not_panic() {
    // The .skip(1) loop appends additional layers; this exercises it.
    // Also: middle layer omits `commit` (None fallback path).
    let cfg = from_json(json!({
      "name": "cfg-multi",
      "last_updated": "2026-06-04T12:00:00Z",
      "layers": [
        {"name": "ss11", "clone_url": "https://x", "playbook": "a.yml", "commit": "abc"},
        {"clone_url": "https://y", "playbook": "b.yml"},
        {"name": "cscs", "clone_url": "https://z", "playbook": "c.yml", "commit": "def"},
      ],
    }));
    print_table_struct(&[(cfg, false)]);
  }

  #[test]
  fn print_config_with_unparseable_date_falls_back_to_raw_string() {
    // The `last_updated.parse::<DateTime<Local>>()` `.map_or_else`
    // fallback returns the raw string when parsing fails.
    let cfg = from_json(json!({
      "name": "cfg-bad-date",
      "last_updated": "not-a-real-date",
      "layers": [],
    }));
    print_table_struct(&[(cfg, true)]);
  }
}
