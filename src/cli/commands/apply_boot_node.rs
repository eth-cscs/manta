//! Implements the `manta apply boot nodes` command.

use crate::{
  cli::commands::power_common::{self, PowerAction},
  common::{self, app_context::AppContext},
  service,
};

use anyhow::{Error, bail};
use manta_backend_dispatcher::types::ims::PatchImage;
use serde_json::json;

/// Apply a boot configuration to specific nodes.
#[allow(clippy::too_many_arguments)]
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  new_boot_image_id_opt: Option<&str>,
  new_boot_image_configuration_opt: Option<&str>,
  new_runtime_configuration_opt: Option<&str>,
  new_kernel_parameters_opt: Option<&str>,
  hosts_expression: &str,
  assume_yes: bool,
  do_not_reboot: bool,
  disable: bool,
  dry_run: bool,
) -> Result<(), Error> {
  let changeset = service::boot_parameters::prepare_boot_config(
    &ctx.infra,
    token,
    hosts_expression,
    new_boot_image_id_opt,
    new_boot_image_configuration_opt,
    new_kernel_parameters_opt,
  )
  .await?;

  tracing::debug!(
    "boot params to update vec:\n{:#?}",
    changeset.boot_param_vec
  );

  let has_changes =
    changeset.need_restart || new_runtime_configuration_opt.is_some();

  if !has_changes {
    bail!("No changes detected. Nothing to do");
  }

  if changeset.need_restart {
    if !common::user_interaction::confirm(
      &format!(
        "This operation will modify the nodes \
         below:\n{}\nDo you want to continue?",
        changeset.xname_vec.join(", ")
      ),
      assume_yes,
    ) {
      bail!("Operation cancelled by user");
    }
    tracing::info!("Continue",);
  }

  if dry_run {
    println!("Dry-run enabled. No requests would be sent.");

    let base = ctx.infra.shasta_base_url;

    if changeset.need_restart {
      for boot_parameter in &changeset.boot_param_vec {
        println!();
        println!("Would send: PATCH {}/bss/boot/v1/bootparameters", base);
        println!(
          "Body:\n{}",
          serde_json::to_string_pretty(boot_parameter).unwrap_or_default()
        );
      }
    }

    if let Some(new_runtime_configuration) = new_runtime_configuration_opt {
      let component_list: Vec<serde_json::Value> = changeset
        .xname_vec
        .iter()
        .map(|xname| {
          json!({
            "id": xname,
            "desired_config": new_runtime_configuration,
            "enabled": !disable,
          })
        })
        .collect();
      println!();
      println!("Would send: PATCH {}/cfs/v3/components", base);
      println!(
        "Body:\n{}",
        serde_json::to_string_pretty(&component_list).unwrap_or_default()
      );

      if changeset.mutated_images {
        for (image_id, image) in &changeset.image_vec {
          let patch_image: PatchImage = image.clone().into();
          println!();
          println!("Would send: PATCH {}/ims/v3/images/{}", base, image_id);
          println!(
            "Body:\n{}",
            serde_json::to_string_pretty(&patch_image).unwrap_or_default()
          );
        }
      }
    }

    Ok(())
  } else {
    service::boot_parameters::persist_boot_config(
      &ctx.infra,
      token,
      &changeset,
      new_runtime_configuration_opt,
      Some(!disable),
    )
    .await?;

    if !do_not_reboot && changeset.need_restart {
      tracing::info!("Restarting nodes");
      let nodes = changeset.xname_vec;
      power_common::exec_nodes(
        ctx,
        PowerAction::Reset,
        &nodes.join(","),
        true,
        assume_yes,
        "table",
        token,
      )
      .await?;
    }

    Ok(())
  }
}
