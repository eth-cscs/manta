//! IMS image queries and safety-checked deletion (rejects images that boot live nodes).

use manta_backend_dispatcher::error::Error;
use manta_backend_dispatcher::interfaces::bss::BootParametersTrait;
use manta_backend_dispatcher::interfaces::ims::ImsTrait;
use manta_backend_dispatcher::types::Group;
use manta_backend_dispatcher::types::bss::BootParameters;
use manta_backend_dispatcher::types::ims::Image;

use crate::server::common::app_context::InfraContext;
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
  let mut image_vec =
    infra.backend.get_images(token, params.id.as_deref()).await?;

  if let Some(limit) = params.limit {
    image_vec.truncate(limit as usize);
  }

  image_vec.sort_by_key(|image| image.created.clone());

  Ok(image_vec)
}

/// Refuse a planned image delete that would orphan a live boot path
/// or touch an image scoped to a group the caller can't reach.
///
/// Two checks run after access validation: any image listed in
/// `image_id_vec` that is the current boot image of an existing BSS
/// record fails with `BadRequest` (deleting it would brick the next
/// boot); any image whose boot record targets hosts outside the
/// caller's available groups fails the same way (so a user can't
/// indirectly remove an image they don't own through a shared id).
/// Pure check — no deletion happens here.
pub async fn validate_image_deletion(
  infra: &InfraContext<'_>,
  token: &str,
  image_id_vec: &[&str],
  settings_group_name_opt: Option<&str>,
) -> Result<(), Error> {
  // One backend fetch + in-memory validation, replacing the prior
  // three round-trips. See `service::group::resolve_target_and_available_groups`.
  let (group_available_vec, _target_group_vec) =
    crate::service::group::resolve_target_and_available_groups(
      infra,
      token,
      settings_group_name_opt,
    )
    .await?;

  let boot_parameter_vec = infra.backend.get_all_bootparameters(token).await?;

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

  // `image_used_to_boot_nodes` is cluster-scale (one entry per BSS
  // record). Hash it once so the safety check across user-supplied
  // delete ids is O(D) rather than O(D·N).
  let image_used_to_boot_nodes_set: std::collections::HashSet<&str> =
    image_used_to_boot_nodes
      .iter()
      .map(String::as_str)
      .collect();
  let image_xnames_boot_map: Vec<&&str> = image_id_vec
    .iter()
    .filter(|id| image_used_to_boot_nodes_set.contains(**id))
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

/// Run [`validate_image_deletion`] then delete each image in
/// `image_id_vec`, best-effort.
///
/// Individual delete failures are logged and skipped — the function
/// keeps going so a single backend hiccup doesn't strand the rest of
/// the batch. The returned vector lists exactly the ids the backend
/// confirmed removed.
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
    match infra.backend.delete_image(token, image_id).await {
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
