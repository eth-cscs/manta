//! Implements the `manta delete hardware` command.
//!
//! Removes hardware components matching `--pattern` from a target HSM
//! cluster, returning them to `--parent-group`. Forwards to
//! `DELETE /api/v1/hardware-clusters/{target}/members` with
//! server-side `dry_run` honoured. If `--delete-group` is set the
//! server also removes the now-empty target HSM group. Inverse of
//! [`super::super::add::hardware`].

use anyhow::{Error, anyhow};

use crate::common::app_context::AppContext;
use crate::http_client::{MantaClient, OpenApiResultExt};
use crate::openapi_client::types::DeleteHwComponentRequest;
use crate::output::action_result;

pub struct ExecParams<'a> {
  pub target_group: Option<&'a str>,
  pub parent_group: Option<&'a str>,
  pub pattern: &'a str,
  pub dry_run: bool,
  pub delete_group: bool,
  pub output: Option<&'a str>,
}

/// Remove hardware components from a cluster group.
///
/// # Errors
///
/// Returns an error when neither the CLI nor `cli.toml` supplies a
/// target or parent group, when the HTTP client cannot be built, or
/// when the `delete_hw_component` call fails.
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  p: ExecParams<'_>,
) -> Result<(), Error> {
  let target = p
    .target_group
    .or(ctx.settings_group_name_opt)
    .ok_or_else(|| anyhow!("No target HSM group specified"))?;
  let parent = p
    .parent_group
    .or(ctx.settings_group_name_opt)
    .ok_or_else(|| anyhow!("No parent HSM group specified"))?;
  let client = MantaClient::from_app_ctx(ctx, Some(token))?;
  let result = client
    .openapi
    .delete_hw_component(
      target,
      client.site_name(),
      &DeleteHwComponentRequest {
        parent_cluster: parent.to_string(),
        pattern: p.pattern.to_string(),
        delete_hsm_group: Some(p.delete_group),
        dry_run: Some(p.dry_run),
      },
    )
    .await
    .into_anyhow()?;
  let message = if p.dry_run {
    "Dry run enabled, not modifying the HSM groups on the system."
  } else {
    "Hardware components removed."
  };
  action_result::print_with_data(message, &result, p.output)?;
  Ok(())
}
