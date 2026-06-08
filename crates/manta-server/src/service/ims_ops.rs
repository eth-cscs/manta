//! IMS image helpers shared by handlers that need to locate or
//! cross-reference images by CFS configuration name (e.g. boot-config
//! application, SAT-file rendering).

use manta_backend_dispatcher::{error::Error, types::ims::Image};

use crate::server::common::app_context::InfraContext;

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

  let mut boot_image_id_vec = Vec::new();

  for cfs_session in cfs_session_image_succeeded_vec {
    let cfs_session_name = cfs_session.name.clone();

    for image_id in cfs_session.get_result_id_vec() {
      tracing::info!(
        "Checking if result_id {} in CFS session {} exists",
        image_id,
        cfs_session_name
      );

      let image_vec_rslt =
        infra.get_images(shasta_token, Some(&image_id)).await;

      match image_vec_rslt {
        Ok(mut image_vec) => {
          tracing::info!(
            "Found the image ID '{}' related to CFS sesison '{}'",
            image_id,
            cfs_session_name,
          );

          boot_image_id_vec.append(&mut image_vec);
        }
        Err(e) => {
          tracing::warn!(
            "Failed to fetch image '{}' for CFS session '{}': {}",
            image_id,
            cfs_session_name,
            e
          );
        }
      }
    }
  }

  Ok(boot_image_id_vec)
}
