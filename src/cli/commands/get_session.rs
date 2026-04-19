use anyhow::Error;

use crate::cli::output;
use crate::common::app_context::AppContext;
use crate::service::session::{self, GetSessionParams};

/// Parse CLI arguments into typed [`GetSessionParams`].
fn parse_session_params(cli_args: &clap::ArgMatches) -> GetSessionParams {
  let limit = if let Some(true) = cli_args.get_one("most-recent") {
    Some(1u8)
  } else {
    cli_args.get_one::<u8>("limit").copied()
  };

  let mut session_type: Option<String> =
    cli_args.get_one::<String>("type").cloned();
  if session_type.as_deref() == Some("runtime") {
    session_type = Some("dynamic".to_string());
  }

  GetSessionParams {
    hsm_group: cli_args.get_one::<String>("hsm-group").cloned(),
    xnames: cli_args
      .get_one::<String>("xnames")
      .map(|s| s.split(',').map(|x| x.trim().to_string()).collect())
      .unwrap_or_default(),
    min_age: cli_args.get_one::<String>("min-age").cloned(),
    max_age: cli_args.get_one::<String>("max-age").cloned(),
    session_type,
    status: cli_args.get_one::<String>("status").cloned(),
    name: cli_args.get_one::<String>("name").cloned(),
    limit,
  }
}

/// CLI adapter for `manta get sessions`.
///
/// Parses CLI arguments into typed parameters, delegates to
/// the service layer, and formats the output.
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  cli_args: &clap::ArgMatches,
) -> Result<(), Error> {
  let params = parse_session_params(cli_args);

  let sessions = session::get_sessions(
    ctx.backend,
    token,
    ctx.shasta_base_url,
    ctx.shasta_root_cert,
    &params,
  )
  .await?;

  let output_opt = cli_args.get_one::<String>("output").map(String::as_str);
  output::session::print(&sessions, output_opt)?;

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  use clap::{ArgGroup, arg, value_parser};

  /// Build a minimal clap Command matching the real `get sessions`
  /// subcommand definition, so we can create valid `ArgMatches`
  /// for testing.
  fn sessions_cmd() -> clap::Command {
    clap::Command::new("sessions")
      .arg(arg!(-n --name <SESSION_NAME> "session name"))
      .arg(arg!(-a --"min-age" <VALUE> "min age"))
      .arg(arg!(-A --"max-age" <VALUE> "max age"))
      .arg(
        arg!(-t --type <VALUE> "session type")
          .value_parser(["image", "runtime"]),
      )
      .arg(
        arg!(-s --status <VALUE> "status")
          .value_parser(["pending", "running", "complete"]),
      )
      .arg(arg!(-m --"most-recent" "most recent"))
      .arg(
        arg!(-l --limit <VALUE> "limit")
          .value_parser(value_parser!(u8).range(1..)),
      )
      .arg(
        arg!(-o --output <FORMAT> "output format").value_parser(["json"]),
      )
      .arg(arg!(-x --xnames <XNAMES> "xnames"))
      .arg(arg!(-H --"hsm-group" <HSM_GROUP_NAME> "hsm group"))
      .group(
        ArgGroup::new("hsm-group_or_xnames_or_name")
          .args(["hsm-group", "xnames", "name"]),
      )
      .group(
        ArgGroup::new("session_limit").args(["most-recent", "limit"]),
      )
  }

  #[test]
  fn parse_no_args_returns_all_none() {
    let matches = sessions_cmd().get_matches_from(["sessions"]);
    let params = parse_session_params(&matches);
    assert!(params.hsm_group.is_none());
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
    let matches =
      sessions_cmd().get_matches_from(["sessions", "--limit", "5"]);
    let params = parse_session_params(&matches);
    assert_eq!(params.limit, Some(5));
  }

  #[test]
  fn parse_xnames_splits_and_trims() {
    let matches = sessions_cmd()
      .get_matches_from(["sessions", "--xnames", "x1, x2 ,x3"]);
    let params = parse_session_params(&matches);
    assert_eq!(params.xnames, vec!["x1", "x2", "x3"]);
  }

  #[test]
  fn parse_single_xname() {
    let matches =
      sessions_cmd().get_matches_from(["sessions", "--xnames", "x1000c0s0b0n0"]);
    let params = parse_session_params(&matches);
    assert_eq!(params.xnames, vec!["x1000c0s0b0n0"]);
  }

  #[test]
  fn parse_hsm_group() {
    let matches = sessions_cmd()
      .get_matches_from(["sessions", "--hsm-group", "compute"]);
    let params = parse_session_params(&matches);
    assert_eq!(params.hsm_group.as_deref(), Some("compute"));
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
    let matches = sessions_cmd()
      .get_matches_from(["sessions", "--min-age", "1d", "--max-age", "6h"]);
    let params = parse_session_params(&matches);
    assert_eq!(params.min_age.as_deref(), Some("1d"));
    assert_eq!(params.max_age.as_deref(), Some("6h"));
  }
}
