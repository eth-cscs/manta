//! `/migrate/backup` and `/migrate/restore` endpoints — vCluster
//! backup/restore. The endpoint paths still live under `/migrate/`
//! for compatibility, but the canonical CLI commands are
//! `manta backup vcluster` and `manta restore vcluster`.

use serde::Serialize;
use serde_json::Value;

use super::MantaClient;

/// Request body for `POST /migrate/restore`.
#[derive(Serialize)]
pub struct RestoreVclusterRequest<'a> {
  pub bos_file: Option<&'a str>,
  pub cfs_file: Option<&'a str>,
  pub hsm_file: Option<&'a str>,
  pub ims_file: Option<&'a str>,
  pub image_dir: Option<&'a str>,
  pub overwrite: bool,
}

impl MantaClient {
  pub async fn backup_vcluster(
    &self,
    token: &str,
    bos: Option<&str>,
    destination: Option<&str>,
  ) -> anyhow::Result<()> {
    let body = serde_json::json!({ "bos": bos, "destination": destination });
    let _: Value = self.post_json(token, "/migrate/backup", &body).await?;
    Ok(())
  }

  pub async fn restore_vcluster(
    &self,
    token: &str,
    req: &RestoreVclusterRequest<'_>,
  ) -> anyhow::Result<()> {
    let _: Value = self.post_json(token, "/migrate/restore", req).await?;
    Ok(())
  }
}
