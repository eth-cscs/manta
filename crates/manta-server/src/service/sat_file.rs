//! SAT file apply orchestration (Vault + K8s + backend).
//!
//! Rendering (Jinja2), parsing, and `image_only` / `session_template_only`
//! filtering are performed client-side by the CLI; this layer receives the
//! post-processed SAT YAML and forwards it to the backend together with
//! the K8s secrets and the available HSM groups.

use manta_backend_dispatcher::{
  error::Error,
  interfaces::{apply_sat_file::SatTrait, hsm::group::GroupTrait},
};

use crate::server::common::app_context::InfraContext;
pub use manta_shared::shared::params::sat_file::ApplySatFileParams;

/// Apply a pre-rendered SAT file via the backend.
pub async fn apply_sat_file(
  infra: &InfraContext<'_>,
  token: &str,
  gitea_token: &str,
  vault_base_url: &str,
  k8s_api_url: &str,
  params: ApplySatFileParams<'_>,
) -> Result<(), Error> {
  let sat_file_yaml: serde_yaml::Value = serde_yaml::from_str(params.sat_yaml)?;

  let shasta_k8s_secrets =
    crate::server::common::vault::http_client::fetch_shasta_k8s_secrets_from_vault(
      vault_base_url,
      infra.site_name,
      token,
    )
    .await?;

  let hsm_group_available_vec =
    infra.backend.get_group_name_available(token).await?;

  infra
    .backend
    .apply_sat_file(
      token,
      infra.shasta_base_url,
      infra.shasta_root_cert,
      vault_base_url,
      infra.site_name,
      k8s_api_url,
      shasta_k8s_secrets,
      sat_file_yaml,
      &hsm_group_available_vec,
      params.ansible_verbosity,
      params.ansible_passthrough,
      infra.gitea_base_url,
      gitea_token,
      params.reboot,
      params.watch_logs,
      params.timestamps,
      true,
      params.overwrite,
      params.dry_run,
    )
    .await
}
