//! Implements the `manta get redfish-endpoints` command.

use anyhow::Error;

use crate::common::app_context::AppContext;
use crate::common::clap_ext::ArgMatchesExt;
use crate::http_client::MantaClient;
use crate::output;
use manta_shared::types::params::redfish_endpoints::GetRedfishEndpointsParams;

/// Parse CLI arguments into typed [`GetRedfishEndpointsParams`].
fn parse_redfish_endpoints_params(
  cli_args: &clap::ArgMatches,
) -> GetRedfishEndpointsParams {
  GetRedfishEndpointsParams {
    id: cli_args.opt_string("id"),
    fqdn: cli_args.opt_string("fqdn"),
    uuid: cli_args.opt_string("uuid"),
    macaddr: cli_args.opt_string("macaddr"),
    ipaddress: cli_args.opt_string("ipaddress"),
  }
}

/// CLI adapter for `manta get redfish-endpoints`.
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  cli_args: &clap::ArgMatches,
) -> Result<(), Error> {
  let params = parse_redfish_endpoints_params(cli_args);
  let output = cli_args.opt_str("output").unwrap_or("table");

  let endpoints = MantaClient::from_app_ctx(ctx)?
    .get_redfish_endpoints(token, &params)
    .await?;

  output::redfish_endpoints::print(&endpoints, output)?;
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  fn redfish_cmd() -> clap::Command {
    crate::build::get::subcommand_get_redfish_endpoints()
  }

  #[test]
  fn parse_no_args() {
    let matches = redfish_cmd().get_matches_from(["redfish-endpoints"]);
    let params = parse_redfish_endpoints_params(&matches);
    assert!(params.id.is_none());
    assert!(params.fqdn.is_none());
    assert!(params.uuid.is_none());
    assert!(params.macaddr.is_none());
    assert!(params.ipaddress.is_none());
  }

  #[test]
  fn parse_all_args() {
    let matches = redfish_cmd().get_matches_from([
      "redfish-endpoints",
      "--id",
      "x3000c0s1b0",
      "--fqdn",
      "node1.example.com",
      "--uuid",
      "abc-123",
      "--macaddr",
      "00:11:22:33:44:55",
      "--ipaddress",
      "10.0.0.1",
    ]);
    let params = parse_redfish_endpoints_params(&matches);
    assert_eq!(params.id.as_deref(), Some("x3000c0s1b0"));
    assert_eq!(params.fqdn.as_deref(), Some("node1.example.com"));
    assert_eq!(params.uuid.as_deref(), Some("abc-123"));
    assert_eq!(params.macaddr.as_deref(), Some("00:11:22:33:44:55"));
    assert_eq!(params.ipaddress.as_deref(), Some("10.0.0.1"));
  }
}
