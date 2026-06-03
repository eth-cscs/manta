//! Implements the `manta get hardware nodes` command.

use anyhow::{Context, Error};

use crate::cli::http_client::MantaClient;
use crate::cli::output;
use manta_shared::common::app_context::AppContext;
use manta_shared::shared::params::hardware::GetHardwareNodesListParams;

/// Parse CLI arguments into typed [`GetHardwareNodesListParams`].
fn parse_hardware_nodes_params(
  cli_args: &clap::ArgMatches,
) -> Result<GetHardwareNodesListParams, Error> {
  let xnames = cli_args
    .get_one::<String>("VALUE")
    .context("The 'VALUE' argument must have a value")?
    .clone();
  Ok(GetHardwareNodesListParams { xnames })
}

/// CLI adapter for `manta get hardware nodes`.
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  cli_args: &clap::ArgMatches,
) -> Result<(), Error> {
  let params = parse_hardware_nodes_params(cli_args)?;
  let output = cli_args
    .get_one::<String>("output")
    .map_or("table", String::as_str);

  let server_url = ctx.manta_server_url;
  let json = MantaClient::new(server_url, ctx.site_name)?
    .get_hardware_nodes_list(token, &params)
    .await?;

  output::hardware::print_nodes_list(&json, output)?;
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  fn hw_nodes_cmd() -> clap::Command {
    crate::cli::build::get::subcommand_get_hardware_nodes()
  }

  #[test]
  fn parse_xnames() {
    let matches =
      hw_nodes_cmd().get_matches_from(["nodes", "x1000c0s0b0n0,x1000c0s0b0n1"]);
    let params = parse_hardware_nodes_params(&matches).unwrap();
    assert_eq!(params.xnames, "x1000c0s0b0n0,x1000c0s0b0n1");
  }

  #[test]
  fn parse_single_xname() {
    let matches = hw_nodes_cmd().get_matches_from(["nodes", "x1000c0s0b0n0"]);
    let params = parse_hardware_nodes_params(&matches).unwrap();
    assert_eq!(params.xnames, "x1000c0s0b0n0");
  }
}
