use crate::{
  cli::commands::power_reset_nodes,
  common::{
    self, ims_ops::get_image_vec_related_cfs_configuration_name, kafka::Kafka,
  },
  manta_backend_dispatcher::StaticBackendDispatcher,
};

use anyhow::Error;
use dialoguer::{theme::ColorfulTheme, Confirm};
use manta_backend_dispatcher::{
  interfaces::{
    bss::BootParametersTrait, cfs::CfsTrait, hsm::component::ComponentTrait,
    ims::ImsTrait,
  },
  types::BootParameters,
};

pub async fn exec(
  backend: &StaticBackendDispatcher,
  shasta_token: &str,
  shasta_base_url: &str,
  shasta_root_cert: &[u8],
  new_boot_image_id_opt: Option<&String>,
  new_boot_image_configuration_opt: Option<&String>,
  new_runtime_configuration_opt: Option<&String>,
  new_kernel_parameters_opt: Option<&String>,
  hosts_expression: &str,
  assume_yes: bool,
  do_not_reboot: bool,
  dry_run: bool,
  kafka_audit_opt: Option<&Kafka>,
) -> Result<(), Error> {
  let mut need_restart = false;

  // Convert user input to xname
  let node_metadata_available_vec =
    backend.get_node_metadata_available(shasta_token).await?;

  let xname_vec = common::node_ops::from_hosts_expression_to_xname_vec(
    hosts_expression,
    false,
    node_metadata_available_vec,
  )
  .await?;

  // VALIDATE
  // Check new configuration exists and exit otherwise
  let _ = backend
    .get_configuration(
      shasta_token,
      shasta_base_url,
      shasta_root_cert,
      new_runtime_configuration_opt,
    )
    .await?;

  let mut current_node_boot_param_vec: Vec<BootParameters> = backend
    .get_bootparameters(
      shasta_token,
      &xname_vec
        .iter()
        .map(|xname| xname.to_string())
        .collect::<Vec<String>>(),
    )
    .await
    .unwrap();

  // Get new boot image
  let new_boot_image_id_opt: Option<String> = if let Some(
    new_boot_image_configuration,
  ) =
    new_boot_image_configuration_opt
  {
    log::info!(
      "Boot configuration '{}' provided",
      new_boot_image_configuration
    );
    let mut image_vec = get_image_vec_related_cfs_configuration_name(
      backend,
      shasta_token,
      shasta_base_url,
      shasta_root_cert,
      new_boot_image_configuration.to_string(),
    )
    .await?;

    if image_vec.is_empty() {
      eprintln!(
        "ERROR - Could not find boot image related to configuration '{}'",
        new_boot_image_configuration
      );
      std::process::exit(1);
    }

    backend.filter_images(&mut image_vec)?;

    let most_recent_image_related_to_cfs_configuration =
      image_vec.iter().last().unwrap();

    let image_id = most_recent_image_related_to_cfs_configuration.id.clone();

    println!(
      "Boot image id related to configuration '{}' found\n{:#?}",
      new_boot_image_configuration, image_id
    );

    image_id
  } else if let Some(boot_image_id) = new_boot_image_id_opt {
    log::info!("Boot image id '{}' provided", boot_image_id);
    // Check image id exists
    /* let image_id_in_csm = ims::image::http_client::get(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        new_boot_image_id_opt.map(|image_id| image_id.as_str()),
    )
    .await; */
    let image_id_in_csm = backend
      .get_images(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        new_boot_image_id_opt.map(|image_id| image_id.as_str()),
      )
      .await?;

    if image_id_in_csm.is_empty() {
      eprintln!("ERROR - boot image id '{}' not found", boot_image_id);
      std::process::exit(1);
    }

    Some(boot_image_id).cloned()
  } else {
    None
  };

  // Update BSS BOOT PARAMETERS
  //
  // IMPORTANT: ALWAYS SET KERNEL PARAMS BEFORE BOOT IMAGE BECAUSE KERNEL ALSO UPDATES THE BOOT
  // IMAGE, THEREFORE IF USER WANTS TO CHANGE BOTH KERNEL PARAMS AND BOOT IMAGE, THEN, CHANGING
  //
  // THE BOOT IMAGE LATER WILL MAKE SURE WE PUT IN PLACE THE RIGHT BOOT IMAGE
  // Update kernel parameters
  if let Some(new_kernel_parameters) = new_kernel_parameters_opt {
    // Update boot params
    current_node_boot_param_vec
            .iter_mut()
            .for_each(|boot_parameter| {
                log::info!(
                    "Updating '{:?}' kernel parameters to '{}'",
                    boot_parameter.hosts,
                    new_kernel_parameters
                );

                let kernel_params_changed =
                    boot_parameter.apply_kernel_params(&new_kernel_parameters);
                need_restart = kernel_params_changed || need_restart;

                /* need_restart =
                need_restart || boot_parameter.apply_kernel_params(&new_kernel_parameters); */

                log::info!("need restart? {}", need_restart);
                let _ = boot_parameter.update_boot_image(&boot_parameter.get_boot_image());
            });
  }

  log::debug!("new kernel params: {:#?}", current_node_boot_param_vec);

  // IMPORTANT: ALWAYS SET KERNEL PARAMS BEFORE BOOT IMAGE BECAUSE KERNEL ALSO UPDATES THE BOOT
  // IMAGE, THEREFORE IF USER WANTS TO CHANGE BOTH KERNEL PARAMS AND BOOT IMAGE, THEN, CHANGING
  //
  // Update boot image
  //
  // Check if boot image changes and notify the user and update the node boot params struct
  if let Some(new_boot_image_id) = new_boot_image_id_opt {
    let boot_params_to_update_vec: Vec<&BootParameters> =
      current_node_boot_param_vec
        .iter()
        .filter(|boot_param| boot_param.get_boot_image() != new_boot_image_id)
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

          let _ = boot_parameter.update_boot_image(&new_boot_image_id);
        });

      need_restart = true;
    }
  } else {
    /* need_restart = false;
    log::info!("Boot image not defined. No need to reboot."); */
  }

  log::debug!(
    "boot params to update vec:\n{:#?}",
    current_node_boot_param_vec
  );

  if !assume_yes {
    if need_restart {
      if Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt(format!(
                    "This operation will modify the nodes below:\n{:?}\nDo you want to continue?",
                    xname_vec
                ))
                .interact()
                .unwrap()
            {
                log::info!("Continue",);
            } else {
                println!("Cancelled by user. Aborting.");
                std::process::exit(0);
            }
    }
  }

  if dry_run {
    println!("Dry-run enabled. No changes persisted into the system");
    println!("New boot parameters:\n{:#?}", current_node_boot_param_vec);
    Ok(())
  } else {
    log::info!("Persist changes");

    // Update boot params
    for boot_parameter in current_node_boot_param_vec {
      log::debug!("Updating boot parameter:\n{:#?}", boot_parameter);
      let component_patch_rep = backend
        .update_bootparameters(
          // shasta_base_url,
          shasta_token,
          // shasta_root_cert,
          &boot_parameter,
        )
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
          shasta_token,
          shasta_base_url,
          shasta_root_cert,
          xname_vec.iter().map(|xname| xname.to_string()).collect(), // TODO: modify function signature
          new_runtime_configuration_name,
          true,
        )
        .await
        .inspect_err(|e| {
          eprintln!("Error updating runtime configuration: {}", e);
          std::process::exit(1);
        });
    } else {
      log::info!("Runtime configuration does not change.");
    }

    if !do_not_reboot && need_restart {
      log::info!("Restarting nodes");

      let nodes: Vec<String> = xname_vec
        .into_iter()
        .map(|xname| xname.to_string())
        .collect();

      power_reset_nodes::exec(
        &backend,
        shasta_token,
        &nodes.join(","),
        true,
        assume_yes,
        "table",
        kafka_audit_opt,
      )
      .await;
    }

    Ok(())
  }
}
