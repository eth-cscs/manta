//! Implements the `manta get configurations` command.
//!
//! Hits `GET /configurations` on `manta-server` to list CFS
//! configurations along with a deletion-safety verdict. The server
//! returns `ConfigurationAnalysis { configuration, safe_to_delete }`
//! rows; the handler renders either a [`crate::output::configuration`]
//! table or pretty JSON, optionally filtered by `--only-safe-to-delete`
//! / `--only-unsafe-to-delete`.

use anyhow::{Context, Error, bail};

use crate::common::app_context::AppContext;
use crate::common::clap_ext::ArgMatchesExt;
use crate::http_client::{MantaClient, OpenApiResultExt};
use crate::output::configuration::print_table_struct;
use manta_shared::types::api::configuration::GetConfigurationParams;
use manta_shared::types::dto::CfsConfigurationResponse;

/// Parse CLI arguments into typed [`GetConfigurationParams`].
///
/// `--most-recent` forces `limit = Some(1)`, overriding any explicit
/// `--limit` value. `settings_hsm_group_name_opt` is the default group
/// from `cli.toml`, used as a fallback when `--group` is omitted.
fn parse_configuration_params(
  cli_args: &clap::ArgMatches,
  settings_hsm_group_name_opt: Option<&str>,
) -> GetConfigurationParams {
  let limit = if let Some(true) = cli_args.get_one("most-recent") {
    Some(1u8)
  } else {
    cli_args.get_one::<u8>("limit").copied()
  };

  GetConfigurationParams {
    name: cli_args.opt_string("name"),
    pattern: cli_args.opt_string("pattern"),
    group_name: cli_args.opt_string("group"),
    settings_hsm_group_name: settings_hsm_group_name_opt.map(String::from),
    since: None,
    until: None,
    limit,
  }
}

/// CLI adapter for `manta get configurations`.
///
/// Consumes clap matches for the `configurations` subcommand
/// (`--name`, `--pattern`, `--group`, `--limit`, `--most-recent`,
/// `--only-safe-to-delete`, `--only-unsafe-to-delete`, `--output`),
/// fetches the analysis rows, applies the optional safety filter, and
/// renders either a typed table or pretty JSON.
///
/// # Errors
///
/// Returns an error if the HTTP request fails, the server returns no
/// rows, deserialising the embedded `configuration` JSON into
/// [`CfsConfigurationResponse`] fails on the table path, or JSON
/// serialisation fails on the `--output json` path.
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  cli_args: &clap::ArgMatches,
) -> Result<(), Error> {
  let params =
    parse_configuration_params(cli_args, ctx.settings_group_name_opt);

  let group_name = params
    .group_name
    .as_deref()
    .or(params.settings_hsm_group_name.as_deref());

  let client = MantaClient::from_app_ctx(ctx, Some(token))?;
  let raw = client
    .openapi
    .get_configurations(
      group_name,
      params.limit.map(i32::from),
      params.name.as_deref(),
      params.pattern.as_deref(),
      client.site_name(),
    )
    .await
    .into_anyhow()?;

  // The server returns one `ConfigurationAnalysis` per configuration:
  // `{ configuration: CfsConfigurationResponse, safe_to_delete: bool }`.
  // `configuration` rides the wire as `serde_json::Value` because the
  // CFS response type has no `ToSchema` upstream.
  if raw.is_empty() {
    bail!("No CFS configuration found!");
  }

  // Optional safety filter.
  let only_safe = cli_args.get_flag("only-safe-to-delete");
  let only_unsafe = cli_args.get_flag("only-unsafe-to-delete");
  let rows: Vec<_> = if only_safe || only_unsafe {
    raw
      .into_iter()
      .filter(|row| {
        (!only_safe || row.safe_to_delete)
          && (!only_unsafe || !row.safe_to_delete)
      })
      .collect()
  } else {
    raw
  };

  if cli_args.opt_str("output") == Some("json") {
    println!(
      "{}",
      serde_json::to_string_pretty(&rows)
        .context("Failed to serialize CFS configurations to JSON")?
    );
  } else {
    // Deserialise `configuration` into the typed CFS struct only on
    // the table path; JSON output already has the right shape.
    let pairs: Vec<(CfsConfigurationResponse, bool)> = rows
      .into_iter()
      .map(|row| {
        let cfg = serde_json::from_value(row.configuration)?;
        Ok::<_, serde_json::Error>((cfg, row.safe_to_delete))
      })
      .collect::<Result<_, _>>()
      .context("Failed to deserialise CFS configurations response")?;
    print_table_struct(&pairs);
  }

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  fn config_cmd() -> clap::Command {
    crate::build::get::subcommand_get_cfs_configuration()
  }

  #[test]
  fn parse_no_args() {
    let matches = config_cmd().get_matches_from(["configurations"]);
    let params = parse_configuration_params(&matches, None);
    assert!(params.name.is_none());
    assert!(params.pattern.is_none());
    assert!(params.group_name.is_none());
    assert!(params.limit.is_none());
  }

  #[test]
  fn parse_name() {
    let matches =
      config_cmd().get_matches_from(["configurations", "--name", "my-config"]);
    let params = parse_configuration_params(&matches, None);
    assert_eq!(params.name.as_deref(), Some("my-config"));
  }

  #[test]
  fn parse_most_recent_sets_limit_to_one() {
    let matches =
      config_cmd().get_matches_from(["configurations", "--most-recent"]);
    let params = parse_configuration_params(&matches, None);
    assert_eq!(params.limit, Some(1));
  }

  #[test]
  fn parse_pattern() {
    let matches = config_cmd().get_matches_from([
      "configurations",
      "--pattern",
      "compute-*",
    ]);
    let params = parse_configuration_params(&matches, None);
    assert_eq!(params.pattern.as_deref(), Some("compute-*"));
  }
}
