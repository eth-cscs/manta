use crate::{
  cli::commands::power_reset_nodes,
  common::{
    self, ims_ops::get_image_vec_related_cfs_configuration_name, kafka::Kafka,
  },
  manta_backend_dispatcher::StaticBackendDispatcher,
};

use anyhow::Error;
use dialoguer::{Confirm, theme::ColorfulTheme};
use manta_backend_dispatcher::{
  interfaces::{
    bss::BootParametersTrait, cfs::CfsTrait, hsm::component::ComponentTrait,
    ims::ImsTrait,
  },
  types::{
    bss::BootParameters,
    ims::{Image, PatchImage},
  },
};
use std::collections::HashMap;

pub async fn exec(
  backend: &StaticBackendDispatcher,
  site_name: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  new_boot_image_id_opt: Option<&str>,
  new_boot_image_configuration_opt: Option<&str>,
  new_runtime_configuration_opt: Option<&str>,
  new_kernel_parameters_opt: Option<&str>,
  hosts_expression: &str,
  assume_yes: bool,
  do_not_reboot: bool,
  dry_run: bool,
  kafka_audit_opt: Option<&Kafka>,
) -> Result<(), Error> {
  let shasta_token =
    common::authentication::get_api_token(backend, site_name).await?;

  let mut need_restart = false;

  // Convert user input to xname
  let node_metadata_available_vec =
    backend.get_node_metadata_available(&shasta_token).await?;

  let xname_vec = common::node_ops::from_hosts_expression_to_xname_vec(
    hosts_expression,
    false,
    node_metadata_available_vec,
  )
  .await?;

  let mut current_node_boot_param_vec: Vec<BootParameters> = backend
    .get_bootparameters(&shasta_token, &xname_vec)
    .await
    .unwrap();

  // Get new boot image
  let new_boot_image_opt = get_new_boot_image(
    backend,
    &shasta_token,
    shasta_base_url,
    shasta_root_cert,
    new_boot_image_configuration_opt,
    new_boot_image_id_opt,
  )
  .await?;

  // Update BSS BOOT PARAMETERS
  //
  // IMPORTANT: ALWAYS SET KERNEL PARAMS BEFORE BOOT IMAGE BECAUSE KERNEL ALSO UPDATES THE BOOT
  // IMAGE, THEREFORE IF USER WANTS TO CHANGE BOTH KERNEL PARAMS AND BOOT IMAGE, THEN, CHANGING
  //
  // THE BOOT IMAGE LATER WILL MAKE SURE WE PUT IN PLACE THE RIGHT BOOT IMAGE
  // Update kernel parameters
  if let Some(new_kernel_parameters) = new_kernel_parameters_opt {
    for boot_parameter in current_node_boot_param_vec.iter_mut() {
      log::info!(
        "Updating '{:?}' kernel parameters to '{}'",
        boot_parameter.hosts,
        new_kernel_parameters
      );

      // First. update the kernel parameters in the current boot params struct
      // Update boot params
      let kernel_params_changed =
        boot_parameter.apply_kernel_params(&new_kernel_parameters);

      need_restart = kernel_params_changed || need_restart;

      log::info!("need restart? {}", need_restart);

      let image_id =
        boot_parameter.try_get_boot_image_id().ok_or_else(|| {
          Error::msg(format!(
            "Could not get boot image id from boot parameters for hosts: {:?}",
            boot_parameter.hosts
          ))
        })?;

      // Second. update the boot image in the current boot params struct to make sure it is in
      // sync with the kernel params
      // with the latest boot image id and etag
      // This is important because kernel params update might have changed the boot image
      // associated with the kernel params
      let _ = boot_parameter
        .update_boot_image(&image_id, &boot_parameter.get_boot_image_etag())?;
    }
  }

  // IMPORTANT: ALWAYS SET KERNEL PARAMS BEFORE BOOT IMAGE BECAUSE KERNEL ALSO UPDATES THE BOOT
  // IMAGE, THEREFORE IF USER WANTS TO CHANGE BOTH KERNEL PARAMS AND BOOT IMAGE, THEN, CHANGING
  //
  // Update boot image
  //
  let mut image_vec = HashMap::<String, Image>::new();

  if let Some(new_boot_image) = new_boot_image_opt {
    // User provided new image and all boot parameters will use it
    let new_boot_image_id = new_boot_image.id.as_ref().unwrap().clone();

    let new_boot_image_etag = new_boot_image
      .link
      .as_ref()
      .and_then(|link| link.etag.as_ref())
      .unwrap();

    image_vec.insert(new_boot_image_id.clone(), new_boot_image.clone());

    let boot_params_to_update_vec: Vec<&BootParameters> =
      current_node_boot_param_vec
        .iter()
        .filter(|&boot_param| {
          boot_param.try_get_boot_image_id() != Some(new_boot_image_id.clone())
        })
        .collect();

    if !boot_params_to_update_vec.is_empty() {
      // Update boot params
      current_node_boot_param_vec
        .iter_mut()
        .for_each(|boot_parameter| {
          log::info!(
            "Updating '{:?}' boot image to '{}'",
            boot_parameter.hosts,
            new_boot_image_id
          );

          let _ = boot_parameter
            .update_boot_image(&new_boot_image_id, &new_boot_image_etag);
        });

      need_restart = true;
    }
  } else {
    // User did not provide new image but we might need to update images if "root" kernel param use
    // "sbps"
    for boot_parameter in &current_node_boot_param_vec {
      let boot_image_id =
        boot_parameter.try_get_boot_image_id().ok_or_else(|| {
          Error::msg(format!(
            "Could not get boot image id from boot parameters for hosts: {:?}",
            boot_parameter.hosts
          ))
        })?;

      let boot_image = backend
        .get_images(&shasta_token, Some(boot_image_id.as_str()))
        .await?
        .first()
        .unwrap()
        .clone();

      image_vec.insert(boot_image_id, boot_image);
    }
  }

  log::debug!(
    "boot params to update vec:\n{:#?}",
    current_node_boot_param_vec
  );

  // Update images that need to be projected through "sbps"
  if current_node_boot_param_vec
    .first()
    .unwrap()
    .is_root_kernel_param_iscsi_ready()
  {
    for (_, image) in &mut image_vec {
      image.set_boot_image_iscsi_ready();
    }
  }

  if need_restart {
    let proceed = if assume_yes {
      true
    } else {
      Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(format!(
          "This operation will modify the nodes below:\n{:?}\nDo you want to continue?",
          xname_vec
        ))
        .interact()
        .unwrap()
    };

    if proceed {
      log::info!("Continue",);
    } else {
      return Err(Error::msg("Operation cancelled by user"));
    }
  } else {
    return Err(Error::msg("No changes detected. Nothing to do. Exit"));
  }

  if dry_run {
    println!("Dry-run enabled. No changes persisted into the system");
    println!(
      "New boot parameters:\n{}",
      serde_json::to_string_pretty(&current_node_boot_param_vec).unwrap()
    );
    println!(
      "Images:\n{}",
      serde_json::to_string_pretty(&image_vec).unwrap()
    );
    Ok(())
  } else {
    log::info!("Persist changes");

    // Update boot params
    for boot_parameter in current_node_boot_param_vec {
      log::debug!("Updating boot parameter:\n{:#?}", boot_parameter);
      let component_patch_rep = backend
        .update_bootparameters(&shasta_token, &boot_parameter)
        .await;

      log::debug!(
        "Component boot parameters resp:\n{:#?}",
        component_patch_rep
      );
    }

    // Update desired configuration
    // NOTE: this is going to foce CFS session to configure the nodes
    if let Some(new_runtime_configuration_name) = new_runtime_configuration_opt
    {
      println!(
        "Updating runtime configuration to '{}'",
        new_runtime_configuration_name
      );

      let _ = backend
        .update_runtime_configuration(
          &shasta_token,
          shasta_base_url,
          shasta_root_cert,
          &xname_vec,
          new_runtime_configuration_name,
          true,
        )
        .await
        .map_err(|e| {
          Error::msg(format!("Error updating runtime configuration: {}", e))
        })?;

      // Update images
      for (_, image) in image_vec {
        let image_id = image.id.clone().unwrap();
        let patch_image: PatchImage = image.into();
        backend
          .update_image(&shasta_token, &image_id, &patch_image)
          .await?;
      }
    } else {
      log::info!("Runtime configuration does not change.");
    }

    if !do_not_reboot && need_restart && !dry_run {
      log::info!("Restarting nodes");

      let nodes: Vec<String> = xname_vec;

      power_reset_nodes::exec(
        &backend,
        &shasta_token,
        &nodes.join(","),
        true,
        assume_yes,
        "table",
        kafka_audit_opt,
      )
      .await?;
    }

    Ok(())
  }
}

pub async fn get_new_boot_image(
  backend: &StaticBackendDispatcher,
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  new_boot_image_configuration_opt: Option<&str>,
  new_boot_image_id_opt: Option<&str>,
) -> Result<Option<Image>, Error> {
  let new_boot_image = if let Some(new_boot_image_configuration) =
    new_boot_image_configuration_opt
  {
    log::info!(
      "Boot configuration '{}' provided",
      new_boot_image_configuration
    );
    let mut image_vec = get_image_vec_related_cfs_configuration_name(
      backend,
      &shasta_token,
      shasta_base_url,
      shasta_root_cert,
      new_boot_image_configuration.to_string(),
    )
    .await?;

    if image_vec.is_empty() {
      return Err(Error::msg(format!(
        "Could not find boot image related to configuration '{}'",
        new_boot_image_configuration
      )));
    }

    backend.filter_images(&mut image_vec)?;

    // Get the most recent image related to the CFS configuration
    let most_recent_image_related_to_cfs_configuration =
      image_vec.iter().last().unwrap();

    log::debug!(
      "Boot image id related to configuration '{}' found:\n{:#?}",
      new_boot_image_configuration,
      most_recent_image_related_to_cfs_configuration
    );

    Some(most_recent_image_related_to_cfs_configuration.clone())
  } else if let Some(boot_image_id) = new_boot_image_id_opt {
    log::info!("Boot image id '{}' provided", boot_image_id);
    // Check image id exists
    let image_in_csm_vec = backend
      .get_images(&shasta_token, new_boot_image_id_opt)
      .await?;

    if image_in_csm_vec.is_empty() {
      return Err(Error::msg(format!(
        "boot image id '{}' not found",
        boot_image_id
      )));
    }

    image_in_csm_vec.first().cloned()
  } else {
    None
  };

  Ok(new_boot_image)
}
