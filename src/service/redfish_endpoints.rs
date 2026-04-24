use anyhow::{Context, Error};
use manta_backend_dispatcher::interfaces::hsm::redfish_endpoint::RedfishEndpointTrait;
use manta_backend_dispatcher::types::hsm::inventory::{
  RedfishEndpoint, RedfishEndpointArray,
};

use crate::common::app_context::InfraContext;

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
  infra: &InfraContext<'_>,
  token: &str,
  params: &GetRedfishEndpointsParams,
) -> Result<RedfishEndpointArray, Error> {
  tracing::info!("Get Redfish endpoints");

  let result = infra.backend
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

/// Delete a Redfish endpoint registration by ID.
pub async fn delete_redfish_endpoint(
  infra: &InfraContext<'_>,
  token: &str,
  id: &str,
) -> Result<(), Error> {
  infra
    .backend
    .delete_redfish_endpoint(token, id)
    .await
    .with_context(|| {
      format!("Failed to delete redfish endpoint for id '{}'", id)
    })?;
  Ok(())
}

/// Typed parameters for updating/adding a Redfish endpoint.
#[derive(serde::Deserialize)]
pub struct UpdateRedfishEndpointParams {
  pub id: String,
  pub name: Option<String>,
  pub hostname: Option<String>,
  pub domain: Option<String>,
  pub fqdn: Option<String>,
  pub enabled: bool,
  pub user: Option<String>,
  pub password: Option<String>,
  pub use_ssdp: bool,
  pub mac_required: bool,
  pub mac_addr: Option<String>,
  pub ip_address: Option<String>,
  pub rediscover_on_update: bool,
  pub template_id: Option<String>,
}

/// Add (register) a new Redfish endpoint.
pub async fn add_redfish_endpoint(
  infra: &InfraContext<'_>,
  token: &str,
  params: UpdateRedfishEndpointParams,
) -> Result<(), Error> {
  let redfish_endpoint = RedfishEndpoint {
    id: params.id,
    name: params.name,
    hostname: params.hostname,
    domain: params.domain,
    fqdn: params.fqdn,
    enabled: Some(params.enabled),
    user: params.user,
    password: params.password,
    use_ssdp: Some(params.use_ssdp),
    mac_required: Some(params.mac_required),
    mac_addr: params.mac_addr,
    ip_address: params.ip_address,
    rediscover_on_update: Some(params.rediscover_on_update),
    template_id: params.template_id,
    r#type: None,
    uuid: None,
    discovery_info: None,
  };

  let redfish_endpoint_array = RedfishEndpointArray {
    redfish_endpoints: Some(vec![redfish_endpoint]),
  };

  infra
    .backend
    .add_redfish_endpoint(token, &redfish_endpoint_array)
    .await?;

  Ok(())
}


/// Update a Redfish endpoint registration.
pub async fn update_redfish_endpoint(
  infra: &InfraContext<'_>,
  token: &str,
  params: UpdateRedfishEndpointParams,
) -> Result<(), Error> {
  let redfish_endpoint = RedfishEndpoint {
    id: params.id,
    name: params.name,
    hostname: params.hostname,
    domain: params.domain,
    fqdn: params.fqdn,
    enabled: Some(params.enabled),
    user: params.user,
    password: params.password,
    use_ssdp: Some(params.use_ssdp),
    mac_required: Some(params.mac_required),
    mac_addr: params.mac_addr,
    ip_address: params.ip_address,
    rediscover_on_update: Some(params.rediscover_on_update),
    template_id: params.template_id,
    r#type: None,
    uuid: None,
    discovery_info: None,
  };

  infra
    .backend
    .update_redfish_endpoint(token, &redfish_endpoint)
    .await?;

  Ok(())
}
