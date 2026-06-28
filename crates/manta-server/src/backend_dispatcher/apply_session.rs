//! [`ApplySessionTrait`] impl for [`StaticBackendDispatcher`].
//!
//! Forwards to the backend's "apply ad-hoc CFS session" helper, which
//! builds a `CfsConfigurationRequest` from the supplied Gitea repos +
//! commit ids, POSTs it to `cfs/v3/configurations`, then POSTs a
//! `CfsSessionPostRequest` to `cfs/v3/sessions` that runs the
//! playbook with the supplied Ansible options.
//!
//! Ochami uses the trait default and returns [`Error::Message`]
//! ("Apply session command not implemented for this backend").

use super::*;

impl ApplySessionTrait for StaticBackendDispatcher {
  /// Build a CFS configuration from `repos_name_vec` /
  /// `repos_last_commit_id_vec` and run an Ansible session against it.
  ///
  /// Returns `(cfs_configuration_name, cfs_session_name)` — the names
  /// of the artifacts created on the backend.
  ///
  /// # Errors
  ///
  /// Forwards backend errors verbatim: [`Error::CsmError`] on CFS
  /// rejection (e.g. duplicate configuration name when the helper
  /// retries), [`Error::LocalGitError`] when Gitea metadata lookup
  /// fails, [`Error::Message`] on Ochami.
  async fn apply_session(
    &self,
    gitea_token: &str,
    gitea_base_url: &str,
    token: &str,
    cfs_conf_sess_name: Option<&str>,
    playbook_yaml_file_name_opt: Option<&str>,
    group_name: Option<&str>,
    repos_name_vec: &[&str],
    repos_last_commit_id_vec: &[&str],
    ansible_limit: Option<&str>,
    ansible_verbosity: Option<&str>,
    ansible_passthrough: Option<&str>,
  ) -> Result<(String, String), Error> {
    dispatch!(
      self,
      apply_session,
      gitea_token,
      gitea_base_url,
      token,
      cfs_conf_sess_name,
      playbook_yaml_file_name_opt,
      group_name,
      repos_name_vec,
      repos_last_commit_id_vec,
      ansible_limit,
      ansible_verbosity,
      ansible_passthrough
    )
  }
}
