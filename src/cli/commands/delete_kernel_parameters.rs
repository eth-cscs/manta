use crate::common::{
  self, app_context::AppContext, authentication::get_api_token,
  authorization::get_groups_names_available,
};
use anyhow::{Context, Error};
use manta_backend_dispatcher::interfaces::hsm::group::GroupTrait;

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
  let backend = ctx.backend;
  let site_name = ctx.site_name;
  let settings_hsm_group_name_opt = ctx.settings_hsm_group_name_opt;

  let shasta_token = get_api_token(backend, site_name).await?;

  // Resolve target nodes: either from HSM group or from explicit node list
  let hosts_expression: String = if hsm_group_name_arg_opt.is_some() {
    let hsm_group_name_vec = get_groups_names_available(
      backend,
      &shasta_token,
      hsm_group_name_arg_opt,
      settings_hsm_group_name_opt,
    )
    .await?;
    let hsm_members: Vec<String> = backend
      .get_member_vec_from_group_name_vec(
        &shasta_token,
        &hsm_group_name_vec,
      )
      .await
      .context("Could not fetch HSM groups members")?;
    hsm_members.join(",")
  } else {
    nodes
      .map(str::to_string)
      .context("Neither HSM group nor nodes defined")?
  };

  let xname_vec = common::node_ops::resolve_hosts_expression(
    backend,
    &shasta_token,
    &hosts_expression,
    false,
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
