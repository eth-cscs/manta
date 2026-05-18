//! Hardware-cluster and hardware-nodes-list read endpoints.

use serde_json::Value;

use manta_shared::shared::params::hardware::{
  GetHardwareClusterParams, GetHardwareNodesListParams,
};

use super::{MantaClient, QueryBuilder};

impl MantaClient {
  pub async fn get_hardware_clusters(
    &self,
    token: &str,
    params: &GetHardwareClusterParams,
  ) -> anyhow::Result<Value> {
    let hsm = params
      .hsm_group_name
      .as_deref()
      .or(params.settings_hsm_group_name.as_deref())
      .map(String::from);
    let q = QueryBuilder::new().opt("hsm_group", &hsm).build();
    self.get_json(token, "/hardware-clusters", &q).await
  }

  pub async fn get_hardware_nodes_list(
    &self,
    token: &str,
    params: &GetHardwareNodesListParams,
  ) -> anyhow::Result<Value> {
    let q: Vec<(&str, String)> = vec![("xnames", params.xnames.clone())];
    self.get_json(token, "/hardware-nodes-list", &q).await
  }
}
