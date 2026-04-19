use anyhow::Context;
use manta_backend_dispatcher::interfaces::hsm::group::GroupTrait;

use crate::{cli::commands::apply_boot_node, common::app_context::AppContext};

/// Apply a boot configuration to all nodes in a cluster.
#[allow(clippy::too_many_arguments)]
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  new_boot_image_id_opt: Option<&str>,
  new_boot_image_configuration_opt: Option<&str>,
  new_runtime_configuration_opt: Option<&str>,
  new_kernel_parameters_opt: Option<&str>,
  hsm_group_name: &str,
  assume_yes: bool,
  do_not_reboot: bool,
  dry_run: bool,
) -> Result<(), anyhow::Error> {
  let xname_vec = ctx
    .infra
    .backend
    .get_member_vec_from_group_name_vec(
      token,
      &[hsm_group_name.to_string()],
    )
    .await
    .context("Failed to get xnames from HSM group")?;

  apply_boot_node::exec(
    ctx,
    token,
    new_boot_image_id_opt,
    new_boot_image_configuration_opt,
    new_runtime_configuration_opt,
    new_kernel_parameters_opt,
    &xname_vec.join(","),
    assume_yes,
    do_not_reboot,
    dry_run,
  )
  .await
  .context("Failed to apply boot configuration to cluster")?;

  Ok(())
}
