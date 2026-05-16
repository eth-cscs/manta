//! Implements the `manta delete images` command.

use anyhow::{Context, Error};

use crate::cli::http_client::MantaClient;
use crate::common::app_context::AppContext;

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

  let server_url = ctx.cli.manta_server_url
    .context("manta server URL must be configured")?;
  let result = MantaClient::new(server_url, ctx.infra.site_name)?
    .delete_images(token, image_id_vec, dry_run)
    .await?;
  if dry_run {
    eprintln!("Dry-run enabled. No changes persisted into the system");
    println!("{}", serde_json::to_string_pretty(&result).unwrap_or_default());
  } else {
    println!("Images deleted:\n{}", image_id_vec.join(", "));
  }
  Ok(())
}
