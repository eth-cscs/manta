use anyhow::{Error, bail};
use manta_backend_dispatcher::{
  interfaces::{
    bss::BootParametersTrait, hsm::group::GroupTrait, ims::ImsTrait,
  },
  types::{Group, bss::BootParameters},
};

use crate::common::{
  app_context::AppContext, authentication::get_api_token,
  authorization::get_groups_names_available,
  boot_parameters::get_restricted_boot_parameters,
};

/// Delete IMS images and their linked artifacts.
pub async fn exec(
  ctx: &AppContext<'_>,
  image_id_vec: &[&str],
  dry_run: bool,
) -> Result<(), Error> {
  log::info!(
    "Executing command to delete images: {}",
    image_id_vec.join(", "),
  );
  let backend = ctx.backend;
  let site_name = ctx.site_name;
  let shasta_base_url = ctx.shasta_base_url;
  let shasta_root_cert = ctx.shasta_root_cert;
  let settings_hsm_group_name_opt = ctx.settings_hsm_group_name_opt;
  let shasta_token = get_api_token(backend, site_name).await?;
  let _hsm_name_available_vec = get_groups_names_available(
    backend,
    &shasta_token,
    None,
    settings_hsm_group_name_opt,
  )
  .await?;

  let group_available_vec = backend.get_group_available(&shasta_token).await?;

  let boot_parameter_vec =
    backend.get_all_bootparameters(&shasta_token).await?;

  // Get list of image ids that are used to boot nodes (Node of these images can be deleted)
  let image_used_to_boot_nodes_opt: Option<Vec<String>> = boot_parameter_vec
    .iter()
    .map(|boot_param| boot_param.try_get_boot_image_id())
    .collect();

  let image_used_to_boot_nodes: Vec<String> = image_used_to_boot_nodes_opt
    .ok_or_else(|| Error::msg("Could not get image ids used to boot nodes"))?;

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
    bail!(
      "The following images could not be deleted \
       since they boot nodes.\n{:#?}",
      image_xnames_boot_map
    );
  }

  // Get list of boot parameters that user can't delete because it's host is not a member of
  // the groups available
  let image_restricted_vec: Vec<String> =
    get_restricted_image_ids(&group_available_vec, &boot_parameter_vec)
      .ok_or_else(|| {
        Error::msg("Could not get restricted image ids used by boot parameters")
      })?;

  if !image_restricted_vec.is_empty() {
    bail!(
      "The following image ids are not deletable \
       because they are used by hosts that are not part \
       of the groups available to the user:\n{:#?}",
      image_restricted_vec
    );
  }

  for image_id in image_id_vec {
    if dry_run {
      eprintln!("Dry-run enabled. No changes persisted into the system");
      eprintln!("Image {} would be deleted", image_id);
    } else {
      let del_rslt = backend
        .delete_image(
          &shasta_token,
          shasta_base_url,
          shasta_root_cert,
          image_id,
        )
        .await;

      match del_rslt {
        Ok(_) => {
          log::info!("Image {} deleted successfully", image_id);
        }
        Err(e) => {
          log::error!("Failed to delete image {}: {}. Continuing", image_id, e);
        }
      }
    }
  }

  println!("Images deleted:\n{:?}", image_id_vec);

  Ok(())
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
