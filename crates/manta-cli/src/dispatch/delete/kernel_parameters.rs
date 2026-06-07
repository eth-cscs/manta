//! Implements the `manta delete kernel-parameters` command.

use crate::common::app_context::AppContext;
use crate::http_client::MantaClient;
use crate::output::action_result;
use anyhow::Error;

pub struct ExecParams<'a> {
  pub kernel_params: &'a str,
  pub nodes: Option<&'a str>,
  pub hsm_group: Option<&'a str>,
  pub dry_run: bool,
  pub output: Option<&'a str>,
}

/// Deletes the specified kernel parameters from a set of nodes.
/// Reboots the nodes whose kernel params have changed.
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  p: ExecParams<'_>,
) -> Result<(), Error> {
  let xnames_expression = if p.hsm_group.is_none() { p.nodes } else { None };
  let result = MantaClient::from_app_ctx(ctx)?
    .delete_kernel_parameters(
      token,
      p.kernel_params,
      xnames_expression,
      p.hsm_group,
      p.dry_run,
    )
    .await?;
  if p.dry_run {
    action_result::print_with_data(
      "Dry-run enabled. No changes persisted into the system.",
      &result,
      p.output,
    )?;
  } else {
    action_result::print("Kernel parameters deleted.", p.output)?;
  }
  Ok(())
}
