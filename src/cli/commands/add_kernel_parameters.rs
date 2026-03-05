use crate::common::{
  self, app_context::AppContext, audit, authentication::get_api_token,
};
use anyhow::{Context, Error, bail};

use manta_backend_dispatcher::{
  interfaces::{
    bss::BootParametersTrait, hsm::group::GroupTrait, ims::ImsTrait,
  },
  types::{self, ims::Image},
};
use nodeset::NodeSet;
use std::collections::HashMap;

/// Updates the kernel parameters for a set of nodes
/// reboots the nodes which kernel params have changed
pub async fn exec(
  ctx: &AppContext<'_>,
  kernel_params: &str,
  hosts_expression: &str,
  overwrite: bool,
  assume_yes: bool,
  do_not_reboot: bool,
  dry_run: bool,
) -> Result<(), Error> {
  let backend = ctx.backend;
  let site_name = ctx.site_name;
  let kafka_audit_opt = ctx.kafka_audit_opt;
  let mut need_restart = false;
  log::info!("Add kernel parameters");

  let shasta_token = get_api_token(backend, site_name).await?;

  // Convert user input to xname
  let xname_vec = common::node_ops::resolve_hosts_expression(
    backend,
    &shasta_token,
    hosts_expression,
    false,
  )
  .await?;

  let mut xname_to_reboot_vec: Vec<String> = Vec::new();
  let mut image_map: HashMap<String, Image> = HashMap::new();

  let mut current_node_boot_params_vec: Vec<types::bss::BootParameters> =
    backend
      .get_bootparameters(&shasta_token, &xname_vec)
      .await
      .context("Failed to get boot parameters")?;

  let node_group: NodeSet = xname_vec
    .join(", ")
    .parse()
    .context("Failed to parse node list")?;

  for boot_parameter in &mut current_node_boot_params_vec {
    log::info!(
      "Add '{}' kernel parameters to '{}'",
      kernel_params,
      node_group,
    );

    let kernel_params_changed =
      boot_parameter.add_kernel_params(kernel_params, overwrite);

    if kernel_params_changed {
      need_restart = true;
      xname_to_reboot_vec.extend(boot_parameter.hosts.iter().cloned());
    }

    // Set image metadata to sbps
    // Check 'root' kernel parameters for sbps
    let image_id = boot_parameter.try_get_boot_image_id().ok_or_else(|| {
      Error::msg(format!(
        "Could not get boot image id from boot parameters for hosts: {:?}",
        boot_parameter.hosts
      ))
    })?;

    #[allow(clippy::map_entry)]
    if !image_map.contains_key(&image_id) {
      let mut image: Image = backend
        .get_images(&shasta_token, Some(image_id.as_str()))
        .await?
        .first()
        .context("No image found for the given image id")?
        .clone();

      if boot_parameter.is_root_kernel_param_iscsi_ready() {
        if common::user_interaction::confirm(
          "Kernel parameters using SBPS/iSCSI. Do you want to project the boot image through SBPS?",
          assume_yes,
        ) {
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
    println!(
      "Add kernel params:\n{:?}\nFor nodes:\n{:?}",
      kernel_params,
      node_group.to_string()
    );
    if !common::user_interaction::confirm(
      "This operation will add the kernel parameters for the nodes below. Please confirm to proceed",
      assume_yes,
    ) {
      bail!("Operation canceled by the user.");
    }
  } else {
    bail!("No changes detected. Nothing to do");
  }

  log::info!("need restart? {}", need_restart);

  if dry_run {
    println!("Dry-run enabled. No changes persisted into the system");
    println!(
      "Dry run mode. Would update kernel parameters below: {}",
      serde_json::to_string_pretty(&current_node_boot_params_vec)
        .context("Failed to serialize boot parameters")?
    );
    println!(
      "Dry run mode. Would update images:\n{}",
      serde_json::to_string_pretty(&image_map)
        .context("Failed to serialize image map")?
    );
  } else {
    log::info!("Persist changes");

    // Update boot parameters
    for boot_parameter in current_node_boot_params_vec {
      log::info!(
        "Add '{}' kernel parameters to '{:?}'",
        kernel_params,
        boot_parameter.hosts,
      );

      backend
        .update_bootparameters(&shasta_token, &boot_parameter)
        .await?;
    }

    // Update images projected through SBPS
    for (_, image) in image_map {
      backend
        .update_image(
          &shasta_token,
          image.id.clone().context("Image has no id")?.as_str(),
          &image.into(),
        )
        .await?;
    }
  }

  // Audit
  if let Some(kafka_audit) = kafka_audit_opt {
    // FIXME: We should not need to make this call here but at the beginning of the method as a
    // prerequisite
    let xnames: Vec<&str> =
      xname_vec.iter().map(|xname| xname.as_str()).collect();

    let group_map_vec = backend
      .get_group_map_and_filter_by_member_vec(&shasta_token, &xnames)
      .await?;

    audit::send_audit(
      kafka_audit,
      &shasta_token,
      format!("Add kernel parameters: {}", kernel_params),
      Some(serde_json::json!(xname_vec)),
      Some(serde_json::json!(group_map_vec.keys().collect::<Vec<_>>())),
    )
    .await;
  }

  // Reboot if needed
  if !do_not_reboot && need_restart && !dry_run {
    log::info!("Restarting nodes");

    crate::cli::commands::power_common::exec_nodes(
      ctx,
      crate::cli::commands::power_common::PowerAction::Reset,
      &xname_to_reboot_vec.join(","),
      true,
      assume_yes,
      "table",
    )
    .await?;
  }

  Ok(())
}
