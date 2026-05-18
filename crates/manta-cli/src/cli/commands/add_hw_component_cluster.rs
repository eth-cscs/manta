//! Implements the `manta add hardware` command.

use crate::cli::http_client::MantaClient;
use manta_shared::common::app_context::AppContext;

/// Add hardware components to a cluster group (CLI entry point).
pub async fn exec(
  ctx: &AppContext<'_>,
  shasta_token: &str,
  target_hsm_group_name: &str,
  parent_hsm_group_name: &str,
  pattern: &str,
  dryrun: bool,
  create_hsm_group: bool,
) -> anyhow::Result<()> {
  let server_url = ctx.manta_server_url;
  let result = MantaClient::new(server_url, ctx.site_name)?
    .add_hw_component(
      shasta_token,
      target_hsm_group_name,
      parent_hsm_group_name,
      pattern,
      create_hsm_group,
      dryrun,
    )
    .await?;
  if dryrun {
    println!("Dryrun enabled, not modifying the groups on the system.");
  }
  println!(
    "{}",
    serde_json::to_string_pretty(&result).unwrap_or_default()
  );
  Ok(())
}
