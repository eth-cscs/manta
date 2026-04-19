use crate::common::DATETIME_FORMAT;
use chrono::{DateTime, Local, NaiveDateTime};
use comfy_table::{ContentArrangement, Table};
use manta_backend_dispatcher::types::ims::Image;

/// Print image details as a formatted table.
pub fn print(image_detail_vec: &[(Image, String, String, bool)]) {
  let mut table = Table::new();
  table.set_content_arrangement(ContentArrangement::Dynamic);

  table.set_header(vec![
    "Image ID",
    "Name",
    "Creation time",
    "CFS config",
    "HSM groups",
    "Tags",
  ]);

  for image_details in image_detail_vec {
    let unknown = String::from("unknown");
    let creation_date = image_details.0.created.as_ref().unwrap_or(&unknown);

    // NOTE: CSM can have different date formats, so we need to try to parse it in different
    // ways
    let creation_date = if let Ok(v) = creation_date.parse::<NaiveDateTime>() {
      v.format(DATETIME_FORMAT).to_string()
    } else if let Ok(v) = creation_date.parse::<DateTime<Local>>() {
      v.naive_local().format(DATETIME_FORMAT).to_string()
    } else {
      creation_date.to_string()
    };

    table.add_row(vec![
      image_details.0.id.as_deref().unwrap_or("unknown"),
      &image_details.0.name,
      &creation_date,
      &image_details.1,
      &image_details
        .2
        .split(',')
        .map(|v| v.trim())
        .collect::<Vec<_>>()
        .join("\n"),
      &image_details
        .0
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
