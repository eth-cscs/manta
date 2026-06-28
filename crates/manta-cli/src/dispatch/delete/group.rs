//! Implements the `manta delete group` command.
//!
//! Removes an HSM group via `DELETE /api/v1/groups/{label}`.
//! `--force` is forwarded as `?force=true` (server allows removal
//! even when members remain). `--dry-run` prints what the request
//! would be and returns without contacting the server (the endpoint
//! has no body and no server-side dry-run flag, so no
//! `preview_request` JSON envelope is emitted).

use anyhow::Error;

use crate::common::app_context::AppContext;
use crate::http_client::{MantaClient, OpenApiResultExt};
use crate::output::action_result;

/// CLI adapter for `manta delete group`.
///
/// # Errors
///
/// Returns an error when no site is selected, when the HTTP client
/// cannot be built, or when the `delete_group` call fails.
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  label: &str,
  force: bool,
  dry_run: bool,
  output_opt: Option<&str>,
) -> Result<(), Error> {
  // A delete always targets a site; resolve it up front so even a
  // dry-run fails fast when no site is selected.
  let site = ctx.require_site()?;

  if dry_run {
    // DELETE has no JSON body — describe the request that would go to the
    // backend so the user can confirm label + force flag before committing.
    println!(
      "Dry-run: would DELETE group '{label}' on site '{site}' (force={force}); no backend call will be made.",
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
