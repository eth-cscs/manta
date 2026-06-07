//! Implements the `manta delete images` command.

use anyhow::Error;

use crate::common::app_context::AppContext;
use crate::http_client::MantaClient;
use crate::output::action_result;

/// Delete IMS images and their linked artifacts.
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

  let result = MantaClient::from_app_ctx(ctx)?
    .delete_images(token, image_id_vec, dry_run)
    .await?;
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
