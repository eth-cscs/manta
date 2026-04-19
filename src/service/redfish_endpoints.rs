use anyhow::Error;
use manta_backend_dispatcher::interfaces::hsm::redfish_endpoint::RedfishEndpointTrait;
use manta_backend_dispatcher::types::hsm::inventory::RedfishEndpointArray;

use crate::manta_backend_dispatcher::StaticBackendDispatcher;

/// Typed parameters for fetching Redfish endpoints.
pub struct GetRedfishEndpointsParams {
  pub id: Option<String>,
  pub fqdn: Option<String>,
  pub uuid: Option<String>,
  pub macaddr: Option<String>,
  pub ipaddress: Option<String>,
}

/// Fetch Redfish endpoint registrations from the backend.
pub async fn get_redfish_endpoints(
  backend: &StaticBackendDispatcher,
  token: &str,
  params: &GetRedfishEndpointsParams,
) -> Result<RedfishEndpointArray, Error> {
  log::info!("Get Redfish endpoints");

  let result = backend
    .get_redfish_endpoints(
      token,
      params.id.as_deref(),
      params.fqdn.as_deref(),
      None,
      params.uuid.as_deref(),
      params.macaddr.as_deref(),
      params.ipaddress.as_deref(),
      None,
    )
    .await?;

  Ok(result)
}
