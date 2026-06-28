//! Implements the `manta add hardware` command.
//!
//! Adds hardware components matching `--pattern` to a target HSM
//! cluster, drawing the components from `--parent-group`. Forwards to
//! `POST /api/v1/hardware-clusters/{target}/members`. The server
//! honours the request's `dry_run` flag (no client-side
//! short-circuit) and returns the projected component move set even
//! on a real run; the leaf hands the result to
//! [`crate::output::action_result::print_with_data`].
//!
//! Authorisation on both the target and parent HSM groups is enforced
//! by the server.

use crate::common::app_context::AppContext;
use crate::http_client::{MantaClient, OpenApiResultExt};
use crate::openapi_client::types::AddHwComponentRequest;
use crate::output::action_result;

pub struct ExecParams<'a> {
  pub target_group: &'a str,
  pub parent_group: &'a str,
  pub pattern: &'a str,
  pub dry_run: bool,
  pub create_group: bool,
  pub output: Option<&'a str>,
}

/// Add hardware components to a cluster group (CLI entry point).
///
/// # Errors
///
/// Returns an error when the HTTP client cannot be built or when the
/// server's `add_hw_component` call fails (authorisation, validation,
/// or backend errors).
pub async fn exec(
  ctx: &AppContext<'_>,
  shasta_token: &str,
  p: ExecParams<'_>,
) -> anyhow::Result<()> {
  let client = MantaClient::from_app_ctx(ctx, Some(shasta_token))?;
  let result = client
    .openapi
    .add_hw_component(
      p.target_group,
      client.site_name(),
      &AddHwComponentRequest {
        parent_cluster: p.parent_group.to_string(),
        pattern: p.pattern.to_string(),
        create_hsm_group: Some(p.create_group),
        dry_run: Some(p.dry_run),
      },
    )
    .await
    .into_anyhow()?;
  let message = if p.dry_run {
    "Dryrun enabled, not modifying the groups on the system."
  } else {
    "Hardware components added."
  };
  action_result::print_with_data(message, &result, p.output)?;
  Ok(())
}
