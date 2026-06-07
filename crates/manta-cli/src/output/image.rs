//! Table and JSON renderers for IMS image output.

use chrono::{DateTime, Local, NaiveDateTime};
use comfy_table::{ContentArrangement, Table};
use manta_shared::common::DATETIME_FORMAT;
use manta_shared::types::dto::Image;

/// Print image details as a formatted table.
pub fn print(image_detail_vec: &[Image]) {
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

    table.add_row(vec![
      image_details.id.as_deref().unwrap_or(&unknown),
      &image_details.name,
      &creation_date,
      &configuration_name,
      &base,
      &groups,
      &image_details
        .metadata
        .clone()
        .unwrap_or_default()
        .iter()
        .map(|(key, value)| format!("{key}:{value}"))
        .collect::<Vec<_>>()
        .join("\n"),
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
    print(&[]);
  }

  #[test]
  fn print_image_with_iso8601_naive_date() {
    // Format that parses as `NaiveDateTime` — first branch.
    let img = from_json(json!({
      "id": "abcd-1234",
      "name": "compute-image-v1",
      "created": "2026-06-04T12:30:00",
    }));
    print(&[img]);
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
    print(&[img]);
  }

  #[test]
  fn print_image_with_unparseable_date_falls_back_to_raw() {
    // Both parser branches fail → raw string falls through.
    let img = from_json(json!({
      "id": "abcd-1234",
      "name": "image-bad-date",
      "created": "not-a-real-date",
    }));
    print(&[img]);
  }

  #[test]
  fn print_image_with_missing_date_uses_unknown_placeholder() {
    // `created` is `Option<String>` — None falls back to "unknown".
    let img = from_json(json!({
      "id": "abcd-1234",
      "name": "image-no-date",
    }));
    print(&[img]);
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
    print(&[img]);
  }
}
