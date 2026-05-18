//! Implements the `manta apply template` command.

use anyhow::Error;

use crate::cli::http_client::MantaClient;
use crate::common::app_context::AppContext;

/// Create a BOS session template and optionally boot.
#[allow(clippy::too_many_arguments)]
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  bos_session_name_opt: Option<&str>,
  bos_sessiontemplate_name: &str,
  bos_session_operation: &str,
  limit: &str,
  include_disabled: bool,
  _assume_yes: bool,
  dry_run: bool,
) -> Result<(), Error> {
  let server_url = ctx.manta_server_url;
  let result = MantaClient::new(server_url, ctx.site_name)?
    .apply_template_session(
      token,
      bos_sessiontemplate_name,
      bos_session_operation,
      limit,
      bos_session_name_opt,
      include_disabled,
      dry_run,
    )
    .await?;
  if dry_run {
    println!(
      "Dry-run enabled. No changes persisted into the system\n{}",
      serde_json::to_string_pretty(&result).unwrap_or_default()
    );
  } else {
    println!(
      "BOS session created: {}",
      serde_json::to_string_pretty(&result).unwrap_or_default()
    );
  }
  Ok(())
}
