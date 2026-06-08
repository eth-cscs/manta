//! Node CRUD endpoints (console lives in `console.rs`).

use serde_json::Value;

use manta_shared::types::dto::NodeDetails;
use manta_shared::types::params::node::GetNodesParams;

use super::{MantaClient, QueryBuilder};

impl MantaClient {
  pub async fn get_nodes(
    &self,
    token: &str,
    params: &GetNodesParams,
  ) -> anyhow::Result<Vec<NodeDetails>> {
    let q = QueryBuilder::new()
      .pair("xname", params.host_expression.clone())
      .flag("include_siblings", params.include_siblings)
      .opt("status", &params.status_filter)
      .build();
    self.get_json(token, "/nodes", &q).await
  }

  pub async fn add_node(
    &self,
    token: &str,
    id: &str,
    group: &str,
    enabled: bool,
    arch: Option<String>,
  ) -> anyhow::Result<()> {
    let body = serde_json::json!({ "id": id, "group": group, "enabled": enabled, "arch": arch });
    let _: Value = self.post_json(token, "/nodes", &body).await?;
    Ok(())
  }

  pub async fn delete_node(&self, token: &str, id: &str) -> anyhow::Result<()> {
    self.delete_no_content(token, &format!("/nodes/{id}")).await
  }
}
