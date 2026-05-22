//! Implements the `manta delete session` command.

use crate::cli::common;
use crate::cli::http_client::MantaClient;
use manta_shared::common::app_context::AppContext;

/// Delete or cancel a CFS session.
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  session_name: &str,
  dry_run: bool,
  assume_yes: bool,
) -> Result<(), anyhow::Error> {
  let server_url = ctx.manta_server_url;
  if !common::user_interaction::confirm(
    &format!(
      "Session '{session_name}' will get canceled:\nDo you want to continue?",
    ),
    assume_yes,
  ) {
    println!("Operation cancelled by user");
    return Ok(());
  }
  MantaClient::new(server_url, ctx.site_name)?
    .delete_session(token, session_name, dry_run)
    .await?;
  Ok(())
}
