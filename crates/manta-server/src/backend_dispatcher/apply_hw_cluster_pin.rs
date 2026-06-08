//! `ApplyHwClusterPin` impl for `StaticBackendDispatcher`.

use super::*;

impl ApplyHwClusterPin for StaticBackendDispatcher {
  async fn apply_hw_cluster_pin(
    &self,
    token: &str,
    target_group_name: &str,
    parent_group_name: &str,
    pattern: &str,
    nodryrun: bool,
    create_target_group: bool,
    delete_empty_parent_group: bool,
  ) -> Result<(), Error> {
    dispatch!(
      self,
      apply_hw_cluster_pin,
      token,
      target_group_name,
      parent_group_name,
      pattern,
      nodryrun,
      create_target_group,
      delete_empty_parent_group
    )
  }
}
