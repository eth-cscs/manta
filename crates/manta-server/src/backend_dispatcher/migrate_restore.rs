//! `MigrateRestoreTrait` impl for `StaticBackendDispatcher`.

use super::*;

impl MigrateRestoreTrait for StaticBackendDispatcher {
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
