//! Implements the `manta add node` command.

use anyhow::Result;
use std::path::PathBuf;

use crate::common::app_context::AppContext;
use crate::http_client::{MantaClient, OpenApiResultExt};
use crate::openapi_client::types::AddNodeRequest;
use crate::output::action_result;

pub struct ExecParams<'a> {
  pub id: &'a str,
  pub group: &'a str,
  pub enabled: bool,
  pub arch: Option<String>,
  pub hardware_file: Option<&'a PathBuf>,
  pub output: Option<&'a str>,
  /// When true, print the `AddNodeRequest` JSON payload via
  /// `action_result::print_with_data` and return without calling
  /// the server. The server has no `dry_run` support on this
  /// endpoint; the preview is purely client-side.
  pub dry_run: bool,
}

/// CLI adapter for `manta add node`.
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  p: ExecParams<'_>,
) -> Result<()> {
  let _ = p.hardware_file;
  let client = MantaClient::from_app_ctx(ctx, Some(token))?;
  let req = AddNodeRequest {
    id: p.id.to_string(),
    group: p.group.to_string(),
    enabled: Some(p.enabled),
    arch: p.arch,
  };

  if p.dry_run {
    return action_result::preview_request("POST", "/nodes", &req, p.output);
  }

  client
    .openapi
    .add_node(client.site_name(), &req)
    .await
    .into_anyhow()?;

  action_result::print(
    &format!("Node '{}' created and added to group '{}'", p.id, p.group),
    p.output,
  )?;

  Ok(())
}

#[cfg(test)]
mod tests {
  /// `--dry-run` parses on `manta add node` (long flag).
  #[test]
  fn accepts_dry_run() {
    let result = crate::build::build_cli().try_get_matches_from([
      "manta",
      "add",
      "node",
      "--id",
      "x1000c0s0b0n0",
      "--group",
      "compute",
      "--dry-run",
    ]);
    assert!(
      result.is_ok(),
      "expected --dry-run to parse on `add node`: {result:?}"
    );
  }

  /// `-d` short alias also parses.
  #[test]
  fn accepts_dry_run_short_alias() {
    let result = crate::build::build_cli().try_get_matches_from([
      "manta",
      "add",
      "node",
      "--id",
      "x1000c0s0b0n0",
      "--group",
      "compute",
      "-d",
    ]);
    assert!(
      result.is_ok(),
      "expected -d short alias to parse: {result:?}"
    );
  }

  /// `-D` short alias for `--disabled` still parses after the
  /// `-d` -> `-D` swap (the swap is the breaking-change surface;
  /// this test pins it down).
  #[test]
  fn accepts_disabled_short_alias() {
    let result = crate::build::build_cli().try_get_matches_from([
      "manta",
      "add",
      "node",
      "--id",
      "x1000c0s0b0n0",
      "--group",
      "compute",
      "-D",
    ]);
    assert!(
      result.is_ok(),
      "expected -D short alias for --disabled to parse: {result:?}"
    );
  }
}
