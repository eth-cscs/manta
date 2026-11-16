//! Implements the `manta delete group` command.
//!
//! Removes an HSM group via `DELETE /v2/groups/{label}`.
//! `--force` is forwarded as `?force=true` (server allows removal
//! even when members remain). `--dry-run` prints what the request
//! would be without deleting anything (the endpoint has no body and
//! no server-side dry-run flag, so no `preview_request` JSON envelope
//! is emitted). To name the site, the dry-run looks the group up with
//! `GET /v2/groups?name=<label>`, a read, which also lets manta-server
//! resolve the site when none is selected.

use anyhow::Error;

use crate::common::app_context::AppContext;
use crate::http_client::{MantaClient, OpenApiResultExt};
use crate::output::action_result;

/// CLI adapter for `manta delete group`.
///
/// # Errors
///
/// Returns an error when no site is selected (or, for `--dry-run`,
/// none can be resolved), when the HTTP client cannot be built, or
/// when the `get_groups` (dry-run) or `delete_group` call fails.
pub async fn exec(
  ctx: &AppContext<'_>,
  label: &str,
  force: bool,
  dry_run: bool,
  output_opt: Option<&str>,
) -> Result<(), Error> {
  if dry_run {
    // Look the group up — a read — to learn the site the delete would
    // target: the selected one, or the one manta-server resolves from
    // the label. A group that doesn't resolve fails here, as the
    // delete would.
    let client = MantaClient::from_app_ctx_resolving(ctx)?;
    client
      .openapi
      .get_groups(Some(label), client.site_hint())
      .await
      .into_anyhow_pinning_site(&client)
      .await?;
    let site = client.site_name()?;
    // DELETE has no JSON body — describe the request that would go to the
    // backend so the user can confirm label + force flag before committing.
    println!(
      "Dry-run: would DELETE group '{label}' on site '{site}' (force={force}); nothing was deleted.",
    );
    return Ok(());
  }

  let client = MantaClient::from_app_ctx(ctx)?;
  client
    .openapi
    .delete_group(label, Some(force), Some(client.site_name()?))
    .await
    .into_anyhow()
    .await?;

  action_result::print(&format!("Group '{label}' deleted"), output_opt)?;

  Ok(())
}
