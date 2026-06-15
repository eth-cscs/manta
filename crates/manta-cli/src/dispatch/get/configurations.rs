//! Implements the `manta get configurations` command.

use std::collections::HashMap;

use anyhow::{Context, Error, bail};

use crate::common::app_context::AppContext;
use crate::common::clap_ext::ArgMatchesExt;
use crate::http_client::{MantaClient, OpenApiResultExt};
use crate::output::configuration::print_table_struct;
use manta_shared::types::dto::CfsConfigurationResponse;
use manta_shared::types::api::configuration::GetConfigurationParams;

/// Parse CLI arguments into typed [`GetConfigurationParams`].
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

  // The server now returns each row as a CfsConfigurationResponse-shaped
  // JSON object with an extra `safe_to_delete: bool|null` sibling field
  // (best-effort: null when the server-side analysis fan-out failed).
  // Read the safety field off each row, then deserialise the same row
  // into the typed CfsConfigurationResponse (unknown `safe_to_delete`
  // field is ignored by serde during deserialisation).
  let raw: Vec<serde_json::Value> = serde_json::from_value(raw)
    .context("Failed to deserialise CFS configurations response")?;
  if raw.is_empty() {
    bail!("No CFS configuration found!");
  }
  let safety: HashMap<String, bool> = raw
    .iter()
    .filter_map(|v| {
      let name = v.get("name")?.as_str()?.to_string();
      let safe = v.get("safe_to_delete")?.as_bool()?;
      Some((name, safe))
    })
    .collect();
  let cfs_configuration_vec: Vec<CfsConfigurationResponse> = raw
    .iter()
    .map(|v| serde_json::from_value(v.clone()))
    .collect::<Result<_, _>>()
    .context("Failed to deserialise CFS configurations response")?;

  // Optional safety filter — mirrors the same flag pair on
  // `manta get analysis configuration`. Configurations whose safety
  // verdict is unknown (server returned `safe_to_delete: null` because
  // the analysis fan-out failed) are excluded from both filtered
  // views — "only-X" doesn't match unknown.
  let only_safe = cli_args.get_flag("only-safe-to-delete");
  let only_unsafe = cli_args.get_flag("only-unsafe-to-delete");
  let cfs_configuration_vec: Vec<CfsConfigurationResponse> = if only_safe
    || only_unsafe
  {
    cfs_configuration_vec
      .into_iter()
      .filter(|cfg| {
        let safe = safety.get(&cfg.name).copied();
        (!only_safe || safe == Some(true))
          && (!only_unsafe || safe == Some(false))
      })
      .collect()
  } else {
    cfs_configuration_vec
  };

  let output_opt = cli_args.opt_str("output");

  if output_opt == Some("json") {
    // The server already includes safe_to_delete on every row; dump
    // the raw response verbatim (filtered the same way as the table).
    let raw_filtered: Vec<&serde_json::Value> = if only_safe || only_unsafe {
      raw
        .iter()
        .filter(|v| {
          let safe = v.get("safe_to_delete").and_then(|s| s.as_bool());
          (!only_safe || safe == Some(true))
            && (!only_unsafe || safe == Some(false))
        })
        .collect()
    } else {
      raw.iter().collect()
    };
    println!(
      "{}",
      serde_json::to_string_pretty(&raw_filtered)
        .context("Failed to serialize CFS configurations to JSON")?
    );
  } else {
    print_table_struct(&cfs_configuration_vec, &safety);
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
