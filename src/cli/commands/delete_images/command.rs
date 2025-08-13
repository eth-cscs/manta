use manta_backend_dispatcher::{
  interfaces::{
    bss::BootParametersTrait, hsm::group::GroupTrait, ims::ImsTrait,
  },
  types::{bss::BootParameters, Group},
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
  hsm_name_available_vec: Vec<String>,
  image_id_vec: &[&str],
  dry_run: bool,
) {
  log::info!(
    "Executing command to delete images: {}",
    image_id_vec.join(", "),
  );

  let group_available_vec_rslt =
    backend.get_group_available(shasta_token).await;

  let group_available_vec = group_available_vec_rslt.unwrap_or_else(|e| {
    eprintln!("ERROR - {}", e);
    std::process::exit(1);
  });

  let boot_parameter_vec_rslt =
    backend.get_all_bootparameters(shasta_token).await;

  let mut boot_parameter_vec = boot_parameter_vec_rslt.unwrap_or_else(|e| {
    eprintln!("ERROR - {}", e);
    std::process::exit(1);
  });

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
    eprintln!(
        "ERROR - The following images could not be deleted since they boot nodes.\n{:#?}",
        image_xnames_boot_map
      );
    std::process::exit(1);
  }

  // Get list of boot parameters that user can't delete because it's host is not a member of
  // the groups available
  let image_restricted_vec: Vec<String> =
    get_restricted_image_ids(&group_available_vec, &boot_parameter_vec);

  if !image_restricted_vec.is_empty() {
    eprintln!(
        "ERROR - The following image ids are not deletable because they are used by hosts that are not part of the groups available to the user:\n{:#?}",
        image_restricted_vec
      );
    std::process::exit(1);
  }

  if dry_run {
    eprintln!("Dry-run enabled. No changes persisted into the system");
  } else {
    for image_id in image_id_vec {
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
