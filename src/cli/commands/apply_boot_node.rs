use crate::{
  cli::commands::power_common::{self, PowerAction},
  common::{self, app_context::AppContext},
  service,
};

use anyhow::{Error, bail};

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

  if changeset.need_restart {
    if common::user_interaction::confirm(
      &format!(
        "This operation will modify the nodes \
         below:\n{}\nDo you want to continue?",
        changeset.xname_vec.join(", ")
      ),
      assume_yes,
    ) {
      tracing::info!("Continue",);
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
      serde_json::to_string_pretty(&changeset.boot_param_vec)
        .unwrap_or_default()
    );
    println!(
      "Images:\n{}",
      serde_json::to_string_pretty(&changeset.image_vec)
        .unwrap_or_default()
    );
    Ok(())
  } else {
    service::boot_parameters::persist_boot_config(
      &ctx.infra,
      token,
      &changeset,
      new_runtime_configuration_opt,
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
