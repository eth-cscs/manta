//! Implements the `manta apply boot group` command.

use anyhow::{anyhow, bail};

use crate::common::app_context::AppContext;
use crate::http_client::{MantaClient, OpenApiResultExt};
use crate::openapi_client::types::ApplyBootConfigRequest;
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
///
/// The server's `/boot-config` endpoint takes a hosts expression
/// (xnames / NIDs / hostlist notation), not a group name, so this
/// fetches the group's members first and forwards them as a
/// comma-separated xname list.
///
/// # Errors
///
/// Returns an error when the HTTP client cannot be built, when the
/// `get_groups` lookup fails, when the named group is absent or has
/// no members, or when the `apply_boot_config` call fails.
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  p: ExecParams<'_>,
) -> Result<(), anyhow::Error> {
  let client = MantaClient::from_app_ctx(ctx, Some(token))?;

  let groups = client
    .openapi
    .get_groups(Some(p.hsm_group_name), client.site_name())
    .await
    .into_anyhow()?;
  let group = groups
    .into_iter()
    .next()
    .ok_or_else(|| anyhow!("HSM group '{}' not found", p.hsm_group_name))?;
  let xnames = group.members.and_then(|m| m.ids).unwrap_or_default();
  if xnames.is_empty() {
    bail!("HSM group '{}' has no members", p.hsm_group_name);
  }

  let result = client
    .openapi
    .apply_boot_config(
      client.site_name(),
      &ApplyBootConfigRequest {
        hosts_expression: xnames.join(","),
        boot_image_id: p.boot_image.map(str::to_string),
        boot_image_configuration: p
          .boot_image_configuration
          .map(str::to_string),
        kernel_parameters: p.kernel_parameters.map(str::to_string),
        runtime_configuration: p.runtime_configuration.map(str::to_string),
        dry_run: Some(p.dry_run),
      },
    )
    .await
    .into_anyhow()?;
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
