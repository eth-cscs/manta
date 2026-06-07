//! Implements the `manta get groups` command.

use anyhow::{Context, Error, bail};

use crate::common::app_context::AppContext;
use crate::http_client::MantaClient;
use crate::output;
use manta_shared::types::params::group::GetGroupParams;

/// Parse CLI arguments into typed [`GetGroupParams`].
fn parse_group_params(
  cli_args: &clap::ArgMatches,
  settings_hsm_group_name_opt: Option<&str>,
) -> GetGroupParams {
  GetGroupParams {
    group_name: cli_args.get_one::<String>("VALUE").cloned(),
    settings_hsm_group_name: settings_hsm_group_name_opt.map(String::from),
  }
}

/// CLI adapter for `manta get groups`.
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  cli_args: &clap::ArgMatches,
) -> Result<(), Error> {
  let params = parse_group_params(cli_args, ctx.settings_hsm_group_name_opt);

  let groups = MantaClient::from_app_ctx(ctx)?
    .get_groups(token, &params)
    .await?;

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
    assert!(params.settings_hsm_group_name.is_none());
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
    assert_eq!(
      params.settings_hsm_group_name.as_deref(),
      Some("default-group")
    );
  }
}
