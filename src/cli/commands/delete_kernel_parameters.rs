use crate::common::app_context::AppContext;
use anyhow::Error;

use super::kernel_parameters_common::{self, KernelParamOperation};

/// Deletes the specified kernel parameters from a set of nodes.
/// Reboots the nodes whose kernel params have changed.
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  hsm_group_name_arg_opt: Option<&str>,
  nodes: Option<&str>,
  kernel_params: &str,
  assume_yes: bool,
  do_not_reboot: bool,
  dry_run: bool,
) -> Result<(), Error> {
  // Resolve target nodes from hosts expression, HSM group, or settings
  let xname_vec =
    crate::common::node_ops::resolve_target_nodes(
      ctx.infra.backend,
      token,
      nodes,
      hsm_group_name_arg_opt,
      ctx.cli.settings_hsm_group_name_opt,
    )
    .await?;

  kernel_parameters_common::exec(
    ctx,
    token,
    &xname_vec,
    &KernelParamOperation::Delete {
      params: kernel_params,
    },
    assume_yes,
    do_not_reboot,
    dry_run,
  )
  .await
}
