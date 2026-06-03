//! Implements the `manta delete kernel-parameters` command.

use crate::cli::http_client::MantaClient;
use crate::cli::output::action_result;
use anyhow::Error;
use crate::cli::common::app_context::AppContext;

/// Deletes the specified kernel parameters from a set of nodes.
/// Reboots the nodes whose kernel params have changed.
#[allow(clippy::too_many_arguments)]
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  hsm_group_name_arg_opt: Option<&str>,
  nodes: Option<&str>,
  kernel_params: &str,
  _assume_yes: bool,
  _do_not_reboot: bool,
  dry_run: bool,
  output_opt: Option<&str>,
) -> Result<(), Error> {
  let server_url = ctx.manta_server_url;
  let xnames_expression = if hsm_group_name_arg_opt.is_none() {
    nodes
  } else {
    None
  };
  let result = MantaClient::new(server_url, ctx.site_name)?
    .delete_kernel_parameters(
      token,
      kernel_params,
      xnames_expression,
      hsm_group_name_arg_opt,
      dry_run,
    )
    .await?;
  if dry_run {
    action_result::print_with_data(
      "Dry-run enabled. No changes persisted into the system.",
      &result,
      output_opt,
    )?;
  } else {
    action_result::print("Kernel parameters deleted.", output_opt)?;
  }
  Ok(())
}
