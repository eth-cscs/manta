use anyhow::{Context, Error};
use manta_backend_dispatcher::interfaces::bos::ClusterTemplateTrait;
use manta_backend_dispatcher::interfaces::hsm::group::GroupTrait;
use manta_backend_dispatcher::types::bos::session_template::BosSessionTemplate;

use crate::common::authorization::get_groups_names_available;
use crate::manta_backend_dispatcher::StaticBackendDispatcher;

/// Typed parameters for fetching BOS session templates.
pub struct GetTemplateParams {
  pub name: Option<String>,
  pub hsm_group: Option<String>,
  pub settings_hsm_group_name: Option<String>,
  pub limit: Option<u8>,
}

/// Fetch and filter BOS session templates from the backend.
///
/// Resolves available HSM groups, fetches their members, then
/// queries and filters templates accordingly.
pub async fn get_templates(
  backend: &StaticBackendDispatcher,
  token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  params: &GetTemplateParams,
) -> Result<Vec<BosSessionTemplate>, Error> {
  let target_hsm_group_vec = get_groups_names_available(
    backend,
    token,
    params.hsm_group.as_deref(),
    params.settings_hsm_group_name.as_deref(),
  )
  .await?;

  let hsm_member_vec = backend
    .get_member_vec_from_group_name_vec(token, &target_hsm_group_vec)
    .await?;

  let limit_ref = params.limit.as_ref();

  log::info!(
    "Get BOS sessiontemplates for HSM groups: {:?}",
    target_hsm_group_vec
  );

  let mut bos_sessiontemplate_vec = backend
    .get_and_filter_templates(
      token,
      shasta_base_url,
      shasta_root_cert,
      &target_hsm_group_vec,
      &hsm_member_vec,
      params.name.as_deref(),
      limit_ref,
    )
    .await
    .context("Could not get BOS sessiontemplate list")?;

  bos_sessiontemplate_vec.sort_by(|a, b| a.name.cmp(&b.name));

  Ok(bos_sessiontemplate_vec)
}
