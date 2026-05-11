//! Implements the `manta delete kernel-parameters` command.

use crate::cli::http_client::MantaClient;
use crate::common::app_context::AppContext;
use anyhow::{Context, Error};

/// Deletes the specified kernel parameters from a set of nodes.
/// Reboots the nodes whose kernel params have changed.
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  hsm_group_name_arg_opt: Option<&str>,
  nodes: Option<&str>,
  kernel_params: &str,
  _assume_yes: bool,
  _do_not_reboot: bool,
  dry_run: bool,
) -> Result<(), Error> {
  let server_url = ctx.cli.manta_server_url
    .context("manta server URL must be configured")?;
  let xnames_expression = if hsm_group_name_arg_opt.is_none() { nodes } else { None };
  let result = MantaClient::new(server_url, ctx.infra.site_name)?
    .delete_kernel_parameters(
      token,
      kernel_params,
      xnames_expression,
      hsm_group_name_arg_opt,
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
