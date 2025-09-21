use crate::{
  common::{self, audit::Audit, jwt_ops, kafka::Kafka},
  manta_backend_dispatcher::StaticBackendDispatcher,
};
use dialoguer::theme::ColorfulTheme;
use manta_backend_dispatcher::{
  error::Error,
  interfaces::{
    bss::BootParametersTrait,
    hsm::{component::ComponentTrait, group::GroupTrait},
    ims::ImsTrait,
  },
  types::{self, ims::Image},
};
use nodeset::NodeSet;
use std::collections::HashMap;

/// Updates the kernel parameters for a set of nodes
/// reboots the nodes which kernel params have changed
pub async fn exec(
  backend: StaticBackendDispatcher,
  shasta_token: &str,
  kernel_params: &str,
  hosts_expression: &str,
  assume_yes: bool,
  do_not_reboot: bool,
  kafka_audit_opt: Option<&Kafka>,
  dry_run: bool,
) -> Result<(), Error> {
  let mut need_restart = false;
  log::info!("Apply kernel parameters");

  // Convert user input to xname
  let node_metadata_available_vec = backend
    .get_node_metadata_available(shasta_token)
    .await
    .unwrap_or_else(|e| {
      eprintln!("ERROR - Could not get node metadata. Reason:\n{e}\nExit");
      std::process::exit(1);
    });

  let xname_vec = common::node_ops::from_hosts_expression_to_xname_vec(
    hosts_expression,
    false,
    node_metadata_available_vec,
  )
  .await
  .unwrap_or_else(|e| {
    eprintln!(
      "ERROR - Could not convert user input to list of xnames. Reason:\n{}",
      e
    );
    std::process::exit(1);
  });

  let mut xname_to_reboot_vec: Vec<String> = Vec::new();
  let mut image_map: HashMap<String, Image> = HashMap::new();

  let mut current_node_boot_params_vec: Vec<types::bss::BootParameters> =
    backend
      .get_bootparameters(
        shasta_token,
        &xname_vec
          .iter()
          .map(|xname| xname.to_string())
          .collect::<Vec<String>>(),
      )
      .await
      .unwrap();

  let node_group: NodeSet = xname_vec.join(", ").parse().unwrap();

  for mut boot_parameter in &mut current_node_boot_params_vec {
    log::info!(
      "Apply '{}' kernel parameters to '{}'",
      kernel_params,
      node_group,
    );

    let kernel_params_changed =
      boot_parameter.apply_kernel_params(&kernel_params);
    need_restart = kernel_params_changed || need_restart;

    if kernel_params_changed {
      need_restart = true;
      xname_to_reboot_vec.extend(boot_parameter.hosts.iter().cloned());
    }

    // Set image metadata to sbps
    // Check 'root' kernel parameters for sbps
    let image_id = boot_parameter.get_boot_image_id();

    if !image_map.contains_key(&image_id) {
      let mut image: Image = backend
        .get_images(shasta_token, Some(image_id.as_str()))
        .await?
        .first()
        .unwrap()
        .clone();

      if boot_parameter.is_root_kernel_param_iscsi_ready() {
        let proceed = if assume_yes {
          true
        } else {
          dialoguer::Confirm::with_theme(
        &ColorfulTheme::default())
        .with_prompt("Kernel parameters using SBPS/iSCSI. Do you want to project the boot image through SBPS?")
        .interact()
        .unwrap()
        };

        if proceed {
          log::info!(
            "Setting 'sbps-project' metadata to 'true' for image id '{}'",
            image_id
          );

          image.set_boot_image_iscsi_ready();

          log::debug!("Image:\n{:#?}", image);

          image_map.insert(image_id, image.clone());
        } else {
          log::info!("User chose to not project the image through SBPS");
        }
      }
    }
  }

  if need_restart {
    let proceed = if assume_yes {
      true
    } else {
      println!(
        "Apply kernel parameters:\n{:?}\nTo nodes:\n{:?}",
        kernel_params,
        node_group.to_string()
      );

      dialoguer::Confirm::with_theme(
        &ColorfulTheme::default())
        .with_prompt("This operation will replace the kernel parameters for the nodes below. Please confirm to proceed")
        .interact()
        .unwrap()
    };

    if !proceed {
      println!("Operation canceled by the user. Exit");
      std::process::exit(1);
    }
  } else {
    println!("No changes detected. Nothing to do. Exit");
    std::process::exit(0);
  }

  log::info!("need restart? {}", need_restart);

  if dry_run {
    println!("Dry-run enabled. No changes persisted into the system");
    println!(
      "Dry run mode. Would update kernel parameters below:\n{}",
      serde_json::to_string_pretty(&current_node_boot_params_vec).unwrap()
    );
    println!(
      "Dry run mode. Would update images:\n{}",
      serde_json::to_string_pretty(&image_map).unwrap()
    );
  } else {
    log::info!("Persist changes");

    // Update boot parameters
    for mut boot_parameter in current_node_boot_params_vec {
      log::info!(
        "Apply '{}' kernel parameters to '{:?}'",
        kernel_params,
        boot_parameter.hosts
      );

      backend
        .update_bootparameters(shasta_token, &boot_parameter)
        .await?;
    }

    // Update images projected through SBPS
    for (_, image) in image_map {
      backend
        .update_image(
          shasta_token,
          image.id.clone().unwrap().as_str(),
          &image.into(),
        )
        .await?;
    }
  }

  // Audit
  if let Some(kafka_audit) = kafka_audit_opt {
    let username = jwt_ops::get_name(shasta_token).unwrap();
    let user_id = jwt_ops::get_preferred_username(shasta_token).unwrap();

    // FIXME: We should not need to make this call here but at the beginning of the method as a
    // prerequisite
    let xnames: Vec<&str> =
      xname_vec.iter().map(|xname| xname.as_str()).collect();

    let group_map_vec = backend
      .get_group_map_and_filter_by_member_vec(shasta_token, &xnames)
      .await?;

    let msg_json = serde_json::json!(
        { "user": {"id": user_id, "name": username}, "host": {"hostname": xname_vec}, "group": group_map_vec.keys().collect::<Vec<_>>(), "message": format!("Apply kernel parameters: {}", kernel_params)});

    let msg_data = serde_json::to_string(&msg_json)
      .expect("Could not serialize audit message data");

    if let Err(e) = kafka_audit.produce_message(msg_data.as_bytes()).await {
      log::warn!("Failed producing messages: {}", e);
    }
    // log::info!(target: "app::audit", "User: {} ({}) ; Operation: Add kernel parameters to {:?}", jwt_ops::get_name(shasta_token).unwrap_or("".to_string()), jwt_ops::get_preferred_username(shasta_token).unwrap_or("".to_string()), xname_vec);
  }

  // Reboot if needed
  if !do_not_reboot && need_restart && !dry_run {
    log::info!("Restarting nodes");

    crate::cli::commands::power_reset_nodes::exec(
      &backend,
      shasta_token,
      &xname_to_reboot_vec.join(","),
      true,
      assume_yes,
      "table",
      kafka_audit_opt,
    )
    .await;
  }

  Ok(())
}
