//! Implements the `manta add kernel-parameters` command.

use crate::cli::http_client::MantaClient;
use crate::common::app_context::AppContext;
use anyhow::{Context, Error};

/// Adds kernel parameters to the specified nodes,
/// optionally overwriting existing values.
/// Reboots the nodes whose kernel params have changed.
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  kernel_params: &str,
  hosts_expression: Option<&str>,
  hsm_group_name_arg_opt: Option<&str>,
  overwrite: bool,
  _assume_yes: bool,
  _do_not_reboot: bool,
  dry_run: bool,
) -> Result<(), Error> {
  let server_url = ctx.cli.manta_server_url
    .context("manta server URL must be configured")?;
  let xnames_vec: Option<Vec<String>> = if hsm_group_name_arg_opt.is_none() {
    hosts_expression.map(|e| e.split(',').map(str::trim).map(String::from).collect())
  } else {
    None
  };
  let result = MantaClient::new(server_url, ctx.infra.site_name)?
    .add_kernel_parameters(
      token,
      kernel_params,
      xnames_vec.as_deref(),
      hsm_group_name_arg_opt,
      overwrite,
      false,
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
