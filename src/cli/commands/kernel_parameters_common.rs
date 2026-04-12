use crate::common::{self, app_context::AppContext, audit};
use anyhow::{Context, Error, bail};

use manta_backend_dispatcher::{
  interfaces::{bss::BootParametersTrait, ims::ImsTrait},
  types::{self, ims::Image},
};
use nodeset::NodeSet;
use std::collections::HashMap;

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
  fn verb(&self) -> &'static str {
    match self {
      Self::Add { .. } => "Add",
      Self::Apply { .. } => "Apply",
      Self::Delete { .. } => "Delete",
    }
  }

  /// The raw kernel parameter string.
  fn params(&self) -> &str {
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
    boot_parameter: &mut types::bss::BootParameters,
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
  fn confirm_message(&self) -> &'static str {
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

/// Shared pipeline for all kernel parameter commands.
///
/// Expects `xname_vec` to be pre-resolved (caller handles node resolution).
///
/// Phases:
/// 1. Fetch boot parameters & parse NodeSet
/// 2. Loop: mutate boot params, track restarts, handle SBPS images
/// 3. User confirmation + dry-run guard
/// 4. Persist changes
/// 5. Audit
/// 6. Reboot if needed
pub async fn exec(
  ctx: &AppContext<'_>,
  xname_vec: &[String],
  operation: &KernelParamOperation<'_>,
  assume_yes: bool,
  do_not_reboot: bool,
  dry_run: bool,
) -> Result<(), Error> {
  let backend = ctx.backend;
  let site_name = ctx.site_name;
  let kafka_audit_opt = ctx.kafka_audit_opt;

  let shasta_token =
    crate::common::authentication::get_api_token(backend, site_name)
      .await?;

  let verb = operation.verb();
  let params = operation.params();

  log::info!("{} kernel parameters", verb);

  let mut need_restart = false;
  let mut xname_to_reboot_vec: Vec<String> = Vec::new();
  let mut image_map: HashMap<String, Image> = HashMap::new();

  // Fetch current boot parameters
  let mut current_node_boot_params_vec: Vec<types::bss::BootParameters> =
    backend
      .get_bootparameters(&shasta_token, xname_vec)
      .await
      .context("Failed to get boot parameters")?;

  let node_group: NodeSet = xname_vec
    .join(", ")
    .parse()
    .context("Failed to parse node list")?;

  // Phase 2: Mutate boot parameters
  for boot_parameter in &mut current_node_boot_params_vec {
    log::info!(
      "{} '{}' kernel parameters to '{}'",
      verb,
      params,
      node_group,
    );

    let kernel_params_changed = operation.mutate(boot_parameter);

    if kernel_params_changed {
      need_restart = true;
      xname_to_reboot_vec.extend(boot_parameter.hosts.iter().cloned());
    }

    // SBPS image handling (add & apply only)
    if operation.handles_sbps_images() {
      let image_id_opt = boot_parameter.try_get_boot_image_id();
      if let Some(image_id) = image_id_opt {
        #[allow(clippy::map_entry)]
        if !image_map.contains_key(&image_id) {
          let mut image: Image = backend
            .get_images(&shasta_token, Some(&image_id))
            .await?
            .first()
            .context("No image found for the given image id")?
            .clone();

          if boot_parameter.is_root_kernel_param_iscsi_ready() {
            if common::user_interaction::confirm(
              "Kernel parameters using SBPS/iSCSI. \
               Do you want to project the boot image \
               through SBPS?",
              assume_yes,
            ) {
              log::info!(
                "Setting 'sbps-project' metadata to \
                 'true' for image id '{}'",
                image_id
              );

              image.set_boot_image_iscsi_ready();

              log::debug!("Image:\n{:#?}", image);

              image_map.insert(image_id, image.clone());
            } else {
              log::info!(
                "User chose to not project the \
                 image through SBPS"
              );
            }
          }
        }
      }
    }
  }

  // Phase 3: User confirmation
  if need_restart {
    println!(
      "{} kernel params:\n{:?}\nFor nodes:\n{:?}",
      verb,
      params,
      node_group.to_string()
    );

    if !common::user_interaction::confirm(
      operation.confirm_message(),
      assume_yes,
    ) {
      bail!("Operation cancelled by user");
    }
  } else {
    bail!("No changes detected. Nothing to do");
  }

  log::info!("need restart? {}", need_restart);

  // Phase 4: Persist changes (or print dry-run summary)
  if dry_run {
    println!("Dry-run enabled. No changes persisted into the system");
    println!(
      "Dry run mode. Would update kernel parameters below:\n{}",
      serde_json::to_string_pretty(&current_node_boot_params_vec)
        .context("Failed to serialize boot parameters")?
    );
    if !image_map.is_empty() {
      println!(
        "Dry run mode. Would update images:\n{}",
        serde_json::to_string_pretty(&image_map)
          .context("Failed to serialize image map")?
      );
    }
  } else {
    log::info!("Persist changes");

    // Update boot parameters
    for boot_parameter in current_node_boot_params_vec {
      log::info!(
        "{} '{}' kernel parameters to '{:?}'",
        verb,
        params,
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

  // Phase 5: Audit
  audit::maybe_send_audit_with_group_lookup(
    kafka_audit_opt,
    backend,
    &shasta_token,
    format!("{} kernel parameters: {}", verb, params),
    xname_vec,
  )
  .await?;

  // Phase 6: Reboot if needed
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

/// Re-export the shared node resolution function so that
/// existing callers of `kernel_parameters_common::resolve_target_nodes`
/// continue to compile without changes.
pub use crate::common::node_ops::resolve_target_nodes;
