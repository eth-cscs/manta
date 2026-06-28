//! Implements the `manta add kernel-parameters` command.
//!
//! Appends (or replaces, with `--overwrite`) kernel parameters on the
//! BSS boot-parameter records for nodes selected by `--group` or a
//! hosts expression. Forwards to `POST /api/v1/kernel-parameters`.
//! The endpoint supports a server-side `dry_run` flag; the leaf
//! forwards it verbatim. Nodes whose effective kernel parameters
//! change are rebooted by the server-side workflow.
//!
//! See [`super::super::apply::kernel_parameters`] for the
//! *replace-the-set* variant invoked by `manta apply kernel-parameters`,
//! and [`super::super::delete::kernel_parameters`] for the symmetric
//! removal command.

use crate::common::app_context::AppContext;
use crate::common::clap_ext::resolve_node_target;
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
///
/// # Errors
///
/// Returns an error when the HTTP client cannot be built or when the
/// server's `add_kernel_parameters` call fails.
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  p: ExecParams<'_>,
) -> Result<(), Error> {
  let xnames_expression = resolve_node_target(p.hsm_group, p.hosts_expression);
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
