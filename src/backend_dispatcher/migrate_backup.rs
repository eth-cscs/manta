use manta_backend_dispatcher::{
  error::Error, interfaces::migrate_backup::MigrateBackupTrait,
};

use StaticBackendDispatcher::*;

use crate::manta_backend_dispatcher::StaticBackendDispatcher;

impl MigrateBackupTrait for StaticBackendDispatcher {
  async fn migrate_backup(
    &self,
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    bos: Option<&str>,
    destination: Option<&str>,
  ) -> Result<(), Error> {
    match self {
      CSM(b) => {
        b.migrate_backup(
          shasta_token,
          shasta_base_url,
          shasta_root_cert,
          bos,
          destination,
        )
        .await
      }
      OCHAMI(b) => {
        b.migrate_backup(
          shasta_token,
          shasta_base_url,
          shasta_root_cert,
          bos,
          destination,
        )
        .await
      }
    }
  }
}
