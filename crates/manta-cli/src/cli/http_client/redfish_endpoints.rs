//! Redfish endpoint CRUD.

use serde_json::Value;

use manta_shared::shared::params::redfish_endpoints::{
  GetRedfishEndpointsParams, UpdateRedfishEndpointParams,
};

use super::{MantaClient, QueryBuilder};

impl MantaClient {
  pub async fn get_redfish_endpoints(
    &self,
    token: &str,
    params: &GetRedfishEndpointsParams,
  ) -> anyhow::Result<serde_json::Value> {
    let q = QueryBuilder::new()
      .opt("id", &params.id)
      .opt("fqdn", &params.fqdn)
      .opt("uuid", &params.uuid)
      .opt("macaddr", &params.macaddr)
      .opt("ipaddress", &params.ipaddress)
      .build();
    self.get_json(token, "/redfish-endpoints", &q).await
  }

  pub async fn add_redfish_endpoint(
    &self,
    token: &str,
    params: UpdateRedfishEndpointParams,
  ) -> anyhow::Result<()> {
    let _: Value = self.post_json(token, "/redfish-endpoints", &params).await?;
    Ok(())
  }

  pub async fn update_redfish_endpoint(
    &self,
    token: &str,
    params: &UpdateRedfishEndpointParams,
  ) -> anyhow::Result<()> {
    self
      .put_no_content(token, "/redfish-endpoints", params)
      .await
  }

  pub async fn delete_redfish_endpoint(
    &self,
    token: &str,
    id: &str,
  ) -> anyhow::Result<()> {
    self
      .delete_no_content(token, &format!("/redfish-endpoints/{}", id))
      .await
  }
}
