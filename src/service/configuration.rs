use anyhow::{Context, Error, bail};
use chrono::NaiveDateTime;
use manta_backend_dispatcher::interfaces::cfs::CfsTrait;
use manta_backend_dispatcher::interfaces::delete_configurations_and_data_related::DeleteConfigurationsAndDataRelatedTrait;
use manta_backend_dispatcher::types::cfs::cfs_configuration_details::{
  ConfigurationDetails, LayerDetails,
};
use manta_backend_dispatcher::types::cfs::cfs_configuration_response::CfsConfigurationResponse;
use manta_backend_dispatcher::types::cfs::session::CfsSessionGetResponse;
use manta_backend_dispatcher::types::bos::session_template::BosSessionTemplate;
use manta_backend_dispatcher::types::ims::Image;

use crate::common::app_context::InfraContext;
use crate::common::authorization::get_groups_names_available;

/// Typed parameters for fetching CFS configurations.
pub struct GetConfigurationParams {
  pub name: Option<String>,
  pub pattern: Option<String>,
  pub hsm_group: Option<String>,
  pub settings_hsm_group_name: Option<String>,
  pub since: Option<NaiveDateTime>,
  pub until: Option<NaiveDateTime>,
  pub limit: Option<u8>,
}

/// Fetch and filter CFS configurations from the backend.
pub async fn get_configurations(
  infra: &InfraContext<'_>,
  token: &str,
  params: &GetConfigurationParams,
) -> Result<Vec<CfsConfigurationResponse>, Error> {
  let target_hsm_group_vec = get_groups_names_available(
    infra.backend,
    token,
    params.hsm_group.as_deref(),
    params.settings_hsm_group_name.as_deref(),
  )
  .await?;

  let limit_ref = params.limit.as_ref();

  let cfs_configuration_vec = infra.backend
    .get_and_filter_configuration(
      token,
      infra.shasta_base_url,
      infra.shasta_root_cert,
      params.name.as_deref(),
      params.pattern.as_deref(),
      &target_hsm_group_vec,
      params.since,
      params.until,
      limit_ref,
    )
    .await?;

  Ok(cfs_configuration_vec)
}

/// Fetch detailed configuration info including layer details and derivatives.
///
/// This fetches the VCS/Gitea token from Vault internally since it is
/// specific to configuration detail queries.
pub async fn get_configuration_details(
  infra: &InfraContext<'_>,
  token: &str,
  config: &CfsConfigurationResponse,
) -> Result<
  (
    ConfigurationDetails,
    Option<Vec<CfsSessionGetResponse>>,
    Option<Vec<BosSessionTemplate>>,
    Option<Vec<Image>>,
  ),
  Error,
> {
  let vault_base_url = infra
    .vault_base_url
    .context("vault_base_url is required for configuration details")?;

  let gitea_token =
    crate::common::vault::http_client::fetch_shasta_vcs_token(
      token,
      vault_base_url,
      infra.site_name,
    )
    .await
    .context("Failed to fetch VCS token from vault")?;

  // Fetch all layer details concurrently instead of sequentially.
  let layer_details_vec: Vec<LayerDetails> =
    futures::future::try_join_all(config.layers.iter().map(|layer| {
      let backend = infra.backend.clone();
      let root_cert = infra.shasta_root_cert.to_vec();
      let gitea_base_url = infra.gitea_base_url.to_string();
      let gitea_token = gitea_token.clone();
      let site_name = infra.site_name.to_string();
      let layer = layer.clone();
      async move {
        backend
          .get_configuration_layer_details(
            &root_cert,
            &gitea_base_url,
            &gitea_token,
            layer,
            &site_name,
          )
          .await
          .context("Could not fetch configuration layer details")
      }
    }))
    .await?;

  let (cfs_session_vec_opt, bos_sessiontemplate_vec_opt, image_vec_opt) =
    infra.backend
      .get_derivatives(token, infra.shasta_base_url, infra.shasta_root_cert, &config.name)
      .await
      .context("Could not fetch configuration derivatives")?;

  let details = ConfigurationDetails::new(
    &config.name,
    &config.last_updated,
    layer_details_vec,
  );

  Ok((
    details,
    cfs_session_vec_opt,
    bos_sessiontemplate_vec_opt,
    image_vec_opt,
  ))
}

/// Data gathered for deletion review and execution.
#[derive(serde::Serialize)]
pub struct DeletionCandidates {
  pub cfs_sessions_to_delete: Vec<CfsSessionGetResponse>,
  pub bos_sessiontemplate_tuples: Vec<(String, String, String)>,
  pub image_ids: Vec<String>,
  pub configuration_names: Vec<String>,
  pub cfs_session_tuples: Vec<(String, String, String)>,
  pub configurations: Vec<CfsConfigurationResponse>,
}

/// Fetch deletion candidates (no side effects).
pub async fn get_deletion_candidates(
  infra: &InfraContext<'_>,
  token: &str,
  settings_hsm_group_name_opt: Option<&str>,
  configuration_name_pattern: Option<&str>,
  since: Option<NaiveDateTime>,
  until: Option<NaiveDateTime>,
) -> Result<DeletionCandidates, Error> {
  if let (Some(s), Some(u)) = (since, until) {
    if s > u {
      bail!("'since' date can't be after 'until' date");
    }
  }

  let target_hsm_group_vec =
    if let Some(settings_hsm_group_name) = settings_hsm_group_name_opt {
      vec![settings_hsm_group_name.to_string()]
    } else {
      get_groups_names_available(
        infra.backend,
        token,
        None,
        settings_hsm_group_name_opt,
      )
      .await?
    };

  let (
    cfs_sessions_to_delete,
    bos_sessiontemplate_tuples,
    image_ids,
    configuration_names,
    cfs_session_tuples,
    configurations,
  ) = infra
    .backend
    .get_data_to_delete(
      token,
      infra.shasta_base_url,
      infra.shasta_root_cert,
      &target_hsm_group_vec,
      configuration_name_pattern,
      since,
      until,
    )
    .await?;

  Ok(DeletionCandidates {
    cfs_sessions_to_delete,
    bos_sessiontemplate_tuples,
    image_ids,
    configuration_names,
    cfs_session_tuples,
    configurations,
  })
}

/// Execute the deletion of configurations and derivatives.
pub async fn delete_configurations_and_derivatives(
  infra: &InfraContext<'_>,
  token: &str,
  candidates: &DeletionCandidates,
) -> Result<(), Error> {
  let cfs_session_name_vec: Vec<String> = candidates
    .cfs_session_tuples
    .iter()
    .map(|(session, _, _)| session.clone())
    .collect();

  let bos_sessiontemplate_name_vec: Vec<String> = candidates
    .bos_sessiontemplate_tuples
    .iter()
    .map(|(sessiontemplate, _, _)| sessiontemplate.clone())
    .collect();

  infra
    .backend
    .delete(
      token,
      infra.shasta_base_url,
      infra.shasta_root_cert,
      &candidates.configuration_names,
      &candidates.image_ids,
      &cfs_session_name_vec,
      &bos_sessiontemplate_name_vec,
    )
    .await?;

  Ok(())
}
