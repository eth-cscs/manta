//! Implements the `manta get groups` command.
//!
//! Hits `GET /groups` on `manta-server` and renders the list of HSM
//! groups visible to the caller. The positional `VALUE` filters to a
//! single group; absent it lists every group. Output is either a
//! [`crate::output::group`] table or a pretty-printed JSON document.

use anyhow::{Context, Error, bail};

use crate::common::app_context::AppContext;
use crate::http_client::{MantaClient, OpenApiResultExt};
use crate::output;
use manta_shared::types::api::group::GetGroupParams;

/// Parse CLI arguments into typed [`GetGroupParams`].
///
/// `settings_hsm_group_name_opt` is the default group from `cli.toml`,
/// carried through so server-side authorization can scope listings even
/// when no positional name is supplied.
fn parse_group_params(
  cli_args: &clap::ArgMatches,
  settings_hsm_group_name_opt: Option<&str>,
) -> GetGroupParams {
  GetGroupParams {
    group_name: cli_args.get_one::<String>("VALUE").cloned(),
    settings_group_name: settings_hsm_group_name_opt.map(String::from),
  }
}

/// CLI adapter for `manta get groups`.
///
/// Consumes clap matches for the `groups` subcommand (optional
/// positional `VALUE` group name; required `--output table|json`),
/// issues a single `get_groups` call, and writes the rendered list to
/// stdout.
///
/// # Errors
///
/// Returns an error if the HTTP client cannot be built, the server
/// rejects the request, the `--output` flag is missing or unrecognised,
/// or JSON serialisation fails.
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  cli_args: &clap::ArgMatches,
) -> Result<(), Error> {
  let params = parse_group_params(cli_args, ctx.settings_group_name_opt);

  let client = MantaClient::from_app_ctx(ctx, Some(token))?;
  let groups = client
    .openapi
    .get_groups(params.group_name.as_deref(), client.site_name())
    .await
    .into_anyhow()?;

  let output: &String = cli_args
    .get_one("output")
    .context("The 'output' argument is mandatory")?;

  match output.as_str() {
    "table" => output::group::print_table(&groups),
    "json" => println!(
      "{}",
      serde_json::to_string_pretty(
        &serde_json::to_value(&groups)
          .context("Failed to convert groups to JSON value")?
      )
      .context("Failed to serialize groups to JSON")?
    ),
    _ => {
      bail!("Output format not valid");
    }
  }

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  fn group_cmd() -> clap::Command {
    crate::build::get::subcommand_get_group()
  }

  #[test]
  fn parse_no_args() {
    let matches = group_cmd().get_matches_from(["groups"]);
    let params = parse_group_params(&matches, None);
    assert!(params.group_name.is_none());
    assert!(params.settings_group_name.is_none());
  }

  #[test]
  fn parse_group_name() {
    let matches = group_cmd().get_matches_from(["groups", "compute"]);
    let params = parse_group_params(&matches, None);
    assert_eq!(params.group_name.as_deref(), Some("compute"));
  }

  #[test]
  fn parse_settings_hsm_group() {
    let matches = group_cmd().get_matches_from(["groups"]);
    let params = parse_group_params(&matches, Some("default-group"));
    assert_eq!(params.settings_group_name.as_deref(), Some("default-group"));
  }
}
