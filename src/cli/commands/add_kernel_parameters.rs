use crate::common::{self, app_context::AppContext, authentication::get_api_token};
use anyhow::Error;

use super::kernel_parameters_common::{self, KernelParamOperation};

/// Adds kernel parameters to the specified nodes,
/// optionally overwriting existing values.
/// Reboots the nodes whose kernel params have changed.
pub async fn exec(
  ctx: &AppContext<'_>,
  kernel_params: &str,
  hosts_expression: &str,
  overwrite: bool,
  assume_yes: bool,
  do_not_reboot: bool,
  dry_run: bool,
) -> Result<(), Error> {
  let shasta_token =
    get_api_token(ctx.backend, ctx.site_name).await?;

  // Resolve hosts expression to xnames
  let xname_vec = common::node_ops::resolve_hosts_expression(
    ctx.backend,
    &shasta_token,
    hosts_expression,
    false,
  )
  .await?;

  kernel_parameters_common::exec(
    ctx,
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
