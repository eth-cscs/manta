//! [`MigrateRestoreTrait`] impl for [`StaticBackendDispatcher`].
//!
//! Inverse of
//! [`crate::backend_dispatcher::migrate_backup`]: reads the files
//! written by a previous backup and POSTs the contained HSM group,
//! CFS configuration, IMS image, and BOS session template back to
//! the backend. Each `overwrite_*` flag controls whether the
//! corresponding artifact is replaced if it already exists; without
//! the flag the backend returns
//! [`Error::ConfigurationAlreadyExistsError`] / [`Error::Conflict`].
//! Ochami uses the trait default and returns [`Error::Message`].

use super::*;

impl MigrateRestoreTrait for StaticBackendDispatcher {
  /// Re-create artifacts from the supplied dump files. Each `*_file`
  /// is optional so callers can restore a subset (e.g. just IMS
  /// images). `image_dir` points at the on-disk IMS payload
  /// directory backed up alongside `ims_file`.
  async fn migrate_restore(
    &self,
    token: &str,
    bos_file: Option<&str>,
    cfs_file: Option<&str>,
    group_file: Option<&str>,
    ims_file: Option<&str>,
    image_dir: Option<&str>,
    overwrite_group: bool,
    overwrite_configuration: bool,
    overwrite_image: bool,
    overwrite_template: bool,
  ) -> Result<(), Error> {
    dispatch!(
      self,
      migrate_restore,
      token,
      bos_file,
      cfs_file,
      group_file,
      ims_file,
      image_dir,
      overwrite_group,
      overwrite_configuration,
      overwrite_image,
      overwrite_template
    )
  }
}
