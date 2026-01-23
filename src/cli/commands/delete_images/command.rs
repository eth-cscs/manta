use anyhow::Error;
use manta_backend_dispatcher::{
  interfaces::{
    bss::BootParametersTrait, hsm::group::GroupTrait, ims::ImsTrait,
  },
  types::{Group, bss::BootParameters},
};

use crate::{
  common::boot_parameters::get_restricted_boot_parameters,
  manta_backend_dispatcher::StaticBackendDispatcher,
};

pub async fn exec(
  backend: &StaticBackendDispatcher,
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  _hsm_name_available_vec: Vec<String>,
  image_id_vec: &[&str],
  dry_run: bool,
) -> Result<(), Error> {
  log::info!(
    "Executing command to delete images: {}",
    image_id_vec.join(", "),
  );

  let group_available_vec = backend.get_group_available(shasta_token).await?;

  let boot_parameter_vec = backend.get_all_bootparameters(shasta_token).await?;

  // Get list of image ids that are used to boot nodes (Node of these images can be deleted)
  let image_used_to_boot_nodes: Vec<String> = boot_parameter_vec
    .iter()
    .map(|boot_param| boot_param.get_boot_image_id())
    .collect();

  // Get list of image ids requested to delete that are used to boot nodes (these images cannot
  // be deleted and will trigger the command to fail)
  let mut image_xnames_boot_map = Vec::new();
  for image_id in image_id_vec {
    if image_used_to_boot_nodes.contains(&image_id.to_string()) {
      image_xnames_boot_map.push(image_id);
    }
  }

  // Exit if any image id user wants to delete is used to boot nodes
  if !image_xnames_boot_map.is_empty() {
    return Err(Error::msg(format!(
      "ERROR - The following images could not be deleted since they boot nodes.\n{:#?}",
      image_xnames_boot_map
    )));
  }

  // Get list of boot parameters that user can't delete because it's host is not a member of
  // the groups available
  let image_restricted_vec: Vec<String> =
    get_restricted_image_ids(&group_available_vec, &boot_parameter_vec);

  if !image_restricted_vec.is_empty() {
    return Err(Error::msg(format!(
      "ERROR - The following image ids are not deletable because they are used by hosts that are not part of the groups available to the user:\n{:#?}",
      image_restricted_vec
    )));
  }

  for image_id in image_id_vec {
    if dry_run {
      eprintln!("Dry-run enabled. No changes persisted into the system");
      eprintln!("Image {} would be deleted", image_id);
    } else {
      let del_rslt = backend
        .delete_image(shasta_token, shasta_base_url, shasta_root_cert, image_id)
        .await;

      match del_rslt {
        Ok(_) => {
          log::info!("Image {} deleted successfully", image_id);
        }
        Err(e) => {
          eprintln!(
            "ERROR - Failed to delete image {}:\n{}\nContinue",
            image_id, e
          );
        }
      }
    }
  }

  println!("Images deleted:\n{:?}", image_id_vec);

  Ok(())
}

pub fn get_restricted_image_ids(
  group_available_vec: &[Group],
  boot_parameter_vec: &[BootParameters],
) -> Vec<String> {
  get_restricted_boot_parameters(group_available_vec, boot_parameter_vec)
    .iter()
    .map(|boot_param| boot_param.get_boot_image_id())
    .collect()
}
