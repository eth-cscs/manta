
use manta_backend_dispatcher::{
  error::Error,
  interfaces::migrate_restore::MigrateRestoreTrait,
};

use StaticBackendDispatcher::*;


use crate::manta_backend_dispatcher::StaticBackendDispatcher;

impl MigrateRestoreTrait for StaticBackendDispatcher {
  async fn migrate_restore(
    &self,
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    bos_file: Option<&str>,
    cfs_file: Option<&str>,
    hsm_file: Option<&str>,
    ims_file: Option<&str>,
    image_dir: Option<&str>,
    overwrite_group: bool,
    overwrite_configuration: bool,
    overwrite_image: bool,
    overwrite_template: bool,
  ) -> Result<(), Error> {
    match self {
      CSM(b) => {
        b.migrate_restore(
          shasta_token,
          shasta_base_url,
          shasta_root_cert,
          bos_file,
          cfs_file,
          hsm_file,
          ims_file,
          image_dir,
          overwrite_group,
          overwrite_configuration,
          overwrite_image,
          overwrite_template,
        )
        .await
      }
      OCHAMI(b) => {
        b.migrate_restore(
          shasta_token,
          shasta_base_url,
          shasta_root_cert,
          bos_file,
          cfs_file,
          hsm_file,
          ims_file,
          image_dir,
          overwrite_group,
          overwrite_configuration,
          overwrite_image,
          overwrite_template,
        )
        .await
      }
    }
  }
}
