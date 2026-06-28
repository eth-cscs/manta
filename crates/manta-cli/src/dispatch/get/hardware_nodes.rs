//! Implements the `manta get hardware nodes` command.
//!
//! Hits `GET /hardware/nodes` on `manta-server` to return per-node
//! hardware inventory for the supplied host expression (xnames, NIDs,
//! host-list syntax). Output is rendered by
//! [`crate::output::hardware::print_nodes_list`]; default view is the
//! `table`. See [`super::hardware_group`] for the cluster-wide variant.

use anyhow::{Context, Error};

use crate::common::app_context::AppContext;
use crate::http_client::{MantaClient, OpenApiResultExt};
use crate::output;
use manta_shared::types::api::hardware::GetHardwareNodesListParams;

/// Parse CLI arguments into typed [`GetHardwareNodesListParams`].
///
/// # Errors
///
/// Returns an error if the required `VALUE` positional argument is
/// missing.
fn parse_hardware_nodes_params(
  cli_args: &clap::ArgMatches,
) -> Result<GetHardwareNodesListParams, Error> {
  let xnames = cli_args
    .get_one::<String>("VALUE")
    .context("The 'VALUE' argument must have a value")?
    .clone();
  Ok(GetHardwareNodesListParams {
    host_expression: xnames,
  })
}

/// CLI adapter for `manta get hardware nodes`.
///
/// Consumes clap matches for the `hardware nodes` subcommand
/// (positional host expression, optional `--output`), fetches the
/// inventory, and hands the JSON to
/// [`crate::output::hardware::print_nodes_list`].
///
/// # Errors
///
/// Returns an error if the positional host expression is missing, the
/// HTTP request fails, or the renderer fails.
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  cli_args: &clap::ArgMatches,
) -> Result<(), Error> {
  let params = parse_hardware_nodes_params(cli_args)?;
  let output = cli_args
    .get_one::<String>("output")
    .map_or("table", String::as_str);

  let client = MantaClient::from_app_ctx(ctx, Some(token))?;
  let json = client
    .openapi
    .get_hardware_nodes_list(&params.host_expression, client.site_name())
    .await
    .into_anyhow()?;

  output::hardware::print_nodes_list(&json, output)?;
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  fn hw_nodes_cmd() -> clap::Command {
    crate::build::get::subcommand_get_hardware_nodes()
  }

  #[test]
  fn parse_xnames() {
    let matches =
      hw_nodes_cmd().get_matches_from(["nodes", "x1000c0s0b0n0,x1000c0s0b0n1"]);
    let params = parse_hardware_nodes_params(&matches).unwrap();
    assert_eq!(params.host_expression, "x1000c0s0b0n0,x1000c0s0b0n1");
  }

  #[test]
  fn parse_single_xname() {
    let matches = hw_nodes_cmd().get_matches_from(["nodes", "x1000c0s0b0n0"]);
    let params = parse_hardware_nodes_params(&matches).unwrap();
    assert_eq!(params.host_expression, "x1000c0s0b0n0");
  }
}
