use anyhow::{Context, Error};
use chrono::NaiveDateTime;
use manta_backend_dispatcher::interfaces::cfs::CfsTrait;
use manta_backend_dispatcher::types::cfs::cfs_configuration_details::{
  ConfigurationDetails, LayerDetails,
};
use manta_backend_dispatcher::types::cfs::cfs_configuration_response::CfsConfigurationResponse;
use manta_backend_dispatcher::types::cfs::session::CfsSessionGetResponse;
use manta_backend_dispatcher::types::bos::session_template::BosSessionTemplate;
use manta_backend_dispatcher::types::ims::Image;

use crate::common::authorization::get_groups_names_available;
use crate::manta_backend_dispatcher::StaticBackendDispatcher;

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
  backend: &StaticBackendDispatcher,
  token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  params: &GetConfigurationParams,
) -> Result<Vec<CfsConfigurationResponse>, Error> {
  let target_hsm_group_vec = get_groups_names_available(
    backend,
    token,
    params.hsm_group.as_deref(),
    params.settings_hsm_group_name.as_deref(),
  )
  .await?;

  let limit_ref = params.limit.as_ref();

  let cfs_configuration_vec = backend
    .get_and_filter_configuration(
      token,
      shasta_base_url,
      shasta_root_cert,
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
  backend: &StaticBackendDispatcher,
  token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  gitea_base_url: &str,
  vault_base_url: &str,
  site_name: &str,
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
  let gitea_token =
    crate::common::vault::http_client::fetch_shasta_vcs_token(
      token,
      vault_base_url,
      site_name,
    )
    .await
    .context("Failed to fetch VCS token from vault")?;

  let mut layer_details_vec: Vec<LayerDetails> = vec![];

  for layer in &config.layers {
    let layer_details = backend
      .get_configuration_layer_details(
        shasta_root_cert,
        gitea_base_url,
        &gitea_token,
        layer.clone(),
        site_name,
      )
      .await
      .context("Could not fetch configuration layer details")?;

    layer_details_vec.push(layer_details);
  }

  let (cfs_session_vec_opt, bos_sessiontemplate_vec_opt, image_vec_opt) =
    backend
      .get_derivatives(token, shasta_base_url, shasta_root_cert, &config.name)
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
