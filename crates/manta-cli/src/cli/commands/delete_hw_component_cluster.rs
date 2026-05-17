//! Implements the `manta delete hardware` command.

use manta_backend_dispatcher::error::Error;

use crate::{cli::http_client::MantaClient, common::app_context::AppContext};

/// Remove hardware components from a cluster group.
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  target_hsm_group_name_arg_opt: Option<&str>,
  parent_hsm_group_name_arg_opt: Option<&str>,
  pattern: &str,
  dryrun: bool,
  delete_hsm_group: bool,
) -> Result<(), Error> {
  let server_url = ctx.cli.manta_server_url;
  let target = target_hsm_group_name_arg_opt
    .or(ctx.cli.settings_hsm_group_name_opt)
    .ok_or_else(|| {
      Error::NotFound("No target HSM group specified".to_string())
    })?;
  let parent = parent_hsm_group_name_arg_opt
    .or(ctx.cli.settings_hsm_group_name_opt)
    .ok_or_else(|| {
      Error::NotFound("No parent HSM group specified".to_string())
    })?;
  let result = MantaClient::new(server_url, ctx.infra.site_name)
    .map_err(|e| Error::Message(e.to_string()))?
    .delete_hw_component(
      token,
      target,
      parent,
      pattern,
      delete_hsm_group,
      dryrun,
    )
    .await
    .map_err(|e| Error::Message(e.to_string()))?;
  if dryrun {
    println!("Dry run enabled, not modifying the HSM groups on the system.");
  }
  println!(
    "{}",
    serde_json::to_string_pretty(&result).unwrap_or_default()
  );
  Ok(())
}
