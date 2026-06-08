//! `/migrate/backup` and `/migrate/restore` endpoints — vCluster
//! backup/restore. The endpoint paths still live under `/migrate/`
//! for compatibility, but the canonical CLI commands are
//! `manta backup vcluster` and `manta restore vcluster`.

use serde_json::Value;

pub use manta_shared::types::wire::migrate::{
  MigrateBackupRequest, MigrateRestoreRequest,
};

use super::MantaClient;

impl MantaClient {
  pub async fn backup_vcluster(
    &self,
    token: &str,
    bos: Option<&str>,
    destination: Option<&str>,
  ) -> anyhow::Result<()> {
    let body = MigrateBackupRequest {
      bos: bos.map(str::to_string),
      destination: destination.map(str::to_string),
    };
    let _: Value = self.post_json(token, "/migrate/backup", &body).await?;
    Ok(())
  }

  pub async fn restore_vcluster(
    &self,
    token: &str,
    req: &MigrateRestoreRequest,
  ) -> anyhow::Result<()> {
    let _: Value = self.post_json(token, "/migrate/restore", req).await?;
    Ok(())
  }
}
