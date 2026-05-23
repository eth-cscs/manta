//! Implements the `manta apply boot group` command (and the deprecated
//! `manta apply boot cluster` alias that forwards to it).

use crate::cli::http_client::MantaClient;
use crate::cli::output::action_result;
use manta_shared::common::app_context::AppContext;

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
  _assume_yes: bool,
  _do_not_reboot: bool,
  dry_run: bool,
  output_opt: Option<&str>,
) -> Result<(), anyhow::Error> {
  let server_url = ctx.manta_server_url;
  let result = MantaClient::new(server_url, ctx.site_name)?
    .apply_boot_config(
      token,
      hsm_group_name,
      new_boot_image_id_opt,
      new_boot_image_configuration_opt,
      new_kernel_parameters_opt,
      new_runtime_configuration_opt,
      dry_run,
    )
    .await?;
  if dry_run {
    action_result::print_with_data(
      "Dry-run enabled. No changes persisted into the system.",
      &result,
      output_opt,
    )?;
  } else {
    action_result::print("Boot configuration applied.", output_opt)?;
  }
  Ok(())
}
