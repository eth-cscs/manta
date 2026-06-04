//! Implements the `manta apply kernel-parameters` command.

use crate::common::app_context::AppContext;
use crate::http_client::{ApplyKernelParametersRequest, MantaClient};
use crate::output::action_result;
use anyhow::Error;

pub struct ExecParams<'a> {
  pub kernel_params: &'a str,
  pub hosts_expression: Option<&'a str>,
  pub hsm_group: Option<&'a str>,
  pub dry_run: bool,
  pub output: Option<&'a str>,
}

/// Replaces the kernel parameters for a set of nodes.
/// Reboots the nodes whose kernel params have changed.
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  p: ExecParams<'_>,
) -> Result<(), Error> {
  let server_url = ctx.manta_server_url;
  let xnames_expression = if p.hsm_group.is_none() {
    p.hosts_expression
  } else {
    None
  };
  let result = MantaClient::new(server_url, ctx.site_name)?
    .apply_kernel_parameters(
      token,
      &ApplyKernelParametersRequest {
        xnames_expression,
        hsm_group: p.hsm_group,
        operation: "apply",
        params: p.kernel_params,
        overwrite: false,
        project_sbps: false,
        dry_run: p.dry_run,
      },
    )
    .await?;
  if p.dry_run {
    action_result::print_with_data(
      "Dry-run enabled. No changes persisted into the system.",
      &result,
      p.output,
    )?;
  } else {
    action_result::print("Kernel parameters applied.", p.output)?;
  }
  Ok(())
}
