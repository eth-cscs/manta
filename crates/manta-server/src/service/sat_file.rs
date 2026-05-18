//! SAT file rendering (Jinja2) and application through the backend.

use manta_backend_dispatcher::{
  error::Error,
  interfaces::{apply_sat_file::SatTrait, hsm::group::GroupTrait},
};

use crate::common::app_context::InfraContext;
pub use manta_shared::shared::params::sat_file::ApplySatFileParams;

/// Render, filter, and apply a SAT file via the backend.
pub async fn apply_sat_file(
  infra: &InfraContext<'_>,
  token: &str,
  gitea_token: &str,
  vault_base_url: &str,
  k8s_api_url: &str,
  params: ApplySatFileParams<'_>,
) -> Result<(), Error> {
  let values_cli_vec: Vec<String> = params
    .values
    .and_then(|v| v.as_object())
    .map(|map| {
      map
        .iter()
        .map(|(k, v)| format!("{}={}", k, v.as_str().unwrap_or(&v.to_string())))
        .collect()
    })
    .unwrap_or_default();

  let sat_template_yaml =
    manta_shared::shared::sat_file::render_jinja2_sat_file_yaml(
      params.sat_file_content,
      params.values_file_content,
      if values_cli_vec.is_empty() {
        None
      } else {
        Some(&values_cli_vec)
      },
    )
    .map_err(crate::wire_conv::to_backend)?;

  let mut sat_file: manta_shared::shared::sat_file::SatFile =
    serde_yaml::from_value(sat_template_yaml)?;

  sat_file
    .filter(params.image_only, params.session_template_only)
    .map_err(crate::wire_conv::to_backend)?;

  let sat_file_yaml = serde_yaml::to_value(sat_file)?;

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
