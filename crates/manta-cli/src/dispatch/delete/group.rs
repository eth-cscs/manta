//! Implements the `manta delete group` command.

use anyhow::Error;

use crate::common::app_context::AppContext;
use crate::http_client::{MantaClient, OpenApiResultExt};
use crate::output::action_result;

/// CLI adapter for `manta delete group`.
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  label: &str,
  force: bool,
  dry_run: bool,
  output_opt: Option<&str>,
) -> Result<(), Error> {
  if dry_run {
    // DELETE has no JSON body — describe the request that would go to the
    // backend so the user can confirm label + force flag before committing.
    println!(
      "Dry-run: would DELETE group '{label}' on site '{}' (force={force}); no backend call will be made.",
      ctx.site_name,
    );
    return Ok(());
  }

  let client = MantaClient::from_app_ctx(ctx, Some(token))?;
  client
    .openapi
    .delete_group(label, Some(force), client.site_name())
    .await
    .into_anyhow()?;

  action_result::print(&format!("Group '{label}' deleted"), output_opt)?;

  Ok(())
}
