//! Implements the `manta delete images` command.
//!
//! Removes IMS images by id (comma-separated list) via
//! `DELETE /api/v1/images?ids=…&dry_run=…`. The server honours
//! `--dry-run` and returns the cascade plan (boot images, S3 artifacts,
//! IMS records) without persisting any changes; live runs return the
//! actually-deleted set.

use anyhow::Error;

use crate::common::app_context::AppContext;
use crate::http_client::{MantaClient, OpenApiResultExt};
use crate::output::action_result;

/// Delete IMS images and their linked artifacts.
///
/// # Errors
///
/// Returns an error when the HTTP client cannot be built or when the
/// `delete_images` call fails.
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  image_id_vec: &[&str],
  dry_run: bool,
  output_opt: Option<&str>,
) -> Result<(), Error> {
  tracing::info!(
    "Executing command to delete images: {}",
    image_id_vec.join(", "),
  );

  let ids = image_id_vec.join(",");
  let client = MantaClient::from_app_ctx(ctx, Some(token))?;
  let result = client
    .openapi
    .delete_images(Some(dry_run), &ids, client.site_name())
    .await
    .into_anyhow()?;
  if dry_run {
    action_result::print_with_data(
      "Dry-run enabled. No changes persisted into the system.",
      &result,
      output_opt,
    )?;
  } else {
    action_result::print(
      &format!("Images deleted: {}", image_id_vec.join(", ")),
      output_opt,
    )?;
  }
  Ok(())
}
