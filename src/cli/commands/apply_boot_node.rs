use crate::{
  cli::commands::power_common::{self, PowerAction},
  common::{
    self, app_context::AppContext,
    ims_ops::get_image_vec_related_cfs_configuration_name,
  },
  manta_backend_dispatcher::StaticBackendDispatcher,
};

use anyhow::{Context, Error, bail};

use manta_backend_dispatcher::{
  interfaces::{bss::BootParametersTrait, cfs::CfsTrait, ims::ImsTrait},
  types::{
    bss::BootParameters,
    ims::{Image, PatchImage},
  },
};
use std::collections::HashMap;

/// Apply a boot configuration to specific nodes.
#[allow(clippy::too_many_arguments)]
pub async fn exec(
  ctx: &AppContext<'_>,
  new_boot_image_id_opt: Option<&str>,
  new_boot_image_configuration_opt: Option<&str>,
  new_runtime_configuration_opt: Option<&str>,
  new_kernel_parameters_opt: Option<&str>,
  hosts_expression: &str,
  assume_yes: bool,
  do_not_reboot: bool,
  dry_run: bool,
) -> Result<(), Error> {
  let backend = ctx.backend;
  let site_name = ctx.site_name;
  let shasta_base_url = ctx.shasta_base_url;
  let shasta_root_cert = ctx.shasta_root_cert;

  let shasta_token =
    common::authentication::get_api_token(backend, site_name).await?;

  let mut need_restart = false;

  // Convert user input to xname
  let xname_vec = common::node_ops::resolve_hosts_expression(
    backend,
    &shasta_token,
    hosts_expression,
    false,
  )
  .await?;

  let mut current_node_boot_param_vec: Vec<BootParameters> = backend
    .get_bootparameters(&shasta_token, &xname_vec)
    .await
    .context("Failed to get boot parameters")?;

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

  // IMPORTANT: ALWAYS SET KERNEL PARAMS BEFORE BOOT
  // IMAGE BECAUSE KERNEL ALSO UPDATES THE BOOT IMAGE,
  // THEREFORE IF USER WANTS TO CHANGE BOTH, CHANGING
  // THE BOOT IMAGE LATER WILL PUT THE RIGHT ONE IN
  // PLACE.
  if let Some(new_kernel_parameters) = new_kernel_parameters_opt {
    need_restart |= apply_kernel_params(
      &mut current_node_boot_param_vec,
      new_kernel_parameters,
    )?;
  }

  let mut image_vec = collect_boot_images(
    backend,
    &shasta_token,
    &mut current_node_boot_param_vec,
    new_boot_image_opt,
    &mut need_restart,
  )
  .await?;

  log::debug!(
    "boot params to update vec:\n{:#?}",
    current_node_boot_param_vec
  );

  // Update images that need to be projected through
  // "sbps"
  if current_node_boot_param_vec
    .first()
    .context("No boot parameters found")?
    .is_root_kernel_param_iscsi_ready()
  {
    for image in image_vec.values_mut() {
      image.set_boot_image_iscsi_ready();
    }
  }

  if need_restart {
    if common::user_interaction::confirm(
      &format!(
        "This operation will modify the nodes \
         below:\n{:?}\nDo you want to continue?",
        xname_vec
      ),
      assume_yes,
    ) {
      log::info!("Continue",);
    } else {
      bail!("Operation cancelled by user");
    }
  } else {
    bail!("No changes detected. Nothing to do");
  }

  if dry_run {
    println!(
      "Dry-run enabled. No changes persisted \
       into the system"
    );
    println!(
      "New boot parameters:\n{}",
      serde_json::to_string_pretty(&current_node_boot_param_vec)
        .context("Failed to serialize boot parameters",)?
    );
    println!(
      "Images:\n{}",
      serde_json::to_string_pretty(&image_vec)
        .context("Failed to serialize images")?
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
    // NOTE: this is going to force CFS session to
    // configure the nodes
    if let Some(new_runtime_configuration_name) = new_runtime_configuration_opt
    {
      println!(
        "Updating runtime configuration to '{}'",
        new_runtime_configuration_name
      );

      backend
        .update_runtime_configuration(
          &shasta_token,
          shasta_base_url,
          shasta_root_cert,
          &xname_vec,
          new_runtime_configuration_name,
          true,
        )
        .await
        .context("Error updating runtime configuration")?;

      // Update images
      for (_, image) in image_vec {
        let image_id = image.id.clone().context("Image id is missing")?;
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

      power_common::exec_nodes(
        ctx,
        PowerAction::Reset,
        &nodes.join(","),
        true,
        assume_yes,
        "table",
      )
      .await?;
    }

    Ok(())
  }
}

async fn get_new_boot_image(
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
      shasta_token,
      shasta_base_url,
      shasta_root_cert,
      new_boot_image_configuration.to_string(),
    )
    .await?;

    if image_vec.is_empty() {
      bail!(
        "Could not find boot image related to \
           configuration '{}'",
        new_boot_image_configuration
      );
    }

    backend.filter_images(&mut image_vec)?;

    // Get the most recent image related to the CFS
    // configuration
    let most_recent_image = image_vec
      .iter()
      .last()
      .context("No image found for configuration")?;

    log::debug!(
      "Boot image id related to configuration \
         '{}' found:\n{:#?}",
      new_boot_image_configuration,
      most_recent_image
    );

    Some(most_recent_image.clone())
  } else if let Some(boot_image_id) = new_boot_image_id_opt {
    log::info!("Boot image id '{}' provided", boot_image_id);
    // Check image id exists
    let image_in_csm_vec = backend
      .get_images(shasta_token, new_boot_image_id_opt)
      .await?;

    if image_in_csm_vec.is_empty() {
      bail!("boot image id '{}' not found", boot_image_id);
    }

    image_in_csm_vec.first().cloned()
  } else {
    None
  };

  Ok(new_boot_image)
}

/// Apply new kernel parameters to all boot parameters,
/// returning `true` if any parameter actually changed.
fn apply_kernel_params(
  boot_param_vec: &mut [BootParameters],
  new_kernel_parameters: &str,
) -> Result<bool, Error> {
  let mut any_changed = false;

  for boot_parameter in boot_param_vec.iter_mut() {
    log::info!(
      "Updating '{:?}' kernel parameters to '{}'",
      boot_parameter.hosts,
      new_kernel_parameters
    );

    let changed = boot_parameter.apply_kernel_params(new_kernel_parameters);
    any_changed = changed || any_changed;

    log::info!("need restart? {}", any_changed);

    let image_id = boot_parameter.try_get_boot_image_id().ok_or_else(|| {
      Error::msg(format!(
        "Could not get boot image id from boot \
           parameters for hosts: {:?}",
        boot_parameter.hosts
      ))
    })?;

    // Update boot image to stay in sync with kernel
    // params (latest image id and etag)
    let _ = boot_parameter
      .update_boot_image(&image_id, &boot_parameter.get_boot_image_etag())?;
  }

  Ok(any_changed)
}

/// Collect boot images: if a new image was provided, update
/// all boot params to use it; otherwise fetch the current
/// boot image for each node.
async fn collect_boot_images(
  backend: &StaticBackendDispatcher,
  shasta_token: &str,
  boot_param_vec: &mut [BootParameters],
  new_boot_image_opt: Option<Image>,
  need_restart: &mut bool,
) -> Result<HashMap<String, Image>, Error> {
  let mut image_vec = HashMap::<String, Image>::new();

  if let Some(new_boot_image) = new_boot_image_opt {
    let new_boot_image_id = new_boot_image
      .id
      .as_ref()
      .context("New boot image id is missing")?
      .clone();

    let new_boot_image_etag = new_boot_image
      .link
      .as_ref()
      .and_then(|link| link.etag.as_ref())
      .context("New boot image etag is missing")?;

    image_vec.insert(new_boot_image_id.clone(), new_boot_image.clone());

    let any_differ = boot_param_vec.iter().any(|bp| {
      bp.try_get_boot_image_id().as_deref() != Some(new_boot_image_id.as_str())
    });

    if any_differ {
      for boot_parameter in boot_param_vec.iter_mut() {
        log::info!(
          "Updating '{:?}' boot image to '{}'",
          boot_parameter.hosts,
          new_boot_image_id
        );
        boot_parameter
          .update_boot_image(&new_boot_image_id, new_boot_image_etag)
          .context("Failed to update boot image")?;
      }

      *need_restart = true;
    }
  } else {
    // No new image — fetch current boot image for each
    // node (may need updating if "root" uses "sbps")
    for boot_parameter in boot_param_vec.iter() {
      let boot_image_id =
        boot_parameter.try_get_boot_image_id().ok_or_else(|| {
          Error::msg(format!(
            "Could not get boot image id from boot \
             parameters for hosts: {:?}",
            boot_parameter.hosts
          ))
        })?;

      let boot_image = backend
        .get_images(shasta_token, Some(boot_image_id.as_str()))
        .await?
        .first()
        .context("No image found for boot image id")?
        .clone();

      image_vec.insert(boot_image_id, boot_image);
    }
  }

  Ok(image_vec)
}
