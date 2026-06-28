//! Implements the `manta add nodes` command.
//!
//! Assigns the xnames matching a host expression to an existing HSM
//! group via `POST /api/v1/groups/{label}/members`. Shows an
//! interactive confirmation prompt unless `--assume-yes` is set.
//! `--dry-run` prints what *would* happen and returns without calling
//! the server. Sibling of [`super::group`] (which can also seed
//! initial members at creation time) and inverse of
//! [`super::super::delete::nodes`].

use anyhow::Error;

use crate::common;
use crate::common::app_context::AppContext;
use crate::http_client::{MantaClient, OpenApiResultExt};
use crate::openapi_client::types::AddNodesToGroupRequest;
use crate::output::action_result;

/// Add/assign a list of xnames to an HSM group.
///
/// # Errors
///
/// Returns an error when the HTTP client cannot be built or when the
/// `add_nodes_to_group` call fails. Declining the confirmation prompt
/// is reported as a successful no-op rather than an error.
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  target_hsm_name: &str,
  hosts_expression: &str,
  dryrun: bool,
  assume_yes: bool,
  output_opt: Option<&str>,
) -> Result<(), Error> {
  if !common::confirm::confirm(
    &format!(
      "Nodes matching '{hosts_expression}' will be added to HSM group '{target_hsm_name}'. Do you want to proceed?"
    ),
    assume_yes,
  ) {
    action_result::print("Operation cancelled by user", output_opt)?;
    return Ok(());
  }

  if dryrun {
    action_result::print(
      &format!(
        "dryrun - Add nodes matching '{hosts_expression}' to {target_hsm_name}"
      ),
      output_opt,
    )?;
    return Ok(());
  }

  let client = MantaClient::from_app_ctx(ctx, Some(token))?;
  let resp = client
    .openapi
    .add_nodes_to_group(
      target_hsm_name,
      client.site_name(),
      &AddNodesToGroupRequest {
        hosts_expression: hosts_expression.to_string(),
      },
    )
    .await
    .into_anyhow()?;

  // Mirror the historical tuple shape (_added, updated_members), then
  // hand the final membership to the action-result printer.
  let updated_members = resp.final_members;

  action_result::print_with_data(
    &format!("HSM '{target_hsm_name}' members updated"),
    &updated_members,
    output_opt,
  )?;

  Ok(())
}
