//! vCluster migrate backup/restore methods on `InfraContext`.

use manta_backend_dispatcher::error::Error;
use manta_backend_dispatcher::interfaces::migrate_backup::MigrateBackupTrait;
use manta_backend_dispatcher::interfaces::migrate_restore::MigrateRestoreTrait;

use crate::server::common::app_context::InfraContext;

impl InfraContext<'_> {
  /// Backup a vCluster (CFS / IMS / BSS / BOS / HSM artefacts).
  pub async fn migrate_backup(
    &self,
    token: &str,
    bos: Option<&str>,
    destination: Option<&str>,
  ) -> Result<(), Error> {
    self.backend.migrate_backup(token, bos, destination).await
  }

  /// Restore a vCluster from backup files.
  ///
  /// The backend trait exposes four independent overwrite flags
  /// (group/configuration/image/template). The HTTP/CLI APIs expose a
  /// single `overwrite` knob that fans out here. Expose them
  /// individually if callers need per-resource control in the future.
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
  ) -> Result<(), Error> {
    self
      .backend
      .migrate_restore(
        token, bos_file, cfs_file, hsm_file, ims_file, image_dir, overwrite,
        overwrite, overwrite, overwrite,
      )
      .await
  }
}
