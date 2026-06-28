//! [`ApplyHwClusterPin`] impl for [`StaticBackendDispatcher`].
//!
//! Forwards to the CSM backend's hardware-cluster-pin orchestration
//! (HSM group create/delete + member migration driven by an xname
//! pattern). Ochami's `Ochami` type uses the trait's default body —
//! calls routed there return [`Error::Message`] with a
//! "not implemented for this backend" payload.

use super::*;

impl ApplyHwClusterPin for StaticBackendDispatcher {
  /// Pin nodes matching `pattern` from `parent_group_name` into
  /// `target_group_name`.
  ///
  /// Forwards to the backend's `apply_hw_cluster_pin`. When
  /// `nodryrun` is false the backend short-circuits before any HSM
  /// mutation. `create_target_group` controls whether the target is
  /// created when missing; `delete_empty_parent_group` removes the
  /// source after migration when it is empty.
  ///
  /// # Errors
  ///
  /// Returns [`Error::InvalidPattern`] when `pattern` does not match
  /// the backend's pattern grammar, [`Error::InsufficientResources`]
  /// when the pattern selects fewer nodes than required,
  /// [`Error::CsmError`] / [`Error::RequestError`] for HSM call
  /// failures, and the Ochami default
  /// [`Error::Message`] for the unsupported backend.
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
