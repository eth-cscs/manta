use anyhow::{Context, Error, bail};
use manta_backend_dispatcher::{
  interfaces::{
    bss::BootParametersTrait,
    cfs::CfsTrait,
    ims::ImsTrait,
  },
  types::{
    bss::BootParameters,
    ims::{Image, PatchImage},
  },
};
use std::collections::HashMap;

use crate::common;
use crate::common::app_context::InfraContext;
use crate::common::authorization::validate_target_hsm_members;
use crate::common::ims_ops::get_image_vec_related_cfs_configuration_name;
use crate::manta_backend_dispatcher::StaticBackendDispatcher;

/// Typed parameters for fetching boot parameters.
pub struct GetBootParametersParams {
  pub hsm_group: Option<String>,
  pub nodes: Option<String>,
  pub settings_hsm_group_name: Option<String>,
}

/// Fetch boot parameters for the specified nodes.
///
/// Resolves target nodes from HSM group or node list, then
/// fetches their BSS boot parameters.
pub async fn get_boot_parameters(
  infra: &InfraContext<'_>,
  token: &str,
  params: &GetBootParametersParams,
) -> Result<Vec<BootParameters>, Error> {
  let xname_vec = common::node_ops::resolve_target_nodes(
    infra.backend,
    token,
    params.nodes.as_deref(),
    params.hsm_group.as_deref(),
    params.settings_hsm_group_name.as_deref(),
  )
  .await?;

  tracing::info!("Get boot parameters");

  infra.backend
    .get_bootparameters(token, &xname_vec)
    .await
    .map_err(|e| anyhow::anyhow!(e))
}

/// Delete boot parameters for the specified hosts.
pub async fn delete_boot_parameters(
  infra: &InfraContext<'_>,
  token: &str,
  hosts: Vec<String>,
) -> Result<(), Error> {
  let boot_parameters = BootParameters {
    hosts,
    macs: None,
    nids: None,
    params: String::new(),
    kernel: String::new(),
    initrd: String::new(),
    cloud_init: None,
  };

  infra
    .backend
    .delete_bootparameters(token, &boot_parameters)
    .await
    .context("Failed to delete boot parameters")?;

  Ok(())
}

/// Add (create) boot parameters for specified nodes.
pub async fn add_boot_parameters(
  infra: &InfraContext<'_>,
  token: &str,
  boot_parameters: &BootParameters,
) -> Result<(), Error> {
  infra
    .backend
    .add_bootparameters(token, boot_parameters)
    .await?;
  Ok(())
}

/// Typed parameters for updating boot parameters.
#[derive(serde::Deserialize)]
pub struct UpdateBootParametersParams {
  pub hosts: Vec<String>,
  pub nids: Option<Vec<u32>>,
  pub macs: Option<Vec<String>>,
  pub params: String,
  pub kernel: String,
  pub initrd: String,
}

/// Update boot parameters for specified nodes.
///
/// Validates target HSM membership, then updates BSS boot parameters.
pub async fn update_boot_parameters(
  infra: &InfraContext<'_>,
  token: &str,
  params: UpdateBootParametersParams,
) -> Result<(), Error> {
  validate_target_hsm_members(infra.backend, token, &params.hosts).await?;

  let boot_parameters = BootParameters {
    hosts: params.hosts,
    macs: params.macs,
    nids: params.nids,
    params: params.params,
    kernel: params.kernel,
    initrd: params.initrd,
    cloud_init: None,
  };

  tracing::debug!("new boot params: {:#?}", boot_parameters);

  infra
    .backend
    .update_bootparameters(token, &boot_parameters)
    .await?;

  Ok(())
}

/// Result of preparing boot configuration changes.
#[derive(serde::Serialize)]
pub struct BootConfigChangeset {
  pub xname_vec: Vec<String>,
  pub boot_param_vec: Vec<BootParameters>,
  pub image_vec: HashMap<String, Image>,
  pub need_restart: bool,
}

/// Prepare boot configuration changes (no side effects).
///
/// Resolves hosts, fetches current boot params, applies image/kernel
/// changes, and returns a changeset ready for user review and persistence.
pub async fn prepare_boot_config(
  infra: &InfraContext<'_>,
  token: &str,
  hosts_expression: &str,
  new_boot_image_id_opt: Option<&str>,
  new_boot_image_configuration_opt: Option<&str>,
  new_kernel_parameters_opt: Option<&str>,
) -> Result<BootConfigChangeset, Error> {
  let backend = infra.backend;

  let mut need_restart = false;

  // Convert user input to xname
  let xname_vec = common::node_ops::resolve_hosts_expression(
    backend,
    token,
    hosts_expression,
    false,
  )
  .await?;

  let mut current_node_boot_param_vec: Vec<BootParameters> = backend
    .get_bootparameters(token, &xname_vec)
    .await
    .context("Failed to get boot parameters")?;

  // Get new boot image
  let new_boot_image_opt = get_new_boot_image(
    backend,
    token,
    infra.shasta_base_url,
    infra.shasta_root_cert,
    new_boot_image_configuration_opt,
    new_boot_image_id_opt,
  )
  .await?;

  // IMPORTANT: ALWAYS SET KERNEL PARAMS BEFORE BOOT IMAGE
  if let Some(new_kernel_parameters) = new_kernel_parameters_opt {
    need_restart |= apply_kernel_params(
      &mut current_node_boot_param_vec,
      new_kernel_parameters,
    )?;
  }

  let mut image_vec = collect_boot_images(
    backend,
    token,
    &mut current_node_boot_param_vec,
    new_boot_image_opt,
    &mut need_restart,
  )
  .await?;

  // Update images that need SBPS projection
  if current_node_boot_param_vec
    .first()
    .context("No boot parameters found")?
    .is_root_kernel_param_iscsi_ready()
  {
    for image in image_vec.values_mut() {
      image.set_boot_image_iscsi_ready();
    }
  }

  Ok(BootConfigChangeset {
    xname_vec,
    boot_param_vec: current_node_boot_param_vec,
    image_vec,
    need_restart,
  })
}

/// Persist boot configuration changes.
pub async fn persist_boot_config(
  infra: &InfraContext<'_>,
  token: &str,
  changeset: &BootConfigChangeset,
  new_runtime_configuration_opt: Option<&str>,
) -> Result<(), Error> {
  tracing::info!("Persist changes");

  // Update boot params
  for boot_parameter in &changeset.boot_param_vec {
    tracing::debug!("Updating boot parameter:\n{:#?}", boot_parameter);
    let component_patch_rep = infra
      .backend
      .update_bootparameters(token, boot_parameter)
      .await;
    tracing::debug!(
      "Component boot parameters resp:\n{:#?}",
      component_patch_rep
    );
  }

  // Update desired configuration
  if let Some(new_runtime_configuration_name) = new_runtime_configuration_opt {
    println!(
      "Updating runtime configuration to '{}'",
      new_runtime_configuration_name
    );

    infra
      .backend
      .update_runtime_configuration(
        token,
        infra.shasta_base_url,
        infra.shasta_root_cert,
        &changeset.xname_vec,
        new_runtime_configuration_name,
        true,
      )
      .await
      .context("Error updating runtime configuration")?;

    // Update images
    for (_, image) in &changeset.image_vec {
      let image_id = image.id.clone().context("Image id is missing")?;
      let patch_image: PatchImage = image.clone().into();
      infra
        .backend
        .update_image(token, &image_id, &patch_image)
        .await?;
    }
  } else {
    tracing::info!("Runtime configuration does not change.");
  }

  Ok(())
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
    tracing::info!(
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

    let most_recent_image = image_vec
      .iter()
      .last()
      .context("No image found for configuration")?;

    tracing::debug!(
      "Boot image id related to configuration \
       '{}' found:\n{:#?}",
      new_boot_image_configuration,
      most_recent_image
    );

    Some(most_recent_image.clone())
  } else if let Some(boot_image_id) = new_boot_image_id_opt {
    tracing::info!("Boot image id '{}' provided", boot_image_id);
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
    tracing::info!(
      "Updating '{:?}' kernel parameters to '{}'",
      boot_parameter.hosts,
      new_kernel_parameters
    );

    let changed = boot_parameter.apply_kernel_params(new_kernel_parameters);
    any_changed = changed || any_changed;

    tracing::info!("need restart? {}", any_changed);

    let image_id = boot_parameter.try_get_boot_image_id().ok_or_else(|| {
      Error::msg(format!(
        "Could not get boot image id from boot \
         parameters for hosts: {:?}",
        boot_parameter.hosts
      ))
    })?;

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
        tracing::info!(
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
