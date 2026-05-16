use manta_backend_dispatcher::{
  error::Error,
  interfaces::{cfs::CfsTrait, ims::ImsTrait},
  types::ims::Image,
};

use crate::manta_backend_dispatcher::StaticBackendDispatcher;

/// This function retrieves the list of image IDs related to a CFS configuration name.
/// It first checks the CFS sessions for any succeeded sessions that built an image related to the
/// CFS configuration. Then, it checks if the image ID exists in the IMS.
pub async fn get_image_vec_related_cfs_configuration_name(
  backend: &StaticBackendDispatcher,
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  cfs_configuration_name: String,
) -> Result<Vec<Image>, Error> {
  tracing::info!(
    "Searching in CFS sessions for image ID related to CFS configuration '{}'",
    cfs_configuration_name
  );

  let cfs_session_vec = backend
    .get_sessions(
      shasta_token,
      shasta_base_url,
      shasta_root_cert,
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
        backend.get_images(shasta_token, Some(&image_id)).await;

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
