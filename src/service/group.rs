use anyhow::{Context, Error};
use manta_backend_dispatcher::interfaces::hsm::group::GroupTrait;
use manta_backend_dispatcher::types::Group;

use crate::common::authorization::get_groups_names_available;
use crate::manta_backend_dispatcher::StaticBackendDispatcher;

/// Typed parameters for fetching HSM groups.
pub struct GetGroupParams {
  pub group_name: Option<String>,
  pub settings_hsm_group_name: Option<String>,
}

/// Fetch HSM groups from the backend.
///
/// Resolves available group names, then fetches their details.
pub async fn get_groups(
  backend: &StaticBackendDispatcher,
  token: &str,
  params: &GetGroupParams,
) -> Result<Vec<Group>, Error> {
  let target_hsm_group_vec = get_groups_names_available(
    backend,
    token,
    params.group_name.as_deref(),
    params.settings_hsm_group_name.as_deref(),
  )
  .await?;

  let group_vec = backend
    .get_groups(token, Some(&target_hsm_group_vec))
    .await
    .context("Failed to fetch HSM groups")?;

  Ok(group_vec)
}
