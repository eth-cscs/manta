use anyhow::{Context, Error};
use manta_backend_dispatcher::interfaces::bss::BootParametersTrait;
use manta_backend_dispatcher::interfaces::ims::ImsTrait;
use manta_backend_dispatcher::types::bss::BootParameters;
use manta_backend_dispatcher::types::ims::Image;
use std::collections::HashMap;

use crate::common;
use crate::common::app_context::InfraContext;

/// Typed parameters for fetching kernel boot parameters.
pub struct GetKernelParametersParams {
  pub hsm_group: Option<String>,
  pub nodes: Option<String>,
  pub settings_hsm_group_name: Option<String>,
}

/// Fetch kernel boot parameters for the specified nodes.
///
/// Resolves target nodes from HSM group or node list, then
/// fetches their BSS boot parameters.
pub async fn get_kernel_parameters(
  infra: &InfraContext<'_>,
  token: &str,
  params: &GetKernelParametersParams,
) -> Result<Vec<BootParameters>, Error> {
  let xname_vec = common::node_ops::resolve_target_nodes(
    infra.backend,
    token,
    params.nodes.as_deref(),
    params.hsm_group.as_deref(),
    params.settings_hsm_group_name.as_deref(),
  )
  .await?;

  let boot_parameter_vec = infra.backend
    .get_bootparameters(token, &xname_vec)
    .await
    .context("Could not get boot parameters")?;

  Ok(boot_parameter_vec)
}

/// Describes which kernel parameter mutation to apply.
pub enum KernelParamOperation<'a> {
  /// Add kernel parameters, optionally overwriting existing values.
  Add { params: &'a str, overwrite: bool },
  /// Replace all kernel parameters with the given value.
  Apply { params: &'a str },
  /// Remove the specified kernel parameters.
  Delete { params: &'a str },
}

impl<'a> KernelParamOperation<'a> {
  /// The verb used in log/user-facing messages.
  pub fn verb(&self) -> &'static str {
    match self {
      Self::Add { .. } => "Add",
      Self::Apply { .. } => "Apply",
      Self::Delete { .. } => "Delete",
    }
  }

  /// The raw kernel parameter string.
  pub fn params(&self) -> &str {
    match self {
      Self::Add { params, .. }
      | Self::Apply { params }
      | Self::Delete { params } => params,
    }
  }

  /// Apply the mutation to a single `BootParameters` entry.
  /// Returns `true` if the parameters were actually changed.
  fn mutate(
    &self,
    boot_parameter: &mut BootParameters,
  ) -> bool {
    match self {
      Self::Add { params, overwrite } => {
        boot_parameter.add_kernel_params(params, *overwrite)
      }
      Self::Apply { params } => {
        boot_parameter.apply_kernel_params(params)
      }
      Self::Delete { params } => {
        boot_parameter.delete_kernel_params(params)
      }
    }
  }

  /// The confirmation message shown to the user.
  pub fn confirm_message(&self) -> &'static str {
    match self {
      Self::Add { .. } => {
        "This operation will add the kernel parameters for the nodes below. \
         Please confirm to proceed"
      }
      Self::Apply { .. } => {
        "This operation will replace the kernel parameters for the nodes below. \
         Please confirm to proceed"
      }
      Self::Delete { .. } => {
        "This operation will delete the kernel parameters for the nodes below. \
         Please confirm to proceed"
      }
    }
  }

  /// Whether this operation should handle SBPS image projection.
  fn handles_sbps_images(&self) -> bool {
    match self {
      Self::Add { .. } | Self::Apply { .. } => true,
      Self::Delete { .. } => false,
    }
  }
}

/// Result of preparing kernel parameter mutations (before persistence).
pub struct KernelParamsChangeset {
  /// The mutated boot parameters ready to persist.
  pub boot_params: Vec<BootParameters>,
  /// Nodes that need rebooting.
  pub xnames_to_reboot: Vec<String>,
  /// Whether any changes were detected.
  pub has_changes: bool,
  /// SBPS images that need iSCSI projection (image_id -> Image).
  /// The CLI layer should confirm with the user before persisting these.
  pub sbps_candidates: Vec<(String, Image)>,
}

/// Fetch boot parameters, apply mutations, and return a changeset.
///
/// Does NOT persist anything — the caller decides whether to proceed.
pub async fn prepare_kernel_params_changes(
  infra: &InfraContext<'_>,
  token: &str,
  xname_vec: &[String],
  operation: &KernelParamOperation<'_>,
) -> Result<KernelParamsChangeset, Error> {
  let mut boot_params: Vec<BootParameters> = infra
    .backend
    .get_bootparameters(token, xname_vec)
    .await
    .context("Failed to get boot parameters")?;

  let mut has_changes = false;
  let mut xnames_to_reboot: Vec<String> = Vec::new();
  let mut seen_images: HashMap<String, bool> = HashMap::new();
  let mut sbps_candidates: Vec<(String, Image)> = Vec::new();

  for bp in &mut boot_params {
    let changed = operation.mutate(bp);
    if changed {
      has_changes = true;
      xnames_to_reboot.extend(bp.hosts.iter().cloned());
    }

    // Detect SBPS image candidates (add & apply only)
    if operation.handles_sbps_images() {
      if let Some(image_id) = bp.try_get_boot_image_id() {
        if !seen_images.contains_key(&image_id) {
          seen_images.insert(image_id.clone(), true);

          let image: Image = infra
            .backend
            .get_images(token, Some(&image_id))
            .await?
            .first()
            .context("No image found for the given image id")?
            .clone();

          if bp.is_root_kernel_param_iscsi_ready() {
            sbps_candidates.push((image_id, image));
          }
        }
      }
    }
  }

  Ok(KernelParamsChangeset {
    boot_params,
    xnames_to_reboot,
    has_changes,
    sbps_candidates,
  })
}

/// Persist the kernel parameter changes and optionally update SBPS images.
pub async fn apply_kernel_params_changes(
  infra: &InfraContext<'_>,
  token: &str,
  changeset: &KernelParamsChangeset,
  images_to_project: &HashMap<String, Image>,
) -> Result<(), Error> {
  // Update boot parameters
  for bp in &changeset.boot_params {
    infra
      .backend
      .update_bootparameters(token, bp)
      .await?;
  }

  // Update images projected through SBPS
  for (_, image) in images_to_project {
    infra
      .backend
      .update_image(
        token,
        image
          .id
          .clone()
          .context("Image has no id")?
          .as_str(),
        &image.clone().into(),
      )
      .await?;
  }

  Ok(())
}
