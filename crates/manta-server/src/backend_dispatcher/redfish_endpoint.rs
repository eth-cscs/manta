//! [`RedfishEndpointTrait`] impl for [`StaticBackendDispatcher`].
//!
//! Forwards to HSM's
//! `/apis/smd/hsm/v2/Inventory/RedfishEndpoints` API. Both CSM and
//! Ochami implement this trait natively.

use super::*;

impl RedfishEndpointTrait for StaticBackendDispatcher {
  /// `GET /RedfishEndpoints` — every registered BMC endpoint.
  async fn get_all_redfish_endpoints(
    &self,
    auth_token: &str,
  ) -> Result<RedfishEndpointArray, Error> {
    dispatch!(self, get_all_redfish_endpoints, auth_token)
  }

  /// `GET /RedfishEndpoints` with the full HSM filter set
  /// (`id`, `fqdn`, `type`, `uuid`, `macaddr`, `ip_address`,
  /// `last_status`).
  async fn get_redfish_endpoints(
    &self,
    auth_token: &str,
    id: Option<&str>,
    fqdn: Option<&str>,
    r#type: Option<&str>,
    uuid: Option<&str>,
    macaddr: Option<&str>,
    ip_address: Option<&str>,
    last_status: Option<&str>,
  ) -> Result<RedfishEndpointArray, Error> {
    dispatch!(
      self,
      get_redfish_endpoints,
      auth_token,
      id,
      fqdn,
      r#type,
      uuid,
      macaddr,
      ip_address,
      last_status
    )
  }

  /// `POST /RedfishEndpoints` — bulk-register endpoints.
  async fn add_redfish_endpoint(
    &self,
    auth_token: &str,
    redfish_endpoint: &RedfishEndpointArray,
  ) -> Result<(), Error> {
    dispatch!(self, add_redfish_endpoint, auth_token, redfish_endpoint)
  }

  /// `PUT /RedfishEndpoints/{id}` — replace a single endpoint's
  /// record.
  async fn update_redfish_endpoint(
    &self,
    auth_token: &str,
    redfish_endpoint: &RedfishEndpoint,
  ) -> Result<(), Error> {
    dispatch!(self, update_redfish_endpoint, auth_token, redfish_endpoint)
  }

  /// `DELETE /RedfishEndpoints/{id}`. Returns HSM's raw JSON
  /// action summary.
  async fn delete_redfish_endpoint(
    &self,
    auth_token: &str,
    id: &str,
  ) -> Result<Value, Error> {
    dispatch!(self, delete_redfish_endpoint, auth_token, id)
  }
}
