//! Implements the `manta delete nodes` command.
//!
//! Removes (unassigns) the xnames matching a host expression from an
//! HSM group via `DELETE /api/v1/groups/{label}/members`. Always
//! prompts for confirmation before sending the request (no
//! `--assume-yes` plumbing on this leaf). `--dry-run` prints a summary
//! and returns without contacting the server. Inverse of
//! [`super::super::add::nodes`].

use anyhow::{Error, bail};

use crate::common;
use crate::common::app_context::AppContext;
use crate::http_client::{MantaClient, OpenApiResultExt};
use crate::openapi_client::types::DeleteGroupMembersRequest;
use crate::output::action_result;

/// Remove/unassign a list of xnames from a list of HSM groups.
///
/// # Errors
///
/// Returns an error when the user declines the confirmation prompt,
/// when the HTTP client cannot be built, or when the
/// `delete_group_members` call fails.
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  target_hsm_name: &str,
  hosts_expression: &str,
  dryrun: bool,
  output_opt: Option<&str>,
) -> Result<(), Error> {
  if !common::confirm::confirm(
    &format!(
      "Nodes matching '{hosts_expression}' will be removed from HSM group '{target_hsm_name}'. Do you want to proceed?"
    ),
    false,
  ) {
    bail!("Operation cancelled by user");
  }

  if dryrun {
    action_result::print(
      &format!(
        "dryrun - Delete nodes matching '{hosts_expression}' in {target_hsm_name}"
      ),
      output_opt,
    )?;
    return Ok(());
  }

  let client = MantaClient::from_app_ctx(ctx, Some(token))?;
  client
    .openapi
    .delete_group_members(
      target_hsm_name,
      client.site_name(),
      &DeleteGroupMembersRequest {
        xnames_expression: hosts_expression.to_string(),
        dry_run: Some(dryrun),
      },
    )
    .await
    .into_anyhow()?;

  action_result::print(
    &format!(
      "Removed nodes matching '{hosts_expression}' from HSM group '{target_hsm_name}'"
    ),
    output_opt,
  )?;

  Ok(())
}
