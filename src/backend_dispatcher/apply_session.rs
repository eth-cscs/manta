use manta_backend_dispatcher::{
  error::Error, interfaces::apply_session::ApplySessionTrait,
};

use StaticBackendDispatcher::*;

use crate::manta_backend_dispatcher::StaticBackendDispatcher;

impl ApplySessionTrait for StaticBackendDispatcher {
  async fn apply_session(
    &self,
    gitea_token: &str,
    gitea_base_url: &str,
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    cfs_conf_sess_name: Option<&str>,
    playbook_yaml_file_name_opt: Option<&str>,
    hsm_group: Option<&str>,
    repos_name_vec: &[&str],
    repos_last_commit_id_vec: &[&str],
    ansible_limit: Option<&str>,
    ansible_verbosity: Option<&str>,
    ansible_passthrough: Option<&str>,
  ) -> Result<(String, String), Error> {
    match self {
      CSM(b) => {
        b.apply_session(
          gitea_token,
          gitea_base_url,
          shasta_token,
          shasta_base_url,
          shasta_root_cert,
          cfs_conf_sess_name,
          playbook_yaml_file_name_opt,
          hsm_group,
          repos_name_vec,
          repos_last_commit_id_vec,
          ansible_limit,
          ansible_verbosity,
          ansible_passthrough,
        )
        .await
      }
      OCHAMI(b) => {
        b.apply_session(
          gitea_token,
          gitea_base_url,
          shasta_token,
          shasta_base_url,
          shasta_root_cert,
          cfs_conf_sess_name,
          playbook_yaml_file_name_opt,
          hsm_group,
          repos_name_vec,
          repos_last_commit_id_vec,
          ansible_limit,
          ansible_verbosity,
          ansible_passthrough,
        )
        .await
      }
    }
  }
}
