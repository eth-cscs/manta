use crate::common::{self, app_context::AppContext};
use anyhow::Error;

use super::kernel_parameters_common::{self, KernelParamOperation};

/// Replaces the kernel parameters for a set of nodes.
/// Reboots the nodes whose kernel params have changed.
pub async fn exec(
  ctx: &AppContext<'_>,
  kernel_params: &str,
  hosts_expression: Option<&str>,
  hsm_group_name_arg_opt: Option<&str>,
  assume_yes: bool,
  do_not_reboot: bool,
  dry_run: bool,
) -> Result<(), Error> {
  let shasta_token =
    common::authentication::get_api_token(ctx.backend, ctx.site_name)
      .await?;

  // Resolve target nodes from hosts expression, HSM group, or settings
  let xname_vec = kernel_parameters_common::resolve_target_nodes(
    ctx.backend,
    &shasta_token,
    hosts_expression,
    hsm_group_name_arg_opt,
    ctx.settings_hsm_group_name_opt,
  )
  .await?;

  kernel_parameters_common::exec(
    ctx,
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
