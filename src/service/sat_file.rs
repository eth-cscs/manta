use anyhow::{Context, Error};
use manta_backend_dispatcher::interfaces::{apply_sat_file::SatTrait, hsm::group::GroupTrait};

use crate::common::app_context::InfraContext;

/// Parameters for applying a SAT file.
pub struct ApplySatFileParams<'a> {
  pub sat_file_content: &'a str,
  pub values: Option<&'a serde_json::Value>,
  pub values_file_content: Option<&'a str>,
  pub ansible_verbosity: Option<u8>,
  pub ansible_passthrough: Option<&'a str>,
  pub reboot: bool,
  pub watch_logs: bool,
  pub timestamps: bool,
  pub image_only: bool,
  pub session_template_only: bool,
  pub overwrite: bool,
  pub dry_run: bool,
}

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
    crate::cli::commands::apply_sat_file::utils::render_jinja2_sat_file_yaml(
      params.sat_file_content,
      params.values_file_content,
      if values_cli_vec.is_empty() { None } else { Some(&values_cli_vec) },
    )
    .context("Failed to render SAT file template")?;

  let mut sat_file: crate::cli::commands::apply_sat_file::utils::SatFile =
    serde_yaml::from_value(sat_template_yaml)
      .context("Failed to parse SAT file")?;

  sat_file
    .filter(params.image_only, params.session_template_only)
    .context("Failed to filter SAT file")?;

  let sat_file_yaml = serde_yaml::to_value(sat_file)
    .context("Failed to convert SAT file to YAML")?;

  let shasta_k8s_secrets =
    crate::common::vault::http_client::fetch_shasta_k8s_secrets_from_vault(
      vault_base_url,
      infra.site_name,
      token,
    )
    .await
    .context("Failed to fetch k8s secrets from vault")?;

  let hsm_group_available_vec = infra
    .backend
    .get_group_name_available(token)
    .await
    .context("Failed to get available HSM groups")?;

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
    .context("Failed to apply SAT file")
}
