//! Implements the `manta delete node` command.
//!
//! Removes an HSM node record via `DELETE /api/v1/nodes/{id}`. The
//! endpoint has no `dry_run` flag; `--dry-run` is a client-side
//! short-circuit that prints the URI/payload via
//! [`crate::output::action_result::preview_request`].

use anyhow::Error;

use crate::common::app_context::AppContext;
use crate::http_client::{MantaClient, OpenApiResultExt};
use crate::output::action_result;

/// CLI adapter for `manta delete node`.
///
/// # Errors
///
/// Returns an error when the HTTP client cannot be built, when the
/// `delete_node` call fails, or when the dry-run preview fails to
/// serialise.
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  id: &str,
  output_opt: Option<&str>,
  dry_run: bool,
) -> Result<(), Error> {
  if dry_run {
    return action_result::preview_request("DELETE", "node", &id, output_opt);
  }
  let client = MantaClient::from_app_ctx(ctx, Some(token))?;
  client
    .openapi
    .delete_node(id, client.site_name())
    .await
    .into_anyhow()?;
  action_result::print(&format!("Node deleted '{id}'"), output_opt)?;
  Ok(())
}

#[cfg(test)]
mod tests {
  /// `--dry-run` parses on `manta delete node` (long flag).
  #[test]
  fn accepts_dry_run() {
    let result = crate::build::build_cli().try_get_matches_from([
      "manta",
      "delete",
      "node",
      "x1000c0s0b0n0",
      "--dry-run",
    ]);
    assert!(
      result.is_ok(),
      "expected --dry-run to parse on `delete node`: {result:?}"
    );
  }

  /// `-d` short alias also parses.
  #[test]
  fn accepts_dry_run_short_alias() {
    let result = crate::build::build_cli().try_get_matches_from([
      "manta",
      "delete",
      "node",
      "x1000c0s0b0n0",
      "-d",
    ]);
    assert!(
      result.is_ok(),
      "expected -d short alias to parse: {result:?}"
    );
  }
}
