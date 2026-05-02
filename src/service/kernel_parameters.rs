use manta_backend_dispatcher::error::Error;
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
    .await?;

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
#[derive(serde::Serialize)]
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
    .await?;

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
            .ok_or_else(|| Error::Message("No image found for the given image id".to_string()))?
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
          .ok_or_else(|| Error::Message("Image has no id".to_string()))?
          .as_str(),
        &image.clone().into(),
      )
      .await?;
  }

  Ok(())
}

/// Build the SBPS images-to-project map from a kernel params changeset.
///
/// Marks each candidate image as iSCSI-ready and returns the projection
/// map. Returns an empty map when `project_sbps` is false.
pub fn build_images_to_project(
  changeset: &KernelParamsChangeset,
  project_sbps: bool,
) -> HashMap<String, Image> {
  if !project_sbps {
    return HashMap::new();
  }
  changeset
    .sbps_candidates
    .iter()
    .map(|(id, img)| {
      let mut img = img.clone();
      img.set_boot_image_iscsi_ready();
      (id.clone(), img)
    })
    .collect()
}

#[cfg(test)]
mod tests {
  use super::*;

  fn add(params: &str) -> KernelParamOperation<'_> {
    KernelParamOperation::Add { params, overwrite: false }
  }
  fn add_overwrite(params: &str) -> KernelParamOperation<'_> {
    KernelParamOperation::Add { params, overwrite: true }
  }
  fn apply(params: &str) -> KernelParamOperation<'_> {
    KernelParamOperation::Apply { params }
  }
  fn delete(params: &str) -> KernelParamOperation<'_> {
    KernelParamOperation::Delete { params }
  }

  #[test]
  fn verb_returns_correct_string() {
    assert_eq!(add("quiet").verb(), "Add");
    assert_eq!(apply("quiet").verb(), "Apply");
    assert_eq!(delete("quiet").verb(), "Delete");
  }

  #[test]
  fn params_returns_the_string() {
    assert_eq!(add("quiet crashkernel=auto").params(), "quiet crashkernel=auto");
    assert_eq!(apply("console=ttyS0").params(), "console=ttyS0");
    assert_eq!(delete("splash").params(), "splash");
  }

  #[test]
  fn overwrite_flag_preserved() {
    match add_overwrite("x") {
      KernelParamOperation::Add { overwrite, .. } => assert!(overwrite),
      _ => panic!("wrong variant"),
    }
    match add("x") {
      KernelParamOperation::Add { overwrite, .. } => assert!(!overwrite),
      _ => panic!("wrong variant"),
    }
  }

  #[test]
  fn handles_sbps_images_only_for_add_and_apply() {
    assert!(add("quiet").handles_sbps_images());
    assert!(apply("quiet").handles_sbps_images());
    assert!(!delete("quiet").handles_sbps_images());
  }

  #[test]
  fn confirm_messages_are_nonempty() {
    assert!(!add("x").confirm_message().is_empty());
    assert!(!apply("x").confirm_message().is_empty());
    assert!(!delete("x").confirm_message().is_empty());
  }

  #[test]
  fn confirm_messages_are_distinct() {
    assert_ne!(add("x").confirm_message(), apply("x").confirm_message());
    assert_ne!(add("x").confirm_message(), delete("x").confirm_message());
    assert_ne!(apply("x").confirm_message(), delete("x").confirm_message());
  }
}
