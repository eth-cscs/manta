//! Implements the `manta apply boot group` command.

use crate::common::app_context::AppContext;
use crate::http_client::{ApplyBootConfigRequest, MantaClient};
use crate::output::action_result;

pub struct ExecParams<'a> {
  pub boot_image: Option<&'a str>,
  pub boot_image_configuration: Option<&'a str>,
  pub runtime_configuration: Option<&'a str>,
  pub kernel_parameters: Option<&'a str>,
  pub hsm_group_name: &'a str,
  pub dry_run: bool,
  pub output: Option<&'a str>,
}

/// Apply a boot configuration to all nodes in a cluster.
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  p: ExecParams<'_>,
) -> Result<(), anyhow::Error> {
  let result = MantaClient::from_app_ctx(ctx)?
    .apply_boot_config(
      token,
      &ApplyBootConfigRequest {
        // NOTE: the server's /boot-config takes a hosts expression,
        // not a group name. Passing the group label literally as
        // below relies on the server's hostlist parser; in practice
        // this codepath needs a separate "resolve group → xnames"
        // step. Tracked separately from the wire-shape unification.
        hosts_expression: p.hsm_group_name.to_string(),
        boot_image_id: p.boot_image.map(str::to_string),
        boot_image_configuration: p
          .boot_image_configuration
          .map(str::to_string),
        kernel_parameters: p.kernel_parameters.map(str::to_string),
        runtime_configuration: p.runtime_configuration.map(str::to_string),
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
