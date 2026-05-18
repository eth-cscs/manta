//! Cluster-scoped node detail queries.

use manta_shared::shared::dto::NodeDetails;
use manta_shared::shared::params::cluster::GetClusterParams;

use super::{MantaClient, QueryBuilder};

impl MantaClient {
  pub async fn get_clusters(
    &self,
    token: &str,
    params: &GetClusterParams,
  ) -> anyhow::Result<Vec<NodeDetails>> {
    let hsm = params
      .hsm_group_name
      .as_deref()
      .or(params.settings_hsm_group_name.as_deref())
      .map(String::from);
    let q = QueryBuilder::new()
      .opt("hsm_group", &hsm)
      .opt("status", &params.status_filter)
      .build();
    self.get_json(token, "/clusters", &q).await
  }
}
