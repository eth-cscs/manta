//! Implements the `manta delete session` command.

use crate::common;
use crate::common::app_context::AppContext;
use crate::http_client::{MantaClient, OpenApiResultExt};
use crate::output::action_result;

/// Delete or cancel a CFS session.
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  session_name: &str,
  dry_run: bool,
  assume_yes: bool,
  output_opt: Option<&str>,
) -> Result<(), anyhow::Error> {
  if !common::confirm::confirm(
    &format!(
      "Session '{session_name}' will get canceled:\nDo you want to continue?",
    ),
    assume_yes,
  ) {
    action_result::print("Operation cancelled by user", output_opt)?;
    return Ok(());
  }
  let client = MantaClient::from_app_ctx(ctx, Some(token))?;
  client
    .openapi
    .delete_session(session_name, Some(dry_run), client.site_name())
    .await
    .into_anyhow()?;
  action_result::print(
    &format!("Session '{session_name}' deleted"),
    output_opt,
  )?;
  Ok(())
}
