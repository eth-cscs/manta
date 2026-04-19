use anyhow::Error;

use crate::common::app_context::AppContext;
use crate::service;

/// Delete IMS images and their linked artifacts.
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  image_id_vec: &[&str],
  dry_run: bool,
) -> Result<(), Error> {
  log::info!(
    "Executing command to delete images: {}",
    image_id_vec.join(", "),
  );

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
