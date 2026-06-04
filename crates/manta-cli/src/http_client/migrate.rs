//! `manta migrate` endpoints: nodes between HSM groups, vCluster backup/restore.

use serde::Serialize;
use serde_json::Value;

use super::MantaClient;

/// Request body for `POST /migrate/restore`.
#[derive(Serialize)]
pub struct MigrateRestoreRequest<'a> {
  pub bos_file: Option<&'a str>,
  pub cfs_file: Option<&'a str>,
  pub hsm_file: Option<&'a str>,
  pub ims_file: Option<&'a str>,
  pub image_dir: Option<&'a str>,
  pub overwrite: bool,
}

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

  pub async fn migrate_backup(
    &self,
    token: &str,
    bos: Option<&str>,
    destination: Option<&str>,
  ) -> anyhow::Result<()> {
    let body = serde_json::json!({ "bos": bos, "destination": destination });
    let _: Value = self.post_json(token, "/migrate/backup", &body).await?;
    Ok(())
  }

  pub async fn migrate_restore(
    &self,
    token: &str,
    req: &MigrateRestoreRequest<'_>,
  ) -> anyhow::Result<()> {
    let _: Value = self.post_json(token, "/migrate/restore", req).await?;
    Ok(())
  }
}
