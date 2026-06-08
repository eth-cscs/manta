//! BSS boot parameter queries, changeset preparation, and persistence.

use manta_backend_dispatcher::{
  error::Error,
  types::{
    Group,
    bss::BootParameters,
    ims::{Image, PatchImage},
  },
};
use std::collections::HashMap;

use crate::server::common::app_context::InfraContext;
use crate::service::authorization::validate_user_group_members_access;
use crate::service::ims_ops::get_image_vec_related_cfs_configuration_name;
use crate::service::node_ops;
pub use manta_shared::types::params::boot_parameters::{
  GetBootParametersParams, UpdateBootParametersParams,
};

/// Fetch BSS boot parameters for the resolved target nodes.
///
/// Targets are resolved from `params` in the
/// [`node_ops::resolve_target_nodes`] priority order (host
/// expression, then `group_name`, then the `settings_group_name`
/// fallback from `cli.toml`). An empty resolution returns `BadRequest`
/// rather than silently querying nothing; otherwise the caller's
/// access to every resolved xname is validated before hitting the
/// backend.
pub async fn get_boot_parameters(
  infra: &InfraContext<'_>,
  token: &str,
  params: &GetBootParametersParams,
) -> Result<Vec<BootParameters>, Error> {
  tracing::info!("Get boot parameters");

  let xname_vec = node_ops::resolve_target_nodes(
    infra,
    token,
    params.host_expression.as_deref(),
    params.group_name.as_deref(),
    params.settings_group_name.as_deref(),
  )
  .await?;

  validate_user_group_members_access(infra, token, &xname_vec).await?;

  if xname_vec.is_empty() {
    return Err(Error::BadRequest(
      "The list of nodes to operate is empty. Nothing to do".to_string(),
    ));
  }

  validate_user_group_members_access(infra, token, &xname_vec).await?;

  infra.get_bootparameters(token, &xname_vec).await
}

/// Remove the BSS boot-parameter record for each host in `hosts`.
///
/// The caller's access to every host is validated before the delete
/// is dispatched. The constructed `BootParameters` carries only the
/// host list — BSS keys deletions by host, so the other fields are
/// intentionally empty.
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

  validate_user_group_members_access(infra, token, &boot_parameters.hosts)
    .await?;

  infra.delete_bootparameters(token, &boot_parameters).await
}

/// Create a new BSS boot-parameter record for `boot_parameters.hosts`.
///
/// The caller's access to every listed host is validated before the
/// create is dispatched. Use [`update_boot_parameters`] when modifying
/// an existing record.
pub async fn add_boot_parameters(
  infra: &InfraContext<'_>,
  token: &str,
  boot_parameters: &BootParameters,
) -> Result<(), Error> {
  validate_user_group_members_access(infra, token, &boot_parameters.hosts)
    .await?;

  infra.add_bootparameters(token, boot_parameters).await
}

/// Replace the BSS boot-parameter record for `params.hosts` with the
/// values carried in `params`.
///
/// The caller's access to every host is validated first. `cloud_init`
/// is intentionally left unset: the update endpoint accepts only the
/// core boot fields (`params`, `kernel`, `initrd`, `macs`, `nids`).
pub async fn update_boot_parameters(
  infra: &InfraContext<'_>,
  token: &str,
  params: UpdateBootParametersParams,
) -> Result<(), Error> {
  validate_user_group_members_access(infra, token, &params.hosts).await?;

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

  infra.update_bootparameters(token, &boot_parameters).await
}

/// Result of preparing boot configuration changes.
#[derive(serde::Serialize)]
pub(crate) struct BootConfigChangeset {
  /// Resolved target xnames.
  pub xname_vec: Vec<String>,
  /// Updated BSS boot parameter records, ready to persist.
  pub boot_param_vec: Vec<BootParameters>,
  /// IMS images referenced by the new boot config, keyed by image ID.
  pub image_vec: HashMap<String, Image>,
  /// Whether nodes need a reboot to apply the new parameters.
  pub need_restart: bool,
}

/// Build a [`BootConfigChangeset`] describing what the requested
/// boot-config edit would write, without persisting anything.
///
/// Resolves `hosts_expression`, fetches the current boot parameters,
/// applies new kernel parameters (always first — the boot image patch
/// reads the updated kernel-params), then attaches the new boot image
/// either by id or by latest image for the named CFS configuration.
/// The iSCSI-ready flag is propagated from the existing kernel
/// parameters onto each affected image.
///
/// The split between this and [`persist_boot_config`] lets callers
/// confirm the planned change with the user before any backend write.
pub(crate) async fn prepare_boot_config(
  infra: &InfraContext<'_>,
  token: &str,
  hosts_expression: &str,
  new_boot_image_id_opt: Option<&str>,
  new_boot_image_configuration_opt: Option<&str>,
  new_kernel_parameters_opt: Option<&str>,
) -> Result<BootConfigChangeset, Error> {
  let mut need_restart = false;

  let xname_vec = node_ops::from_user_hosts_expression_to_xname_vec(
    infra,
    token,
    hosts_expression,
    false,
  )
  .await?;

  // Gate before the BSS / IMS lookups below: the dry-run handler
  // path returns the changeset (boot params + referenced images)
  // directly to the caller, so an unauthorized resolution would leak
  // state. `persist_boot_config` re-checks at write time.
  validate_user_group_members_access(infra, token, &xname_vec).await?;

  let mut current_node_boot_param_vec: Vec<BootParameters> =
    infra.get_bootparameters(token, &xname_vec).await?;

  let new_boot_image_opt = get_new_boot_image(
    infra,
    token,
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
    infra,
    token,
    &mut current_node_boot_param_vec,
    new_boot_image_opt,
    &mut need_restart,
  )
  .await?;

  if current_node_boot_param_vec
    .first()
    .ok_or_else(|| Error::NotFound("No boot parameters found".to_string()))?
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

/// Write a [`BootConfigChangeset`] previously built by
/// [`prepare_boot_config`].
///
/// Validates access to every xname in the changeset, writes each
/// updated BSS record, then — if `new_runtime_configuration_opt` is
/// supplied — points the runtime configuration at it and patches the
/// referenced images so they boot under the new CFS configuration.
pub(crate) async fn persist_boot_config(
  infra: &InfraContext<'_>,
  token: &str,
  changeset: &BootConfigChangeset,
  new_runtime_configuration_opt: Option<&str>,
) -> Result<(), Error> {
  tracing::info!("Persist changes");

  validate_user_group_members_access(infra, token, &changeset.xname_vec)
    .await?;

  for boot_parameter in &changeset.boot_param_vec {
    tracing::debug!("Updating boot parameter:\n{:#?}", boot_parameter);
    let component_patch_rep =
      infra.update_bootparameters(token, boot_parameter).await;
    tracing::debug!(
      "Component boot parameters resp:\n{:#?}",
      component_patch_rep
    );
  }

  if let Some(new_runtime_configuration_name) = new_runtime_configuration_opt {
    tracing::info!(
      "Updating runtime configuration to '{new_runtime_configuration_name}'"
    );

    infra
      .update_runtime_configuration(
        token,
        &changeset.xname_vec,
        new_runtime_configuration_name,
        true,
      )
      .await?;

    for image in changeset.image_vec.values() {
      let image_id = image.id.clone().ok_or_else(|| {
        Error::MissingField("Image id is missing".to_string())
      })?;
      let patch_image: PatchImage = image.clone().into();
      infra.update_image(token, &image_id, &patch_image).await?;
    }
  } else {
    tracing::info!("Runtime configuration does not change.");
  }

  Ok(())
}

async fn get_new_boot_image(
  infra: &InfraContext<'_>,
  shasta_token: &str,
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
      infra,
      shasta_token,
      new_boot_image_configuration.to_string(),
    )
    .await?;

    if image_vec.is_empty() {
      return Err(Error::NotFound(format!(
        "No boot image found for configuration '{new_boot_image_configuration}'"
      )));
    }

    infra.filter_images(&mut image_vec)?;

    let most_recent_image = image_vec.iter().last().ok_or_else(|| {
      Error::NotFound("No image found for configuration".to_string())
    })?;

    tracing::debug!(
      "Boot image id related to configuration '{}' found:\n{:#?}",
      new_boot_image_configuration,
      most_recent_image
    );

    Some(most_recent_image.clone())
  } else if let Some(boot_image_id) = new_boot_image_id_opt {
    tracing::info!("Boot image id '{}' provided", boot_image_id);
    let image_in_csm_vec = infra
      .get_images(shasta_token, new_boot_image_id_opt)
      .await?;

    if image_in_csm_vec.is_empty() {
      return Err(Error::NotFound(format!(
        "Boot image id '{boot_image_id}' not found"
      )));
    }

    image_in_csm_vec.first().cloned()
  } else {
    None
  };

  Ok(new_boot_image)
}

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
      Error::MissingField(format!(
        "Could not get boot image id from boot parameters for hosts: {:?}",
        boot_parameter.hosts
      ))
    })?;

    let _ = boot_parameter
      .update_boot_image(&image_id, &boot_parameter.get_boot_image_etag())?;
  }

  Ok(any_changed)
}

async fn collect_boot_images(
  infra: &InfraContext<'_>,
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
      .ok_or_else(|| {
        Error::MissingField("New boot image id is missing".to_string())
      })?
      .clone();

    let new_boot_image_etag = new_boot_image
      .link
      .as_ref()
      .and_then(|link| link.etag.as_ref())
      .ok_or_else(|| {
        Error::MissingField("New boot image etag is missing".to_string())
      })?;

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
          .update_boot_image(&new_boot_image_id, new_boot_image_etag)?;
      }
      *need_restart = true;
    }
  } else {
    for boot_parameter in boot_param_vec.iter() {
      let boot_image_id =
        boot_parameter.try_get_boot_image_id().ok_or_else(|| {
          Error::MissingField(format!(
            "Could not get boot image id from boot parameters for hosts: {:?}",
            boot_parameter.hosts
          ))
        })?;

      let boot_image = infra
        .get_images(shasta_token, Some(boot_image_id.as_str()))
        .await?
        .first()
        .ok_or_else(|| {
          Error::NotFound("No image found for boot image id".to_string())
        })?
        .clone();

      image_vec.insert(boot_image_id, boot_image);
    }
  }

  Ok(image_vec)
}

/// Return the subset of `boot_parameter_vec` whose `hosts` list
/// includes at least one member of the groups in `group_available_vec`.
/// Used by `service::image` to scope image-deletion safety checks to
/// the boot parameters that name nodes the caller can actually see.
pub fn get_restricted_boot_parameters(
  group_available_vec: &[Group],
  boot_parameter_vec: &[BootParameters],
) -> Vec<BootParameters> {
  let group_members: Vec<String> = group_available_vec
    .iter()
    .flat_map(Group::get_members)
    .collect();

  boot_parameter_vec
    .iter()
    .filter(|boot_param| {
      group_members
        .iter()
        .any(|gma| boot_param.hosts.contains(gma))
    })
    .cloned()
    .collect::<Vec<BootParameters>>()
}

#[cfg(test)]
mod tests {
  use super::*;
  use manta_backend_dispatcher::types::Member;

  /// Helper: create a Group with given label and member xnames.
  fn make_group(label: &str, member_ids: Vec<&str>) -> Group {
    Group {
      label: label.to_string(),
      description: None,
      tags: None,
      members: Some(Member {
        ids: Some(member_ids.into_iter().map(String::from).collect()),
      }),
      exclusive_group: None,
    }
  }

  /// Helper: create a BootParameters with given hosts.
  fn make_boot_params(hosts: Vec<&str>) -> BootParameters {
    BootParameters {
      hosts: hosts.into_iter().map(String::from).collect(),
      ..Default::default()
    }
  }

  #[test]
  fn filters_boot_params_by_group_membership() {
    let groups =
      vec![make_group("grp1", vec!["x1000c0s0b0n0", "x1000c0s0b0n1"])];
    let boot_params = vec![
      make_boot_params(vec!["x1000c0s0b0n0"]),
      make_boot_params(vec!["x9999c0s0b0n0"]),
      make_boot_params(vec!["x1000c0s0b0n1"]),
    ];
    let result = get_restricted_boot_parameters(&groups, &boot_params);
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].hosts, vec!["x1000c0s0b0n0"]);
    assert_eq!(result[1].hosts, vec!["x1000c0s0b0n1"]);
  }

  #[test]
  fn returns_empty_when_no_group_members_match() {
    let groups = vec![make_group("grp1", vec!["x1000c0s0b0n0"])];
    let boot_params = vec![make_boot_params(vec!["x9999c0s0b0n0"])];
    let result = get_restricted_boot_parameters(&groups, &boot_params);
    assert!(result.is_empty());
  }

  #[test]
  fn returns_empty_when_groups_are_empty() {
    let boot_params = vec![make_boot_params(vec!["x1000c0s0b0n0"])];
    let result = get_restricted_boot_parameters(&[], &boot_params);
    assert!(result.is_empty());
  }

  #[test]
  fn returns_empty_when_boot_params_are_empty() {
    let groups = vec![make_group("grp1", vec!["x1000c0s0b0n0"])];
    let result = get_restricted_boot_parameters(&groups, &[]);
    assert!(result.is_empty());
  }

  #[test]
  fn aggregates_members_across_multiple_groups() {
    let groups = vec![
      make_group("grp1", vec!["x1000c0s0b0n0"]),
      make_group("grp2", vec!["x2000c0s0b0n0"]),
    ];
    let boot_params = vec![
      make_boot_params(vec!["x1000c0s0b0n0"]),
      make_boot_params(vec!["x2000c0s0b0n0"]),
      make_boot_params(vec!["x3000c0s0b0n0"]),
    ];
    let result = get_restricted_boot_parameters(&groups, &boot_params);
    assert_eq!(result.len(), 2);
  }
}
