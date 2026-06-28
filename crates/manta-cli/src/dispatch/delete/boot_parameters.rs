//! Implements the `manta delete boot-parameters` command.
//!
//! Removes the BSS boot-parameter records for the given hosts via
//! `DELETE /api/v1/boot-parameters`. `--dry-run` is a client-side
//! short-circuit (the endpoint has no `dry_run` flag): the leaf prints
//! the request payload that *would* be sent via
//! [`crate::output::action_result::preview_request`] and returns.

use anyhow::Error;

use crate::common::app_context::AppContext;
use crate::http_client::{MantaClient, OpenApiResultExt};
use crate::openapi_client::types::DeleteBootParametersRequest;
use crate::output::action_result;

/// CLI adapter for `manta delete boot-parameters`.
///
/// # Errors
///
/// Returns an error when the HTTP client cannot be built, when the
/// `delete_boot_parameters` call fails, or when the dry-run preview
/// fails to serialise.
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  hosts: Vec<String>,
  output_opt: Option<&str>,
  dry_run: bool,
) -> Result<(), Error> {
  let req = DeleteBootParametersRequest { hosts };

  if dry_run {
    return action_result::preview_request(
      "DELETE",
      "boot-parameters",
      &req,
      output_opt,
    );
  }

  let client = MantaClient::from_app_ctx(ctx, Some(token))?;
  client
    .openapi
    .delete_boot_parameters(client.site_name(), &req)
    .await
    .into_anyhow()?;
  action_result::print("Boot parameters deleted successfully", output_opt)?;
  Ok(())
}

#[cfg(test)]
mod tests {
  /// `--dry-run` parses on `manta delete boot-parameters` (long flag).
  #[test]
  fn accepts_dry_run() {
    let result = crate::build::build_cli().try_get_matches_from([
      "manta",
      "delete",
      "boot-parameters",
      "--hosts",
      "x1000c0s0b0n0",
      "--dry-run",
    ]);
    assert!(
      result.is_ok(),
      "expected --dry-run to parse on `delete boot-parameters`: {result:?}"
    );
  }

  /// `-d` short alias also parses.
  #[test]
  fn accepts_dry_run_short_alias() {
    let result = crate::build::build_cli().try_get_matches_from([
      "manta",
      "delete",
      "boot-parameters",
      "--hosts",
      "x1000c0s0b0n0",
      "-d",
    ]);
    assert!(
      result.is_ok(),
      "expected -d short alias to parse: {result:?}"
    );
  }
}
