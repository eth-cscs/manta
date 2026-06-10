//! Implements the `manta add kernel-parameters` command.

use crate::common::app_context::AppContext;
use crate::http_client::{MantaClient, OpenApiResultExt};
use crate::openapi_client::types::AddKernelParametersRequest;
use crate::output::action_result;
use anyhow::Error;

pub struct ExecParams<'a> {
  pub kernel_params: &'a str,
  pub hosts_expression: Option<&'a str>,
  pub hsm_group: Option<&'a str>,
  pub overwrite: bool,
  pub dry_run: bool,
  pub output: Option<&'a str>,
}

/// Adds kernel parameters to the specified nodes,
/// optionally overwriting existing values.
/// Reboots the nodes whose kernel params have changed.
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  p: ExecParams<'_>,
) -> Result<(), Error> {
  let xnames_expression = if p.hsm_group.is_none() {
    p.hosts_expression
  } else {
    None
  };
  let client = MantaClient::from_app_ctx(ctx, Some(token))?;
  let result = client
    .openapi
    .add_kernel_parameters(
      client.site_name(),
      &AddKernelParametersRequest {
        params: p.kernel_params.to_string(),
        xnames_expression: xnames_expression.map(str::to_string),
        hsm_group: p.hsm_group.map(str::to_string),
        overwrite: Some(p.overwrite),
        project_sbps: Some(false),
        dry_run: Some(p.dry_run),
      },
    )
    .await
    .into_anyhow()?;
  if p.dry_run {
    action_result::print_with_data(
      "Dry-run enabled. No changes persisted into the system.",
      &result,
      p.output,
    )?;
  } else {
    action_result::print("Kernel parameters added.", p.output)?;
  }
  Ok(())
}
