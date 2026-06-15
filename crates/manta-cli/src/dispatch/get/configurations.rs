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
  let site = client.site_name();

  // Fan out: the listing endpoint and the deletion-safety analysis are
  // independent, so let manta-server handle both concurrently. The
  // analysis endpoint already sequences its own upstream calls, so we
  // don't compound the upstream-reset risk that motivated that sequencing.
  let (raw, safety_rows) = tokio::try_join!(
    async {
      client
        .openapi
        .get_configurations(
          group_name,
          params.limit.map(i32::from),
          params.name.as_deref(),
          params.pattern.as_deref(),
          site,
        )
        .await
        .into_anyhow()
    },
    async {
      client
        .openapi
        .get_configuration_analysis(site)
        .await
        .into_anyhow()
    },
  )?;

  // The /configurations response is `Vec<CfsConfigurationResponse>`
  // wire-side; the generated client surfaces it as `serde_json::Value`
  // because the schema is sourced from csm-rs (not declared on the
  // manta server's typed annotations). Round-trip into the typed shape
  // so the table renderer keeps its existing signature.
  let cfs_configuration_vec: Vec<CfsConfigurationResponse> =
    serde_json::from_value(raw)
      .context("Failed to deserialise CFS configurations response")?;

  if cfs_configuration_vec.is_empty() {
    bail!("No CFS configuration found!");
  }

  // name -> safe_to_delete lookup from the analysis endpoint. The
  // analysis returns the full system-wide list; we only consult it
  // by name, so any /configurations filter (group, name, pattern,
  // limit) still applies as the user expects.
  let safety: HashMap<String, bool> = safety_rows
    .into_iter()
    .map(|r| (r.name, r.safe_to_delete))
    .collect();

  let output_opt = cli_args.opt_str("output");

  if output_opt == Some("json") {
    // Inject safe_to_delete into each serialised row so JSON consumers
    // see the same field the table column reflects.
    let enriched: Vec<serde_json::Value> = cfs_configuration_vec
      .iter()
      .map(|cfg| {
        let mut v = serde_json::to_value(cfg)
          .context("Failed to serialize CFS configuration to JSON")?;
        if let serde_json::Value::Object(map) = &mut v {
          let safe = safety.get(&cfg.name).copied();
          map.insert(
            "safe_to_delete".to_string(),
            match safe {
              Some(b) => serde_json::Value::Bool(b),
              None => serde_json::Value::Null,
            },
          );
        }
        Ok::<_, anyhow::Error>(v)
      })
      .collect::<Result<_, _>>()?;
    println!(
      "{}",
      serde_json::to_string_pretty(&enriched)
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
