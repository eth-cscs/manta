use manta_backend_dispatcher::{
  error::Error,
  interfaces::hsm::redfish_endpoint::RedfishEndpointTrait,
  types::hsm::inventory::{RedfishEndpoint, RedfishEndpointArray},
};

use StaticBackendDispatcher::*;

use serde_json::Value;

use crate::manta_backend_dispatcher::StaticBackendDispatcher;

impl RedfishEndpointTrait for StaticBackendDispatcher {
  async fn get_all_redfish_endpoints(
    &self,
    auth_token: &str,
  ) -> Result<RedfishEndpointArray, Error> {
    match self {
      CSM(b) => b.get_all_redfish_endpoints(auth_token).await,
      OCHAMI(b) => b.get_all_redfish_endpoints(auth_token).await,
    }
  }

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
    match self {
      CSM(b) => {
        b.get_redfish_endpoints(
          auth_token,
          id,
          fqdn,
          r#type,
          uuid,
          macaddr,
          ip_address,
          last_status,
        )
        .await
      }
      OCHAMI(b) => {
        b.get_redfish_endpoints(
          auth_token,
          id,
          fqdn,
          r#type,
          uuid,
          macaddr,
          ip_address,
          last_status,
        )
        .await
      }
    }
  }

  async fn add_redfish_endpoint(
    &self,
    auth_token: &str,
    redfish_endpoint: &RedfishEndpointArray,
  ) -> Result<(), Error> {
    match self {
      CSM(b) => b.add_redfish_endpoint(auth_token, redfish_endpoint).await,
      OCHAMI(b) => b.add_redfish_endpoint(auth_token, redfish_endpoint).await,
    }
  }

  async fn update_redfish_endpoint(
    &self,
    auth_token: &str,
    redfish_endpoint: &RedfishEndpoint,
  ) -> Result<(), Error> {
    match self {
      CSM(b) => {
        b.update_redfish_endpoint(auth_token, redfish_endpoint)
          .await
      }
      OCHAMI(b) => {
        b.update_redfish_endpoint(auth_token, redfish_endpoint)
          .await
      }
    }
  }

  async fn delete_redfish_endpoint(
    &self,
    auth_token: &str,
    id: &str,
  ) -> Result<Value, Error> {
    match self {
      CSM(b) => b.delete_redfish_endpoint(auth_token, id).await,
      OCHAMI(b) => b.delete_redfish_endpoint(auth_token, id).await,
    }
  }
}
