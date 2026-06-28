//! Implements the `manta apply template` command.
//!
//! Creates a BOS session from an existing session template via
//! `POST /api/v1/sessiontemplates/{name}/session`. The `--operation`
//! argument is one of `boot`, `reboot`, `shutdown` — anything else
//! aborts before any HTTP call. The endpoint honours `dry_run`
//! server-side and returns the created session JSON, which the leaf
//! hands to [`crate::output::action_result::print_with_data`].

use anyhow::Error;

use crate::common::app_context::AppContext;
use crate::http_client::{MantaClient, OpenApiResultExt};
use crate::openapi_client::types::{BosOperation, PostTemplateSessionRequest};
use crate::output::action_result;

pub struct ExecParams<'a> {
  pub session_name: Option<&'a str>,
  pub template_name: &'a str,
  pub operation: &'a str,
  pub limit: &'a str,
  pub include_disabled: bool,
  pub dry_run: bool,
  pub output: Option<&'a str>,
}

/// Create a BOS session template and optionally boot.
///
/// # Errors
///
/// Returns an error when `--operation` is not `boot` / `reboot` /
/// `shutdown`, when the HTTP client cannot be built, or when the
/// `post_template_session` call fails.
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  p: ExecParams<'_>,
) -> Result<(), Error> {
  let operation = match p.operation {
    "boot" => BosOperation::Boot,
    "reboot" => BosOperation::Reboot,
    "shutdown" => BosOperation::Shutdown,
    other => anyhow::bail!(
      "unknown BOS operation '{other}' (expected boot/reboot/shutdown)"
    ),
  };
  let client = MantaClient::from_app_ctx(ctx, Some(token))?;
  let result = client
    .openapi
    .post_template_session(
      p.template_name,
      client.site_name(),
      &PostTemplateSessionRequest {
        operation,
        limit: p.limit.to_string(),
        session_name: p.session_name.map(str::to_string),
        include_disabled: Some(p.include_disabled),
        dry_run: Some(p.dry_run),
      },
    )
    .await
    .into_anyhow()?;
  let message = if p.dry_run {
    "Dry-run enabled. No changes persisted into the system."
  } else {
    "BOS session created."
  };
  action_result::print_with_data(message, &result, p.output)?;
  Ok(())
}
