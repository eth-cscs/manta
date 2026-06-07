//! Implements the `manta apply template` command.

use anyhow::Error;

use crate::common::app_context::AppContext;
use crate::http_client::{ApplyTemplateSessionRequest, MantaClient};
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
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  p: ExecParams<'_>,
) -> Result<(), Error> {
  let result = MantaClient::from_app_ctx(ctx)?
    .apply_template_session(
      token,
      p.template_name,
      &ApplyTemplateSessionRequest {
        operation: p.operation,
        limit: p.limit,
        session_name: p.session_name,
        include_disabled: p.include_disabled,
        dry_run: p.dry_run,
      },
    )
    .await?;
  let message = if p.dry_run {
    "Dry-run enabled. No changes persisted into the system."
  } else {
    "BOS session created."
  };
  action_result::print_with_data(message, &result, p.output)?;
  Ok(())
}
