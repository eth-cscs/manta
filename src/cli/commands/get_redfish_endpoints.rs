use anyhow::Error;

use crate::common::app_context::AppContext;
use crate::service::redfish_endpoints::{self, GetRedfishEndpointsParams};

/// Parse CLI arguments into typed [`GetRedfishEndpointsParams`].
fn parse_redfish_endpoints_params(cli_args: &clap::ArgMatches) -> GetRedfishEndpointsParams {
  GetRedfishEndpointsParams {
    id: cli_args.get_one::<String>("id").cloned(),
    fqdn: cli_args.get_one::<String>("fqdn").cloned(),
    uuid: cli_args.get_one::<String>("uuid").cloned(),
    macaddr: cli_args.get_one::<String>("macaddr").cloned(),
    ipaddress: cli_args.get_one::<String>("ipaddress").cloned(),
  }
}

/// CLI adapter for `manta get redfish-endpoints`.
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  cli_args: &clap::ArgMatches,
) -> Result<(), Error> {
  let params = parse_redfish_endpoints_params(cli_args);

  let endpoints =
    redfish_endpoints::get_redfish_endpoints(&ctx.infra, token, &params).await?;

  println!("{}", serde_json::to_string_pretty(&endpoints)?);

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  use clap::arg;

  fn redfish_cmd() -> clap::Command {
    clap::Command::new("redfish-endpoints")
      .arg(arg!(--id <ID> "endpoint id"))
      .arg(arg!(--fqdn <FQDN> "fqdn"))
      .arg(arg!(--uuid <UUID> "uuid"))
      .arg(arg!(--macaddr <MACADDR> "mac address"))
      .arg(arg!(--ipaddress <IPADDRESS> "ip address"))
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
      "--id", "x3000c0s1b0",
      "--fqdn", "node1.example.com",
      "--uuid", "abc-123",
      "--macaddr", "00:11:22:33:44:55",
      "--ipaddress", "10.0.0.1",
    ]);
    let params = parse_redfish_endpoints_params(&matches);
    assert_eq!(params.id.as_deref(), Some("x3000c0s1b0"));
    assert_eq!(params.fqdn.as_deref(), Some("node1.example.com"));
    assert_eq!(params.uuid.as_deref(), Some("abc-123"));
    assert_eq!(params.macaddr.as_deref(), Some("00:11:22:33:44:55"));
    assert_eq!(params.ipaddress.as_deref(), Some("10.0.0.1"));
  }
}
