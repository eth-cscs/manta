//! Implements the `manta add kernel-parameters` command.

use crate::cli::http_client::{AddKernelParametersRequest, MantaClient};
use crate::cli::output::action_result;
use anyhow::Error;
use crate::cli::common::app_context::AppContext;

/// Adds kernel parameters to the specified nodes,
/// optionally overwriting existing values.
/// Reboots the nodes whose kernel params have changed.
#[allow(clippy::too_many_arguments)]
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
  output_opt: Option<&str>,
) -> Result<(), Error> {
  let server_url = ctx.manta_server_url;
  let xnames_expression = if hsm_group_name_arg_opt.is_none() {
    hosts_expression
  } else {
    None
  };
  let result = MantaClient::new(server_url, ctx.site_name)?
    .add_kernel_parameters(
      token,
      &AddKernelParametersRequest {
        params: kernel_params,
        xnames_expression,
        hsm_group: hsm_group_name_arg_opt,
        overwrite,
        project_sbps: false,
        dry_run,
      },
    )
    .await?;
  if dry_run {
    action_result::print_with_data(
      "Dry-run enabled. No changes persisted into the system.",
      &result,
      output_opt,
    )?;
  } else {
    action_result::print("Kernel parameters added.", output_opt)?;
  }
  Ok(())
}
