use anyhow::{Context, Error};
use manta_backend_dispatcher::interfaces::ims::GetImagesAndDetailsTrait;
use manta_backend_dispatcher::types::ims::Image;

use crate::common::authorization::get_groups_names_available;
use crate::manta_backend_dispatcher::StaticBackendDispatcher;

/// Typed parameters for fetching IMS images.
pub struct GetImagesParams {
  pub id: Option<String>,
  pub hsm_group: Option<String>,
  pub settings_hsm_group_name: Option<String>,
  pub limit: Option<u8>,
}

/// Fetch images and their associated details from the backend.
///
/// Returns tuples of (Image, CFS config name, HSM groups string, bool).
pub async fn get_images(
  backend: &StaticBackendDispatcher,
  token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  params: &GetImagesParams,
) -> Result<Vec<(Image, String, String, bool)>, Error> {
  let target_hsm_group_vec = get_groups_names_available(
    backend,
    token,
    params.hsm_group.as_deref(),
    params.settings_hsm_group_name.as_deref(),
  )
  .await
  .context("Failed to get available HSM group names")?;

  let limit_ref = params.limit.as_ref();

  let image_detail_vec = backend
    .get_images_and_details(
      token,
      shasta_base_url,
      shasta_root_cert,
      &target_hsm_group_vec,
      params.id.as_deref(),
      limit_ref,
    )
    .await?;

  Ok(image_detail_vec)
}
