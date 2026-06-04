//! Hardware-cluster mutation endpoints: pin/unpin (apply_hw_configuration),
//! add/delete hw component.

use serde::Serialize;
use serde_json::Value;

use super::MantaClient;

/// Request body for `POST /hardware-clusters/{target}/configuration`.
#[derive(Serialize)]
pub struct ApplyHwConfigurationRequest<'a> {
  pub parent_cluster: &'a str,
  pub pattern: &'a str,
  pub mode: &'a str,
  pub dry_run: bool,
  pub create_target_hsm_group: bool,
  pub delete_empty_parent_hsm_group: bool,
}

impl MantaClient {
  pub async fn add_hw_component(
    &self,
    token: &str,
    target: &str,
    parent_cluster: &str,
    pattern: &str,
    create_hsm_group: bool,
    dry_run: bool,
  ) -> anyhow::Result<Value> {
    let body = serde_json::json!({
      "parent_cluster": parent_cluster,
      "pattern": pattern,
      "create_hsm_group": create_hsm_group,
      "dry_run": dry_run,
    });
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
    let body = serde_json::json!({
      "parent_cluster": parent_cluster,
      "pattern": pattern,
      "delete_hsm_group": delete_hsm_group,
      "dry_run": dry_run,
    });
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
    req: &ApplyHwConfigurationRequest<'_>,
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
