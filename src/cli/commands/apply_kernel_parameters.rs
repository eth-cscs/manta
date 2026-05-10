//! Implements the `manta apply kernel-parameters` command.

use crate::cli::http_client::MantaClient;
use crate::common::app_context::AppContext;
use anyhow::Error;

use super::kernel_parameters_common::{self, KernelParamOperation};

/// Replaces the kernel parameters for a set of nodes.
/// Reboots the nodes whose kernel params have changed.
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  kernel_params: &str,
  hosts_expression: Option<&str>,
  hsm_group_name_arg_opt: Option<&str>,
  assume_yes: bool,
  do_not_reboot: bool,
  dry_run: bool,
) -> Result<(), Error> {
  if let Some(server_url) = ctx.infra.manta_server_url {
    let xnames_vec: Option<Vec<String>> = if hsm_group_name_arg_opt.is_none() {
      hosts_expression.map(|e| e.split(',').map(str::trim).map(String::from).collect())
    } else {
      None
    };
    let result = MantaClient::new(server_url, ctx.infra.site_name)?
      .apply_kernel_parameters(
        token,
        xnames_vec.as_deref(),
        hsm_group_name_arg_opt,
        "apply",
        kernel_params,
        false,
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
    return Ok(());
  }

  // Resolve target nodes from hosts expression, HSM group, or settings
  let xname_vec = kernel_parameters_common::resolve_target_nodes(
    ctx.infra.backend,
    token,
    hosts_expression,
    hsm_group_name_arg_opt,
    ctx.cli.settings_hsm_group_name_opt,
  )
  .await?;

  kernel_parameters_common::exec(
    ctx,
    token,
    &xname_vec,
    &KernelParamOperation::Apply {
      params: kernel_params,
    },
    assume_yes,
    do_not_reboot,
    dry_run,
  )
  .await
}
