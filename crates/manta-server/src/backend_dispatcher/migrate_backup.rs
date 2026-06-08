//! `MigrateBackupTrait` impl for `StaticBackendDispatcher`.

use super::*;

impl MigrateBackupTrait for StaticBackendDispatcher {
  async fn migrate_backup(
    &self,
    token: &str,
    bos: Option<&str>,
    destination: Option<&str>,
  ) -> Result<(), Error> {
    dispatch!(self, migrate_backup, token, bos, destination)
  }
}
