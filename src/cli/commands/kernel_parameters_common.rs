use crate::common::{self, app_context::AppContext, audit};
use crate::service::kernel_parameters;
use anyhow::{Context, Error, bail};
use manta_backend_dispatcher::types::ims::Image;
use nodeset::NodeSet;
use std::collections::HashMap;

// Re-export types used by command files
pub use crate::service::kernel_parameters::KernelParamOperation;

/// Shared pipeline for all kernel parameter commands.
///
/// Expects `xname_vec` to be pre-resolved (caller handles node resolution).
///
/// Phases:
/// 1. Prepare changeset (service layer: fetch, mutate, detect changes)
/// 2. Handle SBPS image confirmation (CLI)
/// 3. User confirmation + dry-run guard (CLI)
/// 4. Persist changes (service layer)
/// 5. Audit (CLI)
/// 6. Reboot if needed (CLI)
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  xname_vec: &[String],
  operation: &KernelParamOperation<'_>,
  assume_yes: bool,
  do_not_reboot: bool,
  dry_run: bool,
) -> Result<(), Error> {
  let verb = operation.verb();
  let params = operation.params();

  log::info!("{} kernel parameters", verb);

  // Phase 1: Prepare changeset via service layer
  let changeset = kernel_parameters::prepare_kernel_params_changes(
    &ctx.infra,
    token,
    xname_vec,
    operation,
  )
  .await?;

  if !changeset.has_changes {
    bail!("No changes detected. Nothing to do");
  }

  // Phase 2: Handle SBPS image confirmation
  let mut images_to_project: HashMap<String, Image> = HashMap::new();
  for (image_id, mut image) in changeset.sbps_candidates.clone() {
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
      images_to_project.insert(image_id, image);
    } else {
      log::info!("User chose to not project the image through SBPS");
    }
  }

  // Phase 3: User confirmation
  let node_group: NodeSet = xname_vec
    .join(", ")
    .parse()
    .context("Failed to parse node list")?;

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

  // Phase 4: Persist changes (or print dry-run summary)
  if dry_run {
    println!("Dry-run enabled. No changes persisted into the system");
    println!(
      "Dry run mode. Would update kernel parameters below:\n{}",
      serde_json::to_string_pretty(&changeset.boot_params)
        .context("Failed to serialize boot parameters")?
    );
    if !images_to_project.is_empty() {
      println!(
        "Dry run mode. Would update images:\n{}",
        serde_json::to_string_pretty(&images_to_project)
          .context("Failed to serialize image map")?
      );
    }
  } else {
    log::info!("Persist changes");

    kernel_parameters::apply_kernel_params_changes(
      &ctx.infra,
      token,
      &changeset,
      &images_to_project,
    )
    .await?;
  }

  // Phase 5: Audit
  audit::maybe_send_audit_with_group_lookup(
    ctx.cli.kafka_audit_opt,
    ctx.infra.backend,
    token,
    format!("{} kernel parameters: {}", verb, params),
    xname_vec,
  )
  .await?;

  // Phase 6: Reboot if needed
  if !do_not_reboot && changeset.has_changes && !dry_run {
    log::info!("Restarting nodes");

    crate::cli::commands::power_common::exec_nodes(
      ctx,
      crate::cli::commands::power_common::PowerAction::Reset,
      &changeset.xnames_to_reboot.join(","),
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
