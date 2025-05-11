use anyhow::Error;
use manta_backend_dispatcher::{
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
  log::info!(
    "Searching in CFS sessions for image ID related to CFS configuration '{}'",
    cfs_configuration_name
  );

  // Get all CFS sessions related which has succeeded and built an image related to CFS
  // configuration

  // Get all CFS sessions which has succeeded
  let cfs_session_vec = backend
    .get_and_filter_sessions(
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
    )
    .await
    .unwrap();

  // Filter CFS sessions to the ones related to CFS configuration and built an image (target
  // definition is 'image' and it actually has at least one artifact)
  let cfs_session_image_succeeded_vec =
    cfs_session_vec.iter().filter(|cfs_session| {
      cfs_session
        .get_configuration_name()
        .unwrap()
        .eq(&cfs_configuration_name)
        && cfs_session.get_target_def().unwrap().eq("image")
        && cfs_session.get_first_result_id().is_some()
    });

  let mut boot_image_id_vec = Vec::new();

  // Find image in CFS sessions
  for cfs_session in cfs_session_image_succeeded_vec {
    let cfs_session_name = cfs_session.name.as_ref().unwrap();

    for image_id in cfs_session.get_result_id_vec() {
      log::info!(
        "Checking if result_id {} in CFS session {} exists",
        image_id,
        cfs_session_name
      );

      // Get IMS image related to the CFS session
      let image_vec_rslt = backend
        .get_images(
          shasta_token,
          shasta_base_url,
          shasta_root_cert,
          Some(&image_id),
        )
        .await;

      if let Ok(mut image_vec) = image_vec_rslt {
        log::info!(
          "Found the image ID '{}' related to CFS sesison '{}'",
          image_id,
          cfs_session_name,
        );

        boot_image_id_vec.append(&mut image_vec);
      };
    }
  }

  Ok(boot_image_id_vec)
}
