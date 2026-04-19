use crate::common::app_context::AppContext;
use anyhow::Error;

use super::kernel_parameters_common::{self, KernelParamOperation};

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
  assume_yes: bool,
  do_not_reboot: bool,
  dry_run: bool,
) -> Result<(), Error> {
  // Resolve target nodes from hosts expression, HSM group, or settings
  let xname_vec = crate::common::node_ops::resolve_target_nodes(
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
    &KernelParamOperation::Add {
      params: kernel_params,
      overwrite,
    },
    assume_yes,
    do_not_reboot,
    dry_run,
  )
  .await
}
