use chrono::{DateTime, Local, NaiveDateTime};
use comfy_table::{ContentArrangement, Table};
use manta_backend_dispatcher::{
  error::Error, interfaces::ims::GetImagesAndDetailsTrait, types::ims::Image,
};

use crate::manta_backend_dispatcher::StaticBackendDispatcher;
use std::collections::HashMap;

/// If filtering by HSM group, then image name must include HSM group name (It assumms each image
/// is built for a specific cluster based on ansible vars used by the CFS session). The reason
/// for this is because CSCS staff deletes all CFS sessions every now and then...
pub async fn exec(
  backend: &StaticBackendDispatcher,
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  hsm_group_name_vec: &[String],
  id_opt: Option<&str>,
  limit_number: Option<&u8>,
) -> Result<(), Error> {
  let image_detail_vec: Vec<(Image, String, String, bool)> = backend
    .get_images_and_details(
      shasta_token,
      shasta_base_url,
      shasta_root_cert,
      hsm_group_name_vec,
      id_opt,
      limit_number,
    )
    .await?;

  // Print data
  let mut table = Table::new();
  table.set_content_arrangement(ContentArrangement::Dynamic);

  table.set_header(vec![
    "Image ID",
    "Name",
    "Creation time",
    "CFS config",
    "HSM groups",
    "Tags", // "BOS sessiontemplate",
            // "CFS session name",
  ]);

  for image_details in image_detail_vec {
    let creation_date = image_details.0.created.as_ref().unwrap();

    // NOTE: CSM can have different date formats, so we need to try to parse it in different
    // ways
    let creation_date = if let Ok(v) = creation_date.parse::<NaiveDateTime>() {
      v.format("%d/%m/%Y %H:%M:%S").to_string()
    } else if let Ok(v) = creation_date.parse::<DateTime<Local>>() {
      v.naive_local().format("%d/%m/%Y %H:%M:%S").to_string()
    } else {
      creation_date.to_string()
    };

    table.add_row(vec![
      image_details.0.id.as_ref().unwrap(),
      &image_details.0.name,
      &creation_date,
      &image_details.1,
      &image_details
        .2
        .split(",")
        .map(|v| v.trim())
        .collect::<Vec<_>>()
        .join("\n"),
      &image_details
        .0
        .metadata
        .unwrap_or(HashMap::new())
        .iter()
        .map(|(key, value)| format!("{key}:{value}"))
        .collect::<Vec<_>>()
        .join("\n"),
    ]);
  }

  println!("{table}");

  Ok(())
}
