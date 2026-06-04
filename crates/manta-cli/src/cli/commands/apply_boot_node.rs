//! Implements the `manta apply boot nodes` command.

use crate::cli::http_client::{ApplyBootConfigRequest, MantaClient};
use crate::cli::output::action_result;
use crate::cli::common::app_context::AppContext;

use anyhow::Error;

/// Apply a boot configuration to specific nodes.
#[allow(clippy::too_many_arguments)]
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  new_boot_image_id_opt: Option<&str>,
  new_boot_image_configuration_opt: Option<&str>,
  new_runtime_configuration_opt: Option<&str>,
  new_kernel_parameters_opt: Option<&str>,
  hosts_expression: &str,
  _assume_yes: bool,
  _do_not_reboot: bool,
  dry_run: bool,
  output_opt: Option<&str>,
) -> Result<(), Error> {
  let server_url = ctx.manta_server_url;
  let result = MantaClient::new(server_url, ctx.site_name)?
    .apply_boot_config(
      token,
      &ApplyBootConfigRequest {
        hosts_expression,
        boot_image_id: new_boot_image_id_opt,
        boot_image_configuration: new_boot_image_configuration_opt,
        kernel_parameters: new_kernel_parameters_opt,
        runtime_configuration: new_runtime_configuration_opt,
        dry_run,
      },
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
