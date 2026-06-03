//! Implements the `manta get group-hardware` command (and the deprecated
//! `manta get hardware cluster` alias that forwards to it).

use anyhow::Error;

use crate::cli::http_client::MantaClient;
use crate::cli::output;
use manta_shared::common::app_context::AppContext;
use manta_shared::shared::params::hardware::GetHardwareClusterParams;

/// Parse CLI arguments into typed [`GetHardwareClusterParams`].
fn parse_hardware_cluster_params(
  cli_args: &clap::ArgMatches,
  settings_hsm_group_name_opt: Option<&str>,
) -> GetHardwareClusterParams {
  GetHardwareClusterParams {
    hsm_group_name: cli_args.get_one::<String>("CLUSTER_NAME").cloned(),
    settings_hsm_group_name: settings_hsm_group_name_opt.map(String::from),
  }
}

/// CLI adapter for `manta get hardware cluster`.
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  cli_args: &clap::ArgMatches,
) -> Result<(), Error> {
  let params =
    parse_hardware_cluster_params(cli_args, ctx.settings_hsm_group_name_opt);
  let output = cli_args
    .get_one::<String>("output")
    .map_or("summary", String::as_str);

  let server_url = ctx.manta_server_url;
  let json = MantaClient::new(server_url, ctx.site_name)?
    .get_hardware_clusters(token, &params)
    .await?;

  output::hardware::print_cluster(&json, output)?;
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  fn hw_cluster_cmd() -> clap::Command {
    crate::cli::build::get::subcommand_get_group_hardware()
  }

  #[test]
  fn parse_positional_only_leaves_settings_unset() {
    let matches =
      hw_cluster_cmd().get_matches_from(["group-hardware", "compute"]);
    let params = parse_hardware_cluster_params(&matches, None);
    assert_eq!(params.hsm_group_name.as_deref(), Some("compute"));
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
