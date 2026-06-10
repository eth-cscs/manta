//! Kernel boot parameter mutations (add, apply, delete) with SBPS iSCSI image projection.

use manta_backend_dispatcher::error::Error;
use manta_backend_dispatcher::interfaces::bss::BootParametersTrait;
use manta_backend_dispatcher::interfaces::ims::ImsTrait;
use manta_backend_dispatcher::types::bss::BootParameters;
use manta_backend_dispatcher::types::ims::Image;
use std::collections::HashMap;

use crate::server::common::app_context::InfraContext;
use crate::service::authorization::validate_user_group_members_access;
use crate::service::node_ops;
pub use manta_shared::types::params::kernel_parameters::GetKernelParametersParams;

/// Fetch BSS kernel parameters for the targets described by `params`.
///
/// Targets are resolved through [`node_ops::resolve_target_nodes`]
/// (host expression → `group_name` → `settings_group_name` fallback
/// from `cli.toml`). The caller's access to every resolved xname is
/// validated before the BSS query runs.
pub async fn get_kernel_parameters(
  infra: &InfraContext<'_>,
  token: &str,
  params: &GetKernelParametersParams,
) -> Result<Vec<BootParameters>, Error> {
  let xname_vec = node_ops::resolve_target_nodes(
    infra,
    token,
    params.nodes.as_deref(),
    params.group_name.as_deref(),
    params.settings_group_name.as_deref(),
  )
  .await?;

  validate_user_group_members_access(infra, token, &xname_vec).await?;

  let boot_parameter_vec =
    infra.backend.get_bootparameters(token, &xname_vec).await?;

  Ok(boot_parameter_vec)
}

/// Describes which kernel parameter mutation to apply.
pub(crate) enum KernelParamOperation<'a> {
  /// Add kernel parameters, optionally overwriting existing values.
  Add {
    /// Space-separated `key=value` pairs to add.
    params: &'a str,
    /// When true, replace existing parameters with the same key
    /// instead of skipping them.
    overwrite: bool,
  },
  /// Replace all kernel parameters with the given value.
  Apply {
    /// Space-separated `key=value` pairs that fully replace the
    /// existing parameter set.
    params: &'a str,
  },
  /// Remove the specified kernel parameters.
  Delete {
    /// Space-separated parameter names (or `key=value` pairs) to
    /// remove.
    params: &'a str,
  },
}

impl<'a> KernelParamOperation<'a> {
  /// Apply the mutation to a single `BootParameters` entry.
  /// Returns `true` if the parameters were actually changed.
  fn mutate(&self, boot_parameter: &mut BootParameters) -> bool {
    match self {
      Self::Add { params, overwrite } => {
        boot_parameter.add_kernel_params(params, *overwrite)
      }
      Self::Apply { params } => boot_parameter.apply_kernel_params(params),
      Self::Delete { params } => boot_parameter.delete_kernel_params(params),
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

/// Compute the kernel-parameter mutation as a
/// [`KernelParamsChangeset`] without writing anything.
///
/// Pulls the current BSS records for `xname_vec`, applies `operation`
/// to each in memory, and tracks which xnames actually changed so the
/// caller can target the reboot list precisely. For `Add`/`Apply`,
/// each unique boot-image referenced by a changed record is
/// inspected once: if its root kernel-parameters look iSCSI-ready it
/// is appended to `sbps_candidates` so the caller can decide whether
/// to project it through SBPS.
pub(crate) async fn prepare_kernel_params_changes(
  infra: &InfraContext<'_>,
  token: &str,
  xname_vec: &[String],
  operation: &KernelParamOperation<'_>,
) -> Result<KernelParamsChangeset, Error> {
  let mut boot_params: Vec<BootParameters> =
    infra.backend.get_bootparameters(token, xname_vec).await?;

  let mut has_changes = false;
  let mut xnames_to_reboot: Vec<String> = Vec::new();

  // First pass: apply the in-memory mutation and gather, in the
  // order they first appear, the unique image ids referenced by
  // iSCSI-ready boot parameters. The image fetches happen in the
  // second pass below — the previous code fetched serially inside
  // the loop (N HTTPS round-trips for N distinct boot images on a
  // cluster-scale write).
  let handles_sbps = operation.handles_sbps_images();
  let mut sbps_image_ids: Vec<String> = Vec::new();
  let mut seen_image_ids: std::collections::HashSet<String> =
    std::collections::HashSet::new();

  for bp in &mut boot_params {
    let changed = operation.mutate(bp);
    if changed {
      has_changes = true;
      xnames_to_reboot.extend(bp.hosts.iter().cloned());
    }

    if handles_sbps
      && bp.is_root_kernel_param_iscsi_ready()
      && let Some(image_id) = bp.try_get_boot_image_id()
      && seen_image_ids.insert(image_id.clone())
    {
      sbps_image_ids.push(image_id);
    }
  }

  // Second pass: resolve each unique image id in parallel.
  let sbps_candidates: Vec<(String, Image)> = if sbps_image_ids.is_empty() {
    Vec::new()
  } else {
    futures::future::try_join_all(sbps_image_ids.into_iter().map(|id| async {
      let image = infra
        .backend
        .get_images(token, Some(id.as_str()))
        .await?
        .first()
        .ok_or_else(|| {
          Error::NotFound(format!("No image found for image id '{id}'"))
        })?
        .clone();
      Ok::<_, Error>((id, image))
    }))
    .await?
  };

  Ok(KernelParamsChangeset {
    boot_params,
    xnames_to_reboot,
    has_changes,
    sbps_candidates,
  })
}

/// Write a previously prepared [`KernelParamsChangeset`] back to BSS,
/// and patch any SBPS images supplied in `images_to_project`.
///
/// Access to every reboot-target xname is re-validated before the
/// first backend write. `images_to_project` is normally built by
/// [`build_images_to_project`]; pass an empty map (as the delete path
/// does) to skip SBPS projection entirely.
pub async fn apply_kernel_params_changes(
  infra: &InfraContext<'_>,
  token: &str,
  changeset: &KernelParamsChangeset,
  images_to_project: &HashMap<String, Image>,
) -> Result<(), Error> {
  validate_user_group_members_access(infra, token, &changeset.xnames_to_reboot)
    .await?;

  // Update boot parameters
  for bp in &changeset.boot_params {
    infra.backend.update_bootparameters(token, bp).await?;
  }

  // Update images projected through SBPS
  for image in images_to_project.values() {
    infra
      .backend
      .update_image(
        token,
        image
          .id
          .clone()
          .ok_or_else(|| Error::MissingField("Image has no id".to_string()))?
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
    KernelParamOperation::Add {
      params,
      overwrite: false,
    }
  }
  fn add_overwrite(params: &str) -> KernelParamOperation<'_> {
    KernelParamOperation::Add {
      params,
      overwrite: true,
    }
  }
  fn apply(params: &str) -> KernelParamOperation<'_> {
    KernelParamOperation::Apply { params }
  }
  fn delete(params: &str) -> KernelParamOperation<'_> {
    KernelParamOperation::Delete { params }
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
}
