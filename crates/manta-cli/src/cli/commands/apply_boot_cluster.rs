//! Implements the `manta apply boot cluster` command.

use anyhow::Context;

use crate::{
  cli::http_client::MantaClient,
  common::app_context::AppContext,
};

/// Apply a boot configuration to all nodes in a cluster.
#[allow(clippy::too_many_arguments)]
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  new_boot_image_id_opt: Option<&str>,
  new_boot_image_configuration_opt: Option<&str>,
  new_runtime_configuration_opt: Option<&str>,
  new_kernel_parameters_opt: Option<&str>,
  hsm_group_name: &str,
  _assume_yes: bool,
  _do_not_reboot: bool,
  dry_run: bool,
) -> Result<(), anyhow::Error> {
  let server_url = ctx.cli.manta_server_url
    .context("manta server URL must be configured")?;
  let result = MantaClient::new(server_url, ctx.infra.site_name)?
    .apply_boot_config(
      token,
      hsm_group_name,
      new_boot_image_id_opt,
      new_boot_image_configuration_opt,
      new_kernel_parameters_opt,
      new_runtime_configuration_opt,
      dry_run,
    )
    .await?;
  if dry_run {
    println!(
      "Dry-run enabled. No changes persisted into the system\n{}",
      serde_json::to_string_pretty(&result).unwrap_or_default()
    );
  }
  Ok(())
}
