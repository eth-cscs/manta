//! Implements the `manta apply boot nodes` command.
//!
//! Applies a boot configuration (image id, runtime configuration,
//! kernel parameters) to nodes selected by a hosts expression via
//! `POST /api/v1/boot-config`. Sibling of [`super::boot_group`] which
//! takes a group name and resolves the members first; both leaves
//! forward the request's `dry_run` flag verbatim to the server.

use crate::common::app_context::AppContext;
use crate::http_client::{MantaClient, OpenApiResultExt};
use crate::openapi_client::types::ApplyBootConfigRequest;
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
///
/// # Errors
///
/// Returns an error when the HTTP client cannot be built or when the
/// `apply_boot_config` call fails.
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  p: ExecParams<'_>,
) -> Result<(), Error> {
  let client = MantaClient::from_app_ctx(ctx, Some(token))?;
  let result = client
    .openapi
    .apply_boot_config(
      client.site_name(),
      &ApplyBootConfigRequest {
        hosts_expression: p.hosts_expression.to_string(),
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
