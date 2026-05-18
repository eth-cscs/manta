//! `manta migrate` endpoints: nodes between HSM groups, vCluster backup/restore.

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

  #[allow(clippy::too_many_arguments)]
  pub async fn migrate_restore(
    &self,
    token: &str,
    bos_file: Option<&str>,
    cfs_file: Option<&str>,
    hsm_file: Option<&str>,
    ims_file: Option<&str>,
    image_dir: Option<&str>,
    overwrite: bool,
  ) -> anyhow::Result<()> {
    let body = serde_json::json!({
      "bos_file": bos_file,
      "cfs_file": cfs_file,
      "hsm_file": hsm_file,
      "ims_file": ims_file,
      "image_dir": image_dir,
      "overwrite": overwrite,
    });
    let _: Value = self.post_json(token, "/migrate/restore", &body).await?;
    Ok(())
  }
}
