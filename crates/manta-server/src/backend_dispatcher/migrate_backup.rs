//! [`MigrateBackupTrait`] impl for [`StaticBackendDispatcher`].
//!
//! Forwards to the backend's "dump live state to disk" helper —
//! pulls a BOS session template and the IMS image / CFS configuration
//! it references, writing them under `destination` in a layout that
//! [`crate::backend_dispatcher::migrate_restore`] can read back.
//! Ochami uses the trait default and returns [`Error::Message`].

use super::*;

impl MigrateBackupTrait for StaticBackendDispatcher {
  /// Dump the state derived from BOS session template `bos` (when
  /// `Some`, otherwise every visible template) into `destination`.
  /// `destination = None` resolves to the backend's default output
  /// directory.
  async fn migrate_backup(
    &self,
    token: &str,
    bos: Option<&str>,
    destination: Option<&str>,
  ) -> Result<(), Error> {
    dispatch!(self, migrate_backup, token, bos, destination)
  }
}
