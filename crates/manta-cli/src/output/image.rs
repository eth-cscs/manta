//! Renderer for [`Image`] (IMS image records).
//!
//! Called by `manta get images`. Supported output formats:
//! **table only** in this module — JSON is emitted directly off
//! the wire type by the dispatcher. CSM returns the `created`
//! timestamp in several shapes, so the renderer tries
//! `NaiveDateTime` first, then `DateTime<Local>`, then falls back to
//! the raw string.

use std::collections::HashMap;

use chrono::{DateTime, Local, NaiveDateTime};
use comfy_table::{ContentArrangement, Table};
use manta_shared::common::DATETIME_FORMAT;
use manta_shared::types::dto::Image;

/// Print image details as a formatted table.
///
/// `safety` is an `image_id -> safe_to_delete` lookup sourced from
/// `/analysis/images`. Images that aren't in the lookup (or that
/// don't have an `id`) render `?` in the safety column.
pub fn print(image_detail_vec: &[Image], safety: &HashMap<String, bool>) {
  let mut table = Table::new();
  table.set_content_arrangement(ContentArrangement::Dynamic);

  table.set_header(vec![
    "Image ID",
    "Name",
    "Creation time",
    "Configuration",
    "Base",
    "Groups",
    "Tags",
    "Safe to delete",
  ]);

  for image_details in image_detail_vec {
    let unknown = String::from("unknown");
    let creation_date = image_details.created.as_ref().unwrap_or(&unknown);

    // NOTE: CSM can have different date formats, so we need to try to parse it in different
    // ways
    let creation_date = if let Ok(v) = creation_date.parse::<NaiveDateTime>() {
      v.format(DATETIME_FORMAT).to_string()
    } else if let Ok(v) = creation_date.parse::<DateTime<Local>>() {
      v.naive_local().format(DATETIME_FORMAT).to_string()
    } else {
      creation_date.clone()
    };
    let configuration_name =
      image_details.configuration.as_ref().unwrap_or(&unknown);
    let base = image_details.base.as_ref().unwrap_or(&unknown);
    let groups = image_details
      .groups
      .as_ref()
      .map(|group_vec| group_vec.join(", "))
      .unwrap_or(unknown.clone());

    let safety_cell = image_details
      .id
      .as_deref()
      .and_then(|id| safety.get(id))
      .map(|safe| if *safe { "yes" } else { "no" })
      .unwrap_or("?");

    table.add_row(vec![
      image_details.id.as_deref().unwrap_or(&unknown),
      &image_details.name,
      &creation_date,
      configuration_name,
      base,
      &groups,
      &image_details
        .metadata
        .clone()
        .unwrap_or_default()
        .iter()
        .map(|(key, value)| format!("{key}:{value}"))
        .collect::<Vec<_>>()
        .join("\n"),
      safety_cell,
    ]);
  }

  println!("{table}");
}

#[cfg(test)]
mod tests {
  //! Smoke tests for the IMS image renderer. Interesting paths:
  //! multi-format date parsing (NaiveDateTime → DateTime<Local> →
  //! raw string fallback) and the metadata key-value rendering.

  use super::*;
  use serde_json::json;

  fn from_json(value: serde_json::Value) -> Image {
    serde_json::from_value(value).unwrap()
  }

  #[test]
  fn print_empty_list_does_not_panic() {
    print(&[], &HashMap::new());
  }

  #[test]
  fn print_image_with_iso8601_naive_date() {
    // Format that parses as `NaiveDateTime` — first branch.
    let img = from_json(json!({
      "id": "abcd-1234",
      "name": "compute-image-v1",
      "created": "2026-06-04T12:30:00",
    }));
    print(&[img], &HashMap::new());
  }

  #[test]
  fn print_image_with_zoned_date() {
    // Format that parses as `DateTime<Local>` (with timezone) —
    // the NaiveDateTime branch fails, the DateTime branch catches.
    let img = from_json(json!({
      "id": "abcd-1234",
      "name": "compute-image-v1",
      "created": "2026-06-04T12:30:00+00:00",
    }));
    let mut safety = HashMap::new();
    safety.insert("abcd-1234".to_string(), true);
    print(&[img], &safety);
  }

  #[test]
  fn print_image_with_unparseable_date_falls_back_to_raw() {
    // Both parser branches fail → raw string falls through.
    let img = from_json(json!({
      "id": "abcd-1234",
      "name": "image-bad-date",
      "created": "not-a-real-date",
    }));
    print(&[img], &HashMap::new());
  }

  #[test]
  fn print_image_with_missing_date_uses_unknown_placeholder() {
    // `created` is `Option<String>` — None falls back to "unknown".
    let img = from_json(json!({
      "id": "abcd-1234",
      "name": "image-no-date",
    }));
    let mut safety = HashMap::new();
    safety.insert("abcd-1234".to_string(), false);
    print(&[img], &safety);
  }

  #[test]
  fn print_image_with_metadata_renders_key_value_pairs() {
    // Metadata is an Option<HashMap<String, String>>; rendered as
    // `key:value` joined on '\n'.
    let img = from_json(json!({
      "id": "abcd-1234",
      "name": "compute-image-v1",
      "created": "2026-06-04T12:30:00",
      "metadata": { "version": "1.0.0", "owner": "ops" },
    }));
    print(&[img], &HashMap::new());
  }
}
