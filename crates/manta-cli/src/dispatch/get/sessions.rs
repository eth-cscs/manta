//! Implements the `manta get sessions` command.
//!
//! Hits `GET /sessions` on `manta-server` to list CFS sessions (image
//! builds and runtime configuration applications) with support for
//! several filters and a `--most-recent` shortcut. The user-facing
//! `runtime` value for `--type` is normalised to CFS's internal
//! `dynamic`. Output is rendered by [`crate::output::session`].

use anyhow::{Context, Error};

use crate::common::app_context::AppContext;
use crate::common::clap_ext::ArgMatchesExt;
use crate::http_client::{MantaClient, OpenApiResultExt};
use crate::output;
use manta_shared::types::api::session::GetSessionParams;
use manta_shared::types::dto::CfsSessionGetResponse;

/// Parse CLI arguments into typed [`GetSessionParams`].
///
/// `--most-recent` forces `limit = Some(1)`, overriding any explicit
/// `--limit` value. `--type runtime` is rewritten to `dynamic` so the
/// value matches CFS's internal session-type vocabulary.
fn parse_session_params(cli_args: &clap::ArgMatches) -> GetSessionParams {
  let limit = if let Some(true) = cli_args.get_one("most-recent") {
    Some(1u8)
  } else {
    cli_args.get_one::<u8>("limit").copied()
  };

  let mut session_type = cli_args.opt_string("type");
  if session_type.as_deref() == Some("runtime") {
    session_type = Some("dynamic".to_string());
  }

  GetSessionParams {
    group: cli_args.opt_string("group"),
    xnames: cli_args
      .opt_string("xnames")
      .map(|s| vec![s])
      .unwrap_or_default(),
    min_age: cli_args.opt_string("min-age"),
    max_age: cli_args.opt_string("max-age"),
    session_type,
    status: cli_args.opt_string("status"),
    name: cli_args.opt_string("name"),
    limit,
  }
}

/// CLI adapter for `manta get sessions`.
///
/// Consumes clap matches for the `sessions` subcommand (`--group`,
/// `--xnames`, `--min-age`, `--max-age`, `--type`, `--status`,
/// `--name`, `--limit`, `--most-recent`, `--output`), calls the server
/// once, and renders the response via [`crate::output::session::print`].
///
/// # Errors
///
/// Returns an error if the HTTP request fails, deserialising the
/// `serde_json::Value` payload into a typed [`CfsSessionGetResponse`]
/// list fails, or the renderer fails.
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  cli_args: &clap::ArgMatches,
) -> Result<(), Error> {
  let params = parse_session_params(cli_args);

  let xnames_csv = if params.xnames.is_empty() {
    None
  } else {
    Some(params.xnames.join(","))
  };

  let client = MantaClient::from_app_ctx(ctx, Some(token))?;
  let raw = client
    .openapi
    .get_sessions(
      params.group.as_deref(),
      params.limit.map(i32::from),
      params.max_age.as_deref(),
      params.min_age.as_deref(),
      params.name.as_deref(),
      params.session_type.as_deref(),
      params.status.as_deref(),
      xnames_csv.as_deref(),
      client.site_name(),
    )
    .await
    .into_anyhow()?;

  let sessions: Vec<CfsSessionGetResponse> = serde_json::from_value(raw)
    .context("Failed to deserialize CFS sessions list")?;

  let output_opt = cli_args.opt_str("output");
  output::session::print(&sessions, output_opt)?;

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  fn sessions_cmd() -> clap::Command {
    crate::build::get::subcommand_get_cfs_session()
  }

  #[test]
  fn parse_no_args_returns_all_none() {
    let matches = sessions_cmd().get_matches_from(["sessions"]);
    let params = parse_session_params(&matches);
    assert!(params.group.is_none());
    assert!(params.xnames.is_empty());
    assert!(params.min_age.is_none());
    assert!(params.max_age.is_none());
    assert!(params.session_type.is_none());
    assert!(params.status.is_none());
    assert!(params.name.is_none());
    assert!(params.limit.is_none());
  }

  #[test]
  fn parse_runtime_type_remapped_to_dynamic() {
    let matches =
      sessions_cmd().get_matches_from(["sessions", "--type", "runtime"]);
    let params = parse_session_params(&matches);
    assert_eq!(params.session_type.as_deref(), Some("dynamic"));
  }

  #[test]
  fn parse_image_type_unchanged() {
    let matches =
      sessions_cmd().get_matches_from(["sessions", "--type", "image"]);
    let params = parse_session_params(&matches);
    assert_eq!(params.session_type.as_deref(), Some("image"));
  }

  #[test]
  fn parse_most_recent_sets_limit_to_one() {
    let matches =
      sessions_cmd().get_matches_from(["sessions", "--most-recent"]);
    let params = parse_session_params(&matches);
    assert_eq!(params.limit, Some(1));
  }

  #[test]
  fn parse_limit_flag() {
    let matches = sessions_cmd().get_matches_from(["sessions", "--limit", "5"]);
    let params = parse_session_params(&matches);
    assert_eq!(params.limit, Some(5));
  }

  #[test]
  fn parse_xnames_passes_expression_verbatim() {
    let matches = sessions_cmd().get_matches_from([
      "sessions",
      "--xnames",
      "x3000c0s1b0n[0-3]",
    ]);
    let params = parse_session_params(&matches);
    assert_eq!(params.xnames, vec!["x3000c0s1b0n[0-3]"]);
  }

  #[test]
  fn parse_single_xname() {
    let matches = sessions_cmd().get_matches_from([
      "sessions",
      "--xnames",
      "x1000c0s0b0n0",
    ]);
    let params = parse_session_params(&matches);
    assert_eq!(params.xnames, vec!["x1000c0s0b0n0"]);
  }

  #[test]
  fn parse_hsm_group() {
    let matches =
      sessions_cmd().get_matches_from(["sessions", "--group", "compute"]);
    let params = parse_session_params(&matches);
    assert_eq!(params.group.as_deref(), Some("compute"));
  }

  #[test]
  fn parse_name_filter() {
    let matches =
      sessions_cmd().get_matches_from(["sessions", "--name", "my-session"]);
    let params = parse_session_params(&matches);
    assert_eq!(params.name.as_deref(), Some("my-session"));
  }

  #[test]
  fn parse_status_filter() {
    let matches =
      sessions_cmd().get_matches_from(["sessions", "--status", "running"]);
    let params = parse_session_params(&matches);
    assert_eq!(params.status.as_deref(), Some("running"));
  }

  #[test]
  fn parse_age_filters() {
    let matches = sessions_cmd().get_matches_from([
      "sessions",
      "--min-age",
      "1d",
      "--max-age",
      "6h",
    ]);
    let params = parse_session_params(&matches);
    assert_eq!(params.min_age.as_deref(), Some("1d"));
    assert_eq!(params.max_age.as_deref(), Some("6h"));
  }
}
