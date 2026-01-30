use anyhow::Error;
use clap::ArgMatches;
use manta_backend_dispatcher::interfaces::hsm::redfish_endpoint::RedfishEndpointTrait;

use crate::{common::authentication::get_api_token, manta_backend_dispatcher::StaticBackendDispatcher};

pub async fn exec(
  backend: &StaticBackendDispatcher,
  site_name: &str,
  arg_matches: &ArgMatches,
) -> Result<(), Error> {
  let shasta_token = get_api_token(backend, site_name).await?;

  let id = arg_matches.get_one::<String>("id").map(|x| x.as_str());
  let fqdn = arg_matches.get_one::<String>("fqdn").map(|x| x.as_str());
  let uuid = arg_matches.get_one::<String>("uuid").map(|x| x.as_str());
  let macaddr = arg_matches
    .get_one::<String>("macaddr")
    .map(|x| x.as_str());
  let ipaddress = arg_matches
    .get_one::<String>("ipaddress")
    .map(|x| x.as_str());

  let redfish_endpoints = backend
    .get_redfish_endpoints(
      &shasta_token,
      id,
      fqdn,
      None,
      uuid,
      macaddr,
      ipaddress,
      None,
    )
    .await?;

  println!("{}", serde_json::to_string_pretty(&redfish_endpoints)?);

  Ok(())
}
