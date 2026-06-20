//! Implements the `manta delete configurations` command, which also
//! cascades to derived artifacts (images, session templates, sessions)
//! — hence the longer filename.

use crate::common::app_context::AppContext;
use crate::http_client::{MantaClient, OpenApiResultExt};
use crate::output::action_result;
use chrono::NaiveDateTime;

pub struct ExecParams<'a> {
  pub configuration_name_pattern: Option<&'a str>,
  pub since: Option<NaiveDateTime>,
  pub until: Option<NaiveDateTime>,
  pub output: Option<&'a str>,
  /// When true, pass `?dry_run=true` to the server: the deletion is
  /// previewed (server returns `DeletionCandidates`) and no records
  /// are modified.
  pub dry_run: bool,
}

/// Delete CFS configurations and their derived artifacts.
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  p: ExecParams<'_>,
) -> Result<(), anyhow::Error> {
  let since_str = p.since.map(|d| d.to_string());
  let until_str = p.until.map(|d| d.to_string());
  let client = MantaClient::from_app_ctx(ctx, Some(token))?;
  let result = client
    .openapi
    .delete_configurations(
      Some(p.dry_run),
      p.configuration_name_pattern,
      since_str.as_deref(),
      until_str.as_deref(),
      client.site_name(),
    )
    .await
    .into_anyhow()?;
  action_result::print_with_data("Configurations deleted", &result, p.output)?;
  Ok(())
}

#[cfg(test)]
mod tests {
  /// `--dry-run` parses on `manta delete configurations` (long flag).
  #[test]
  fn accepts_dry_run() {
    let result = crate::build::build_cli().try_get_matches_from([
      "manta",
      "delete",
      "configurations",
      "-n",
      "test-*",
      "--dry-run",
    ]);
    assert!(
      result.is_ok(),
      "expected --dry-run to parse on `delete configurations`: {result:?}"
    );
  }

  /// `-d` short alias also parses.
  #[test]
  fn accepts_dry_run_short_alias() {
    let result = crate::build::build_cli().try_get_matches_from([
      "manta",
      "delete",
      "configurations",
      "-n",
      "test-*",
      "-d",
    ]);
    assert!(
      result.is_ok(),
      "expected -d short alias to parse: {result:?}"
    );
  }
}
