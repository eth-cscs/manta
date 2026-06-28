//! IMS image helpers shared by handlers that need to locate or
//! cross-reference images by CFS configuration name (e.g. boot-config
//! application, SAT-file rendering).

use std::collections::HashMap;

use manta_backend_dispatcher::{
  error::Error,
  interfaces::{cfs::CfsTrait, ims::ImsTrait},
  types::ims::Image,
};

use crate::server::common::app_context::InfraContext;

/// Fan out IMS PATCH calls for every image in `images` concurrently.
///
/// Extracts the image id from each [`Image`] value and calls
/// `backend.update_image`. Uses [`futures::future::try_join_all`] so
/// all per-image PATCH requests are in flight simultaneously; the first
/// error encountered is returned to the caller (same semantics as the
/// sequential loops this helper replaces).
///
/// # Errors
///
/// - [`Error::MissingField`] when an image has no `id`.
/// - [`Error::NetError`] / [`Error::CsmError`] from the backend
///   `update_image` call.
pub(crate) async fn apply_image_patches(
  infra: &InfraContext<'_>,
  token: &str,
  images: &HashMap<String, Image>,
) -> Result<(), Error> {
  futures::future::try_join_all(images.values().map(|image| async move {
    let image_id = image.id.as_deref().ok_or_else(|| {
      Error::MissingField("Image id is missing".to_string())
    })?;
    infra
      .backend
      .update_image(token, image_id, &image.clone().into())
      .await
  }))
  .await
  .map(|_| ())
}

/// Return the IMS images produced by succeeded image-build CFS
/// sessions that referenced `cfs_configuration_name`.
///
/// The CFS session list is filtered to entries whose configuration
/// matches, whose target definition is `"image"`, and which carry at
/// least one `result_id`. For each matching session every result id
/// is looked up in IMS; misses are logged and skipped so a partially
/// garbage-collected IMS doesn't break callers that just want
/// whatever images still exist (boot-config application, SAT-file
/// rendering, etc.).
///
/// # Errors
///
/// [`Error::NetError`] / [`Error::CsmError`] from the backend
/// `get_sessions` call. IMS image lookups that 404 are logged and
/// skipped without surfacing an error.
pub async fn get_image_vec_related_cfs_configuration_name(
  infra: &InfraContext<'_>,
  shasta_token: &str,
  cfs_configuration_name: String,
) -> Result<Vec<Image>, Error> {
  tracing::info!(
    "Searching in CFS sessions for image ID related to CFS configuration '{}'",
    cfs_configuration_name
  );

  let cfs_session_vec = infra
    .backend
    .get_sessions(
      shasta_token,
      None,
      None,
      None,
      None,
      None,
      None,
      None,
      Some(true),
      None,
    )
    .await?;

  // Filter to sessions related to the CFS configuration that built an image
  let cfs_session_image_succeeded_vec =
    cfs_session_vec.iter().filter(|cfs_session| {
      cfs_session
        .get_configuration_name()
        .is_some_and(|name| name.eq(&cfs_configuration_name))
        && cfs_session
          .get_target_def()
          .is_some_and(|def| def.eq("image"))
        && cfs_session.get_first_result_id().is_some()
    });

  // Deduplicate image ids across all matching sessions before fetching.
  let image_ids: std::collections::HashSet<String> =
    cfs_session_image_succeeded_vec
      .flat_map(|s| s.get_result_id_vec())
      .collect();

  let fetch_results =
    futures::future::join_all(image_ids.iter().map(|id| async move {
      (
        id.clone(),
        infra
          .backend
          .get_images(shasta_token, Some(id.as_str()))
          .await,
      )
    }))
    .await;

  let mut boot_image_id_vec = Vec::new();
  for (id, rslt) in fetch_results {
    match rslt {
      Ok(mut images) => boot_image_id_vec.append(&mut images),
      Err(e) => tracing::warn!("Failed to fetch image '{}': {}", id, e),
    }
  }

  Ok(boot_image_id_vec)
}
