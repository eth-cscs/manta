//! Dispatches `MigrateBackupTrait` methods to csm-rs or ochami-rs.

use manta_backend_dispatcher::{
  error::Error, interfaces::migrate_backup::MigrateBackupTrait,
};

use StaticBackendDispatcher::*;

use crate::manta_backend_dispatcher::StaticBackendDispatcher;

impl MigrateBackupTrait for StaticBackendDispatcher {
  async fn migrate_backup(
    &self,
    shasta_token: &str,
    bos: Option<&str>,
    destination: Option<&str>,
  ) -> Result<(), Error> {
    dispatch!(self, migrate_backup, shasta_token, bos, destination)
  }
}
