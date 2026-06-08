//! `ApplySessionTrait` impl for `StaticBackendDispatcher`.

use super::*;

impl ApplySessionTrait for StaticBackendDispatcher {
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
