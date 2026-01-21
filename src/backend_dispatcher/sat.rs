
use manta_backend_dispatcher::{
  error::Error,
  interfaces::apply_sat_file::SatTrait,
};

use StaticBackendDispatcher::*;


use crate::manta_backend_dispatcher::StaticBackendDispatcher;

impl SatTrait for StaticBackendDispatcher {
  async fn apply_sat_file(
    &self,
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    vault_base_url: &str,
    vault_secret_path: &str,
    k8s_api_url: &str,
    shasta_k8s_secrets: serde_json::Value,
    sat_template_file_yaml: serde_yaml::Value,
    hsm_group_available_vec: &[String],
    ansible_verbosity_opt: Option<u8>,
    ansible_passthrough_opt: Option<&str>,
    gitea_base_url: &str,
    gitea_token: &str,
    do_not_reboot: bool,
    watch_logs: bool,
    timestamps: bool,
    debug_on_failure: bool,
    overwrite: bool,
    dry_run: bool,
  ) -> Result<(), Error> {
    match self {
      CSM(b) => {
        b.apply_sat_file(
          shasta_token,
          shasta_base_url,
          shasta_root_cert,
          vault_base_url,
          vault_secret_path,
          k8s_api_url,
          shasta_k8s_secrets,
          sat_template_file_yaml,
          hsm_group_available_vec,
          ansible_verbosity_opt,
          ansible_passthrough_opt,
          gitea_base_url,
          gitea_token,
          do_not_reboot,
          watch_logs,
          timestamps,
          debug_on_failure,
          overwrite,
          dry_run,
        )
        .await
      }
      OCHAMI(b) => {
        b.apply_sat_file(
          shasta_token,
          shasta_base_url,
          shasta_root_cert,
          vault_base_url,
          vault_secret_path,
          k8s_api_url,
          shasta_k8s_secrets,
          sat_template_file_yaml,
          hsm_group_available_vec,
          ansible_verbosity_opt,
          ansible_passthrough_opt,
          gitea_base_url,
          gitea_token,
          do_not_reboot,
          watch_logs,
          timestamps,
          debug_on_failure,
          overwrite,
          dry_run,
        )
        .await
      }
    }
  }
}
