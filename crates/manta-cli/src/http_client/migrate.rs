//! `/migrate/nodes` endpoint — move xnames between HSM groups.
//! vCluster backup/restore lives in [`super::vcluster`].

use serde_json::Value;

use super::MantaClient;

impl MantaClient {
  pub async fn migrate_nodes(
    &self,
    token: &str,
    target_hsm_names: &[String],
    parent_hsm_names: &[String],
    hosts_expression: &str,
    dry_run: bool,
    create_hsm_group: bool,
  ) -> anyhow::Result<Value> {
    let body = serde_json::json!({
      "target_hsm_names": target_hsm_names,
      "parent_hsm_names": parent_hsm_names,
      "hosts_expression": hosts_expression,
      "dry_run": dry_run,
      "create_hsm_group": create_hsm_group,
    });
    self.post_json(token, "/migrate/nodes", &body).await
  }
}
