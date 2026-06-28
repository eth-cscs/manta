//! Implements the `manta delete redfish-endpoint` command.
//!
//! Removes a registered Redfish endpoint from HSM via
//! `DELETE /api/v1/redfish-endpoints/{id}`. The endpoint has no
//! `dry_run` flag; `--dry-run` is a client-side short-circuit that
//! prints the URI/payload via
//! [`crate::output::action_result::preview_request`]. Sibling of
//! [`super::super::add::redfish_endpoint`] (creation) and
//! [`super::super::apply::redfish_endpoint`] (update).

use anyhow::Error;

use crate::common::app_context::AppContext;
use crate::http_client::{MantaClient, OpenApiResultExt};
use crate::output::action_result;

/// CLI adapter for `manta delete redfish-endpoint`.
///
/// # Errors
///
/// Returns an error when the HTTP client cannot be built, when the
/// `delete_redfish_endpoint` call fails, or when the dry-run preview
/// fails to serialise.
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  id: &str,
  output_opt: Option<&str>,
  dry_run: bool,
) -> Result<(), Error> {
  if dry_run {
    return action_result::preview_request(
      "DELETE",
      "redfish-endpoint",
      &id,
      output_opt,
    );
  }

  let client = MantaClient::from_app_ctx(ctx, Some(token))?;
  client
    .openapi
    .delete_redfish_endpoint(id, client.site_name())
    .await
    .into_anyhow()?;
  action_result::print(
    &format!("Redfish endpoint for id '{id}' deleted successfully"),
    output_opt,
  )?;
  Ok(())
}

#[cfg(test)]
mod tests {
  /// `--dry-run` parses on `manta delete redfish-endpoints` (long flag).
  #[test]
  fn accepts_dry_run() {
    let result = crate::build::build_cli().try_get_matches_from([
      "manta",
      "delete",
      "redfish-endpoints",
      "--id",
      "x1000c0s0b0",
      "--dry-run",
    ]);
    assert!(
      result.is_ok(),
      "expected --dry-run to parse on `delete redfish-endpoints`: {result:?}"
    );
  }

  /// `-d` short alias also parses.
  #[test]
  fn accepts_dry_run_short_alias() {
    let result = crate::build::build_cli().try_get_matches_from([
      "manta",
      "delete",
      "redfish-endpoints",
      "--id",
      "x1000c0s0b0",
      "-d",
    ]);
    assert!(
      result.is_ok(),
      "expected -d short alias to parse: {result:?}"
    );
  }
}
