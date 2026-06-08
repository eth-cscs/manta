//! Hardware-cluster endpoints: read (HSM-scoped inventory + explicit
//! xname inventory) + mutations (pin/unpin via `apply_hw_configuration`,
//! add/delete hw component).

use serde_json::Value;

use manta_shared::types::params::hardware::{
  GetHardwareClusterParams, GetHardwareNodesListParams,
};
pub use manta_shared::types::wire::hw_cluster::{
  AddHwComponentRequest, ApplyHwConfigurationRequest, DeleteHwComponentRequest,
};

use super::{MantaClient, QueryBuilder};

impl MantaClient {
  pub async fn get_hardware_clusters(
    &self,
    token: &str,
    params: &GetHardwareClusterParams,
  ) -> anyhow::Result<Value> {
    let hsm = params
      .group_name
      .as_deref()
      .or(params.settings_hsm_group_name.as_deref())
      .map(String::from);
    let q = QueryBuilder::new().opt("hsm_group", &hsm).build();
    self.get_json(token, "/groups/hardware", &q).await
  }

  pub async fn get_hardware_nodes_list(
    &self,
    token: &str,
    params: &GetHardwareNodesListParams,
  ) -> anyhow::Result<Value> {
    let q: Vec<(&str, String)> =
      vec![("xnames", params.host_expression.clone())];
    self.get_json(token, "/hardware-nodes-list", &q).await
  }

  pub async fn add_hw_component(
    &self,
    token: &str,
    target: &str,
    parent_cluster: &str,
    pattern: &str,
    create_hsm_group: bool,
    dry_run: bool,
  ) -> anyhow::Result<Value> {
    let body = AddHwComponentRequest {
      parent_cluster: parent_cluster.to_string(),
      pattern: pattern.to_string(),
      create_hsm_group,
      dry_run,
    };
    self
      .post_json(
        token,
        &format!("/hardware-clusters/{target}/members"),
        &body,
      )
      .await
  }

  pub async fn delete_hw_component(
    &self,
    token: &str,
    target: &str,
    parent_cluster: &str,
    pattern: &str,
    delete_hsm_group: bool,
    dry_run: bool,
  ) -> anyhow::Result<Value> {
    let body = DeleteHwComponentRequest {
      parent_cluster: parent_cluster.to_string(),
      pattern: pattern.to_string(),
      delete_hsm_group,
      dry_run,
    };
    self
      .delete_json_with_body(
        token,
        &format!("/hardware-clusters/{target}/members"),
        &body,
      )
      .await
  }

  pub async fn apply_hw_configuration(
    &self,
    token: &str,
    target: &str,
    req: &ApplyHwConfigurationRequest,
  ) -> anyhow::Result<Value> {
    self
      .post_json(
        token,
        &format!("/hardware-clusters/{target}/configuration"),
        req,
      )
      .await
  }
}
