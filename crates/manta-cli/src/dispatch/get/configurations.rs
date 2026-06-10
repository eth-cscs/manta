//! Implements the `manta get configurations` command.

use anyhow::{Context, Error, bail};

use crate::common::app_context::AppContext;
use crate::common::clap_ext::ArgMatchesExt;
use crate::http_client::{MantaClient, OpenApiResultExt};
use crate::output::configuration::print_table_struct;
use manta_shared::types::dto::CfsConfigurationResponse;
use manta_shared::types::params::configuration::GetConfigurationParams;

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

  // The server's response is `Vec<CfsConfigurationResponse>` wire-side;
  // the generated client surfaces it as `serde_json::Value` because
  // the schema is sourced from csm-rs (not declared on the manta
  // server's typed annotations). Round-trip into the typed shape so
  // the table renderer keeps its existing signature.
  let cfs_configuration_vec: Vec<CfsConfigurationResponse> =
    serde_json::from_value(raw)
      .context("Failed to deserialise CFS configurations response")?;

  if cfs_configuration_vec.is_empty() {
    bail!("No CFS configuration found!");
  }

  let output_opt = cli_args.opt_str("output");

  if output_opt == Some("json") {
    println!(
      "{}",
      serde_json::to_string_pretty(&cfs_configuration_vec)
        .context("Failed to serialize CFS configurations to JSON")?
    );
  } else {
    print_table_struct(&cfs_configuration_vec);
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
