use crate::common::{
  DATETIME_FORMAT, authorization::get_groups_names_available,
};
use anyhow::Context;
use chrono::{DateTime, Local, NaiveDateTime};
use comfy_table::{ContentArrangement, Table};
use manta_backend_dispatcher::{
  interfaces::ims::GetImagesAndDetailsTrait, types::ims::Image,
};

use crate::manta_backend_dispatcher::StaticBackendDispatcher;

/// If filtering by HSM group, then image name must include HSM group name (It assumms each image
/// is built for a specific cluster based on ansible vars used by the CFS session). The reason
/// for this is because CSCS staff deletes all CFS sessions every now and then...
pub async fn exec(
  backend: &StaticBackendDispatcher,
  site_name: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  cli_get_images: &clap::ArgMatches,
  settings_hsm_group_name_opt: Option<&str>,
) -> Result<(), anyhow::Error> {
  let shasta_token =
    crate::common::authentication::get_api_token(backend, site_name).await?;

  let id: Option<&String> = cli_get_images.get_one::<String>("id");
  let hsm_group_name_arg_opt: Option<&str> = cli_get_images
    .try_get_one::<String>("hsm-group")
    .ok()
    .flatten()
    .map(String::as_str);
  let limit: Option<&u8> = cli_get_images.get_one::<u8>("limit");
  let target_hsm_group_vec = get_groups_names_available(
    backend,
    &shasta_token,
    hsm_group_name_arg_opt,
    settings_hsm_group_name_opt,
  )
  .await
  .context("Failed to get available HSM group names")?;

  let image_detail_vec: Vec<(Image, String, String, bool)> = backend
    .get_images_and_details(
      &shasta_token,
      shasta_base_url,
      shasta_root_cert,
      &target_hsm_group_vec,
      id.map(String::as_str),
      limit,
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
        .unwrap_or_default()
        .iter()
        .map(|(key, value)| format!("{key}:{value}"))
        .collect::<Vec<_>>()
        .join("\n"),
    ]);
  }

  println!("{table}");

  Ok(())
}
