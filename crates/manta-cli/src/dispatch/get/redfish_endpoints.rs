//! Implements the `manta get redfish-endpoints` command.
//!
//! Hits `GET /redfish-endpoints` on `manta-server` to list the BMC
//! Redfish endpoints (BMCs, chassis controllers, PDUs) known to HSM,
//! optionally filtered by id, FQDN, UUID, MAC, or IP. Output defaults
//! to the `table` view in [`crate::output::redfish_endpoints`].

use anyhow::Error;

use crate::common::app_context::AppContext;
use crate::common::clap_ext::ArgMatchesExt;
use crate::http_client::{MantaClient, OpenApiResultExt};
use crate::output;
use manta_shared::types::api::redfish_endpoints::GetRedfishEndpointsParams;

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
///
/// Consumes clap matches for the `redfish-endpoints` subcommand
/// (`--id`, `--fqdn`, `--uuid`, `--macaddr`, `--ipaddress`,
/// `--output`), issues a single `get_redfish_endpoints` call, and
/// hands the response to [`crate::output::redfish_endpoints::print`].
///
/// # Errors
///
/// Returns an error if the HTTP request fails or the renderer fails.
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  cli_args: &clap::ArgMatches,
) -> Result<(), Error> {
  let params = parse_redfish_endpoints_params(cli_args);
  let output = cli_args.opt_str("output").unwrap_or("table");

  let client = MantaClient::from_app_ctx(ctx, Some(token))?;
  let endpoints = client
    .openapi
    .get_redfish_endpoints(
      params.fqdn.as_deref(),
      params.id.as_deref(),
      params.ipaddress.as_deref(),
      params.macaddr.as_deref(),
      params.uuid.as_deref(),
      client.site_name(),
    )
    .await
    .into_anyhow()?;

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
