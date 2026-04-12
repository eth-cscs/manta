use crate::common::{app_context::AppContext, authentication::get_api_token};
use anyhow::Error;

use super::kernel_parameters_common::{self, KernelParamOperation};

/// Deletes the specified kernel parameters from a set of nodes.
/// Reboots the nodes whose kernel params have changed.
pub async fn exec(
  ctx: &AppContext<'_>,
  hsm_group_name_arg_opt: Option<&str>,
  nodes: Option<&str>,
  kernel_params: &str,
  assume_yes: bool,
  do_not_reboot: bool,
  dry_run: bool,
) -> Result<(), Error> {
  let shasta_token =
    get_api_token(ctx.backend, ctx.site_name).await?;

  // Resolve target nodes from hosts expression, HSM group, or settings
  let xname_vec =
    crate::common::node_ops::resolve_target_nodes(
      ctx.backend,
      &shasta_token,
      nodes,
      hsm_group_name_arg_opt,
      ctx.settings_hsm_group_name_opt,
    )
    .await?;

  kernel_parameters_common::exec(
    ctx,
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
