use manta_backend_dispatcher::error::Error;
use manta_backend_dispatcher::interfaces::bss::BootParametersTrait;
use manta_backend_dispatcher::interfaces::hsm::group::GroupTrait;
use manta_backend_dispatcher::interfaces::ims::GetImagesAndDetailsTrait;
use manta_backend_dispatcher::interfaces::ims::ImsTrait;
use manta_backend_dispatcher::types::bss::BootParameters;
use manta_backend_dispatcher::types::ims::Image;
use manta_backend_dispatcher::types::Group;

use crate::common::app_context::InfraContext;
use crate::common::authorization::get_groups_names_available;
use crate::common::boot_parameters::get_restricted_boot_parameters;

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
  infra: &InfraContext<'_>,
  token: &str,
  params: &GetImagesParams,
) -> Result<Vec<(Image, String, String, bool)>, Error> {
  let target_hsm_group_vec = get_groups_names_available(
    infra.backend,
    token,
    params.hsm_group.as_deref(),
    params.settings_hsm_group_name.as_deref(),
  )
  .await?;

  let limit_ref = params.limit.as_ref();

  let image_detail_vec = infra.backend
    .get_images_and_details(
      token,
      infra.shasta_base_url,
      infra.shasta_root_cert,
      &target_hsm_group_vec,
      params.id.as_deref(),
      limit_ref,
    )
    .await?;

  Ok(image_detail_vec)
}

/// Validate that images can be deleted (not used by boot nodes,
/// not restricted). Does NOT perform deletion.
pub async fn validate_image_deletion(
  infra: &InfraContext<'_>,
  token: &str,
  image_id_vec: &[&str],
  settings_hsm_group_name_opt: Option<&str>,
) -> Result<(), Error> {
  let backend = infra.backend;

  get_groups_names_available(backend, token, None, settings_hsm_group_name_opt)
    .await?;

  let (group_available_vec, boot_parameter_vec) = tokio::try_join!(
    backend.get_group_available(token),
    backend.get_all_bootparameters(token),
  )?;

  // Check if any requested image is used to boot nodes
  let image_used_to_boot_nodes: Vec<String> = boot_parameter_vec
    .iter()
    .map(|boot_param| boot_param.try_get_boot_image_id())
    .collect::<Option<Vec<String>>>()
    .ok_or_else(|| Error::Message("Could not get image ids used to boot nodes".to_string()))?;

  let image_xnames_boot_map: Vec<&&str> = image_id_vec
    .iter()
    .filter(|id| image_used_to_boot_nodes.contains(&id.to_string()))
    .collect();

  if !image_xnames_boot_map.is_empty() {
    return Err(Error::BadRequest(format!(
      "The following images could not be deleted \
       since they boot nodes.\n{}",
      image_xnames_boot_map
        .iter()
        .map(|id| id.to_string())
        .collect::<Vec<_>>()
        .join(", ")
    )));
  }

  // Check restricted images
  let image_restricted_vec =
    get_restricted_image_ids(&group_available_vec, &boot_parameter_vec)
      .ok_or_else(|| {
        Error::Message("Could not get restricted image ids used by boot parameters".to_string())
      })?;

  if !image_restricted_vec.is_empty() {
    return Err(Error::BadRequest(format!(
      "The following image ids are not deletable \
       because they are used by hosts that are not part \
       of the groups available to the user:\n{}",
      image_restricted_vec.join(", ")
    )));
  }

  Ok(())
}

/// Validate and delete IMS images.
/// Returns list of image IDs that were successfully deleted.
pub async fn delete_images(
  infra: &InfraContext<'_>,
  token: &str,
  image_id_vec: &[&str],
  settings_hsm_group_name_opt: Option<&str>,
) -> Result<Vec<String>, Error> {
  validate_image_deletion(infra, token, image_id_vec, settings_hsm_group_name_opt)
    .await?;

  let mut deleted = Vec::new();
  for image_id in image_id_vec {
    let del_rslt = infra
      .backend
      .delete_image(
        token,
        infra.shasta_base_url,
        infra.shasta_root_cert,
        image_id,
      )
      .await;

    match del_rslt {
      Ok(_) => {
        tracing::info!("Image {} deleted successfully", image_id);
        deleted.push(image_id.to_string());
      }
      Err(e) => {
        tracing::error!("Failed to delete image {}: {}. Continuing", image_id, e);
      }
    }
  }

  Ok(deleted)
}

fn get_restricted_image_ids(
  group_available_vec: &[Group],
  boot_parameter_vec: &[BootParameters],
) -> Option<Vec<String>> {
  get_restricted_boot_parameters(group_available_vec, boot_parameter_vec)
    .iter()
    .map(|boot_param| boot_param.try_get_boot_image_id())
    .collect()
}
