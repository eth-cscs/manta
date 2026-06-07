//! Implements the `manta apply boot nodes` command.

use crate::common::app_context::AppContext;
use crate::http_client::{ApplyBootConfigRequest, MantaClient};
use crate::output::action_result;

use anyhow::Error;

pub struct ExecParams<'a> {
  pub boot_image: Option<&'a str>,
  pub boot_image_configuration: Option<&'a str>,
  pub runtime_configuration: Option<&'a str>,
  pub kernel_parameters: Option<&'a str>,
  pub hosts_expression: &'a str,
  pub dry_run: bool,
  pub output: Option<&'a str>,
}

/// Apply a boot configuration to specific nodes.
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  p: ExecParams<'_>,
) -> Result<(), Error> {
  let result = MantaClient::from_app_ctx(ctx)?
    .apply_boot_config(
      token,
      &ApplyBootConfigRequest {
        hosts_expression: p.hosts_expression,
        boot_image_id: p.boot_image,
        boot_image_configuration: p.boot_image_configuration,
        kernel_parameters: p.kernel_parameters,
        runtime_configuration: p.runtime_configuration,
        dry_run: p.dry_run,
      },
    )
    .await?;
  if p.dry_run {
    action_result::print_with_data(
      "Dry-run enabled. No changes persisted into the system.",
      &result,
      p.output,
    )?;
  } else {
    action_result::print("Boot configuration applied.", p.output)?;
  }
  Ok(())
}
