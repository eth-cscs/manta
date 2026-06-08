//! IMS image queries and safety-checked deletion (rejects images that boot live nodes).

use manta_backend_dispatcher::error::Error;
use manta_backend_dispatcher::types::Group;
use manta_backend_dispatcher::types::bss::BootParameters;
use manta_backend_dispatcher::types::ims::Image;

use crate::server::common::app_context::InfraContext;
use crate::service::authorization::validate_user_group_vec_access;
use crate::service::boot_parameters::get_restricted_boot_parameters;
pub use manta_shared::types::params::image::GetImagesParams;

/// Fetch IMS images from the backend, sorted by creation time.
///
/// Honors `params.id` and `params.limit`. The `pattern` regex is applied
/// by the CLI client after the response is received; the service does
/// not filter on name here.
pub async fn get_images(
  infra: &InfraContext<'_>,
  token: &str,
  params: &GetImagesParams,
) -> Result<Vec<Image>, Error> {
  let mut image_vec = infra.get_images(token, params.id.as_deref()).await?;

  if let Some(limit) = params.limit {
    image_vec.truncate(limit as usize);
  }

  image_vec.sort_by_key(|image| image.created.clone());

  Ok(image_vec)
}

/// Validate that images can be deleted (not used by boot nodes,
/// not restricted). Does NOT perform deletion.
pub async fn validate_image_deletion(
  infra: &InfraContext<'_>,
  token: &str,
  image_id_vec: &[&str],
  settings_group_name_opt: Option<&str>,
) -> Result<(), Error> {
  // Get list of target groups the user is asking for
  let target_group_vec: Vec<String> =
    if let Some(group) = &settings_group_name_opt {
      vec![group.to_string()]
    } else {
      infra
        .get_group_available(token)
        .await?
        .iter()
        .map(|group| group.label.clone())
        .collect()
    };

  // Validate groups and get list of groups available
  validate_user_group_vec_access(infra, token, &target_group_vec).await?;

  let (group_available_vec, boot_parameter_vec) = tokio::try_join!(
    infra.get_group_available(token),
    infra.get_all_bootparameters(token),
  )?;

  // Check if any requested image is used to boot nodes
  let image_used_to_boot_nodes: Vec<String> = boot_parameter_vec
    .iter()
    .map(manta_backend_dispatcher::types::bss::BootParameters::try_get_boot_image_id)
    .collect::<Option<Vec<String>>>()
    .ok_or_else(|| {
      Error::MissingField(
        "Could not get image ids used to boot nodes".to_string(),
      )
    })?;

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
        .map(std::string::ToString::to_string)
        .collect::<Vec<_>>()
        .join(", ")
    )));
  }

  // Check restricted images
  let image_restricted_vec =
    get_restricted_image_ids(&group_available_vec, &boot_parameter_vec)
      .ok_or_else(|| {
        Error::MissingField(
          "Could not get restricted image ids used by boot parameters"
            .to_string(),
        )
      })?;

  if !image_restricted_vec.is_empty() {
    return Err(Error::BadRequest(format!(
      "The following image ids can't be deleted \
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
  validate_image_deletion(
    infra,
    token,
    image_id_vec,
    settings_hsm_group_name_opt,
  )
  .await?;

  let mut deleted = Vec::new();
  for image_id in image_id_vec {
    match infra.delete_image(token, image_id).await {
      Ok(()) => {
        tracing::info!("Image {} deleted successfully", image_id);
        deleted.push((*image_id).to_string());
      }
      Err(e) => tracing::error!(
        "Failed to delete image {}: {}. Continuing",
        image_id,
        e
      ),
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
    .map(manta_backend_dispatcher::types::bss::BootParameters::try_get_boot_image_id)
    .collect()
}
