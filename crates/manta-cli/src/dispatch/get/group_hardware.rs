//! Implements the `manta get group-hardware` command.

use anyhow::Error;

use crate::common::app_context::AppContext;
use crate::http_client::{MantaClient, OpenApiResultExt};
use crate::output;
use manta_shared::types::params::hardware::GetHardwareClusterParams;

/// Parse CLI arguments into typed [`GetHardwareClusterParams`].
fn parse_hardware_cluster_params(
  cli_args: &clap::ArgMatches,
  settings_hsm_group_name_opt: Option<&str>,
) -> GetHardwareClusterParams {
  GetHardwareClusterParams {
    group_name: cli_args.get_one::<String>("GROUP_NAME").cloned(),
    settings_hsm_group_name: settings_hsm_group_name_opt.map(String::from),
  }
}

/// CLI adapter for `manta get group-hardware`.
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  cli_args: &clap::ArgMatches,
) -> Result<(), Error> {
  let params =
    parse_hardware_cluster_params(cli_args, ctx.settings_group_name_opt);
  let output = cli_args
    .get_one::<String>("output")
    .map_or("summary", String::as_str);

  let hsm = params
    .group_name
    .as_deref()
    .or(params.settings_hsm_group_name.as_deref());

  let client = MantaClient::from_app_ctx(ctx, Some(token))?;
  let json = client
    .openapi
    .get_groups_hardware(hsm, client.site_name())
    .await
    .into_anyhow()?;

  output::hardware::print_cluster(&json, output)?;
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  fn hw_cluster_cmd() -> clap::Command {
    crate::build::get::subcommand_get_group_hardware()
  }

  #[test]
  fn parse_positional_only_leaves_settings_unset() {
    let matches =
      hw_cluster_cmd().get_matches_from(["group-hardware", "compute"]);
    let params = parse_hardware_cluster_params(&matches, None);
    assert_eq!(params.group_name.as_deref(), Some("compute"));
    assert!(params.settings_hsm_group_name.is_none());
  }

  #[test]
  fn parse_settings_hsm_group_preserved_alongside_positional() {
    let matches =
      hw_cluster_cmd().get_matches_from(["group-hardware", "compute"]);
    let params = parse_hardware_cluster_params(&matches, Some("default"));
    assert_eq!(params.settings_hsm_group_name.as_deref(), Some("default"));
  }
}
