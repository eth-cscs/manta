use anyhow::{Context, Error, bail};

use manta_backend_dispatcher::{
  interfaces::{
    bss::BootParametersTrait,
    hsm::{component::ComponentTrait, group::GroupTrait},
  },
  types::{self},
};

use crate::common::{
  self, app_context::AppContext, audit, authentication::get_api_token,
  authorization::get_groups_names_available, jwt_ops,
};
use nodeset::NodeSet;

/// Updates the kernel parameters for a set of nodes
/// reboots the nodes which kernel params have changed
pub async fn exec(
  ctx: &AppContext<'_>,
  hsm_group_name_arg_opt: Option<&String>,
  nodes: Option<&String>,
  kernel_params: &str,
  assume_yes: bool,
  do_not_reboot: bool,
  dry_run: bool,
) -> Result<(), Error> {
  let backend = ctx.backend;
  let site_name = ctx.site_name;
  let settings_hsm_group_name_opt = ctx.settings_hsm_group_name_opt;
  let kafka_audit_opt = ctx.kafka_audit_opt;
  let shasta_token = get_api_token(backend, site_name).await?;

  let hosts_expression: String = if hsm_group_name_arg_opt.is_some() {
    let hsm_group_name_vec = get_groups_names_available(
      backend,
      &shasta_token,
      hsm_group_name_arg_opt,
      settings_hsm_group_name_opt,
    )
    .await?;
    let hsm_members_rslt: Result<Vec<String>, _> = backend
      .get_member_vec_from_group_name_vec(&shasta_token, &hsm_group_name_vec)
      .await;
    match hsm_members_rslt {
      Ok(hsm_members) => hsm_members.join(","),
      Err(e) => {
        bail!("Could not fetch HSM groups members: {}", e);
      }
    }
  } else {
    nodes
      .cloned()
      .context("Neither HSM group nor nodes defined")?
  };

  let mut need_restart = false;
  println!("Delete kernel parameters");

  let mut xname_to_reboot_vec: Vec<String> = Vec::new();

  // Convert user input to xname
  let node_metadata_available_vec = backend
    .get_node_metadata_available(&shasta_token)
    .await
    .map_err(|e| {
      Error::msg(format!("Could not get node metadata. Reason:\n{e}"))
    })?;

  let xname_vec = common::node_ops::from_hosts_expression_to_xname_vec(
    &hosts_expression,
    false,
    node_metadata_available_vec,
  )
  .await
  .map_err(|e| {
    Error::msg(format!(
      "Could not convert user input to list of xnames. Reason:\n{e}"
    ))
  })?;

  let mut current_node_boot_params_vec: Vec<types::bss::BootParameters> =
    backend
      .get_bootparameters(&shasta_token, &xname_vec)
      .await
      .context("Failed to get boot parameters")?;

  let node_group: NodeSet = xname_vec
    .join(", ")
    .parse()
    .context("Failed to parse node list")?;

  println!(
    "Delete kernel params:\n{:?}\nFor nodes:\n{:?}",
    kernel_params,
    xname_vec.join(", ")
  );

  for boot_parameter in &mut current_node_boot_params_vec {
    log::info!(
      "Delete '{:?}' kernel parameters to '{}'",
      kernel_params,
      node_group
    );

    let kernel_params_changed =
      boot_parameter.delete_kernel_params(kernel_params);
    need_restart = kernel_params_changed || need_restart;

    if kernel_params_changed {
      need_restart = true;
      xname_to_reboot_vec.extend(boot_parameter.hosts.iter().cloned());
    }
  }

  if need_restart {
    println!(
      "Delete kernel params:\n{:?}\nFor nodes:\n{:?}",
      kernel_params,
      node_group.to_string()
    );
    if !common::user_interaction::confirm(
      "This operation will delete the kernel parameters for the nodes below. Please confirm to proceed",
      assume_yes,
    ) {
      bail!("Operation canceled by the user.");
    }
  } else {
    bail!("No changes detected. Nothing to do");
  }

  log::info!("need restart? {}", need_restart);

  if dry_run {
    println!("Dry-run enabled. No changes persisted into the system");
    println!(
      "Dry run mode. Would update kernel parameters below: {}",
      serde_json::to_string_pretty(&current_node_boot_params_vec)
        .context("Failed to serialize boot parameters")?
    );
  } else {
    log::info!("Persist changes");

    // Update boot parameters
    for boot_parameter in current_node_boot_params_vec {
      log::info!(
        "Delete '{}' kernel parameters to '{:?}'",
        kernel_params,
        boot_parameter.hosts,
      );

      backend
        .update_bootparameters(&shasta_token, &boot_parameter)
        .await?;
    }
  }

  // Audit
  if let Some(kafka_audit) = kafka_audit_opt {
    let username = jwt_ops::get_name(&shasta_token)
      .context("Failed to get username from token")?;
    let user_id = jwt_ops::get_preferred_username(&shasta_token)
      .context("Failed to get preferred username from token")?;

    // FIXME: We should not need to make this call here but at the beginning of the method as a
    // prerequisite
    let xnames: Vec<&str> =
      xname_vec.iter().map(|xname| xname.as_str()).collect();

    let group_map_vec = backend
      .get_group_map_and_filter_by_member_vec(&shasta_token, &xnames)
      .await?;

    let msg_json = serde_json::json!({
      "user": {"id": user_id, "name": username},
      "host": {"hostname": xname_vec},
      "group": group_map_vec
        .keys()
        .collect::<Vec<&String>>(),
      "message": format!(
        "Delete kernel parameters: {}",
        kernel_params
      ),
    });

    audit::send_audit_message(kafka_audit, msg_json).await;
  }

  // Reboot if needed
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
