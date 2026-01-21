
use manta_backend_dispatcher::{
  error::Error,
  interfaces::apply_hw_cluster_pin::ApplyHwClusterPin,
};

use StaticBackendDispatcher::*;


use crate::manta_backend_dispatcher::StaticBackendDispatcher;

impl ApplyHwClusterPin for StaticBackendDispatcher {
  async fn apply_hw_cluster_pin(
    &self,
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    target_hsm_group_name: &str,
    parent_hsm_group_name: &str,
    pattern: &str,
    nodryrun: bool,
    create_target_hsm_group: bool,
    delete_empty_parent_hsm_group: bool,
  ) -> Result<(), Error> {
    match self {
      CSM(b) => {
        b.apply_hw_cluster_pin(
          shasta_token,
          shasta_base_url,
          shasta_root_cert,
          target_hsm_group_name,
          parent_hsm_group_name,
          pattern,
          nodryrun,
          create_target_hsm_group,
          delete_empty_parent_hsm_group,
        )
        .await
      }
      OCHAMI(b) => {
        b.apply_hw_cluster_pin(
          shasta_token,
          shasta_base_url,
          shasta_root_cert,
          target_hsm_group_name,
          parent_hsm_group_name,
          pattern,
          nodryrun,
          create_target_hsm_group,
          delete_empty_parent_hsm_group,
        )
        .await
      }
    }
  }
}
