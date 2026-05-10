//! Implements the `manta delete images` command.

use anyhow::Error;

use crate::cli::http_client::MantaClient;
use crate::common::app_context::AppContext;
use crate::service;

/// Delete IMS images and their linked artifacts.
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  image_id_vec: &[&str],
  dry_run: bool,
) -> Result<(), Error> {
  tracing::info!(
    "Executing command to delete images: {}",
    image_id_vec.join(", "),
  );

  if let Some(server_url) = ctx.infra.manta_server_url {
    let result = MantaClient::new(server_url, ctx.infra.site_name)?
      .delete_images(token, image_id_vec, dry_run)
      .await?;
    if dry_run {
      eprintln!("Dry-run enabled. No changes persisted into the system");
      println!("{}", serde_json::to_string_pretty(&result).unwrap_or_default());
    } else {
      println!("Images deleted:\n{}", image_id_vec.join(", "));
    }
    return Ok(());
  }

  if dry_run {
    // Validate only — no actual deletion
    service::image::validate_image_deletion(
      &ctx.infra,
      token,
      image_id_vec,
      ctx.cli.settings_hsm_group_name_opt,
    )
    .await?;

    eprintln!("Dry-run enabled. No changes persisted into the system");
    for image_id in image_id_vec {
      eprintln!("Image {} would be deleted", image_id);
    }
  } else {
    service::image::delete_images(
      &ctx.infra,
      token,
      image_id_vec,
      ctx.cli.settings_hsm_group_name_opt,
    )
    .await?;
  }

  println!("Images deleted:\n{}", image_id_vec.join(", "));

  Ok(())
}
