//! Redfish endpoint registration methods on `InfraContext`.

use manta_backend_dispatcher::error::Error;
use manta_backend_dispatcher::interfaces::hsm::redfish_endpoint::RedfishEndpointTrait;
use manta_backend_dispatcher::types::hsm::inventory::{
  RedfishEndpoint, RedfishEndpointArray,
};
use manta_shared::types::params::redfish_endpoints::{
  GetRedfishEndpointsParams, UpdateRedfishEndpointParams,
};

use crate::server::common::app_context::InfraContext;

impl InfraContext<'_> {
  /// Fetch Redfish endpoint registrations matching the filters.
  pub async fn get_redfish_endpoints(
    &self,
    token: &str,
    params: &GetRedfishEndpointsParams,
  ) -> Result<RedfishEndpointArray, Error> {
    self
      .backend
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
      .await
  }

  /// Delete a Redfish endpoint registration by id.
  pub async fn delete_redfish_endpoint(
    &self,
    token: &str,
    id: &str,
  ) -> Result<(), Error> {
    self
      .backend
      .delete_redfish_endpoint(token, id)
      .await
      .map(|_| ())
  }

  /// Register a new Redfish endpoint.
  pub async fn add_redfish_endpoint(
    &self,
    token: &str,
    params: UpdateRedfishEndpointParams,
  ) -> Result<(), Error> {
    let array = RedfishEndpointArray {
      redfish_endpoints: Some(vec![params_to_redfish_endpoint(params)]),
    };
    self.backend.add_redfish_endpoint(token, &array).await
  }

  /// Update an existing Redfish endpoint's properties.
  pub async fn update_redfish_endpoint(
    &self,
    token: &str,
    params: UpdateRedfishEndpointParams,
  ) -> Result<(), Error> {
    let endpoint = params_to_redfish_endpoint(params);
    self.backend.update_redfish_endpoint(token, &endpoint).await
  }
}

fn params_to_redfish_endpoint(
  params: UpdateRedfishEndpointParams,
) -> RedfishEndpoint {
  RedfishEndpoint {
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
  }
}
