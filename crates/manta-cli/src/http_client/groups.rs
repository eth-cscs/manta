//! HSM group endpoints: list, available/all, create, add/remove
//! members, delete, plus `/groups/nodes` for group-scoped node detail
//! queries.

use serde_json::Value;

use manta_shared::types::dto::{Group, NodeDetails};
use manta_shared::types::params::cluster::GetClusterParams;
use manta_shared::types::params::group::GetGroupParams;

use super::{MantaClient, QueryBuilder};

impl MantaClient {
  pub async fn get_groups(
    &self,
    token: &str,
    params: &GetGroupParams,
  ) -> anyhow::Result<Vec<Group>> {
    let q = QueryBuilder::new().opt("name", &params.group_name).build();
    self.get_json(token, "/groups", &q).await
  }

  /// `GET /api/v1/groups/nodes` — list node details scoped to an HSM
  /// group (with optional status filter). Backs `manta get group-nodes`.
  pub async fn get_group_nodes(
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
    self.get_json(token, "/groups/nodes", &q).await
  }

  /// `GET /api/v1/groups/available` — list HSM group names the token can
  /// access. Replaces the CLI's direct `backend.get_group_name_available`
  /// calls.
  pub async fn get_available_groups(
    &self,
    token: &str,
  ) -> anyhow::Result<Vec<String>> {
    self.get_json(token, "/groups/available", &[]).await
  }

  /// `GET /api/v1/groups/all` — list every HSM group in the system,
  /// regardless of access. Used by CLI commands that need the full
  /// catalogue (e.g. setting a default HSM group at config time).
  pub async fn get_all_groups(
    &self,
    token: &str,
  ) -> anyhow::Result<Vec<Group>> {
    self.get_json(token, "/groups/all", &[]).await
  }

  pub async fn create_group(
    &self,
    token: &str,
    group: Group,
  ) -> anyhow::Result<()> {
    let _: Value = self.post_json(token, "/groups", &group).await?;
    Ok(())
  }

  pub async fn add_nodes_to_group(
    &self,
    token: &str,
    name: &str,
    hosts_expression: &str,
  ) -> anyhow::Result<(Vec<String>, Vec<String>)> {
    let body = serde_json::json!({ "hosts_expression": hosts_expression });
    let resp: Value = self
      .post_json(token, &format!("/groups/{name}/members"), &body)
      .await?;
    let added = resp["added"]
      .as_array()
      .map(|a| {
        a.iter()
          .filter_map(|v| v.as_str().map(String::from))
          .collect()
      })
      .unwrap_or_default();
    let removed = resp["removed"]
      .as_array()
      .map(|a| {
        a.iter()
          .filter_map(|v| v.as_str().map(String::from))
          .collect()
      })
      .unwrap_or_default();
    Ok((added, removed))
  }

  pub async fn delete_group(
    &self,
    token: &str,
    label: &str,
    force: bool,
  ) -> anyhow::Result<()> {
    let q = [("force", force.to_string())];
    self
      .delete_no_content_with_query(token, &format!("/groups/{label}"), &q)
      .await
  }

  pub async fn delete_group_members(
    &self,
    token: &str,
    name: &str,
    xnames_expression: &str,
    dry_run: bool,
  ) -> anyhow::Result<()> {
    let body = serde_json::json!({ "xnames_expression": xnames_expression, "dry_run": dry_run });
    self
      .delete_no_content_with_body(
        token,
        &format!("/groups/{name}/members"),
        &body,
      )
      .await
  }
}
