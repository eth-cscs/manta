use crate::cli::commands::{
  add_boot_parameters, add_group, add_hw_component_cluster,
  add_kernel_parameters, add_node, add_redfish_endpoint,
};
use crate::common::app_context::AppContext;
use crate::common::{
  authentication::get_api_token, authorization::get_groups_names_available,
};
use anyhow::{Context, Error, bail};
use clap::ArgMatches;
use std::path::PathBuf;

/// Dispatch `manta add` subcommands (node, group,
/// kernel-parameters, hardware cluster, boot-parameters,
/// redfish-endpoint).
pub async fn handle_add(
  cli_add: &ArgMatches,
  ctx: &AppContext<'_>,
) -> Result<(), Error> {
  let token = get_api_token(ctx.infra.backend, ctx.infra.site_name).await?;

  if let Some(cli_add_node) = cli_add.subcommand_matches("node") {
    let id = cli_add_node
      .get_one::<String>("id")
      .context("'id' argument is mandatory")?;
    let group = cli_add_node
      .get_one::<String>("group")
      .context("'group' argument is mandatory")?;
    let hardware_file_opt = cli_add_node.get_one::<PathBuf>("hardware");

    let arch_opt = cli_add_node.get_one::<String>("arch").cloned();
    let enabled = cli_add_node
      .get_one::<bool>("disabled")
      .cloned()
      .unwrap_or(true);

    add_node::exec(
      ctx,
      &token,
      id,
      group,
      enabled,
      arch_opt,
      hardware_file_opt,
    )
    .await?;
  } else if let Some(cli_add_group) = cli_add.subcommand_matches("group") {
    let label = cli_add_group
      .get_one::<String>("label")
      .context("'label' argument is mandatory")?;
    let description = cli_add_group
      .get_one::<String>("description")
      .map(String::as_str);
    let node_expression =
      cli_add_group.get_one::<String>("nodes").map(String::as_str);
    add_group::exec(
      ctx,
      &token,
      label,
      description,
      node_expression,
      true,
      false,
    )
    .await?;
  } else if let Some(cli_add_hw_configuration) =
    cli_add.subcommand_matches("hardware")
  {
    let target_hsm_group_name_arg_opt = cli_add_hw_configuration
      .get_one::<String>("target-cluster")
      .map(String::as_str);
    let target_hsm_group_vec = get_groups_names_available(
      ctx.infra.backend,
      &token,
      target_hsm_group_name_arg_opt,
      ctx.cli.settings_hsm_group_name_opt,
    )
    .await?;
    let parent_hsm_group_name_arg_opt = cli_add_hw_configuration
      .get_one::<String>("parent-cluster")
      .map(String::as_str);
    let parent_hsm_group_vec = get_groups_names_available(
      ctx.infra.backend,
      &token,
      parent_hsm_group_name_arg_opt,
      ctx.cli.settings_hsm_group_name_opt,
    )
    .await?;
    let dryrun = cli_add_hw_configuration.get_flag("dry-run");
    let create_hsm_group = *cli_add_hw_configuration
      .get_one::<bool>("create-hsm-group")
      .unwrap_or(&false);
    add_hw_component_cluster::exec(
      ctx.infra.backend,
      &token,
      target_hsm_group_vec
        .first()
        .context("No target HSM groups available")?,
      parent_hsm_group_vec
        .first()
        .context("No parent HSM groups available")?,
      cli_add_hw_configuration
        .get_one::<String>("pattern")
        .context("'pattern' argument is mandatory")?,
      dryrun,
      create_hsm_group,
    )
    .await?;
  } else if let Some(cli_add_boot_parameters) =
    cli_add.subcommand_matches("boot-parameters")
  {
    add_boot_parameters::exec(ctx, &token, cli_add_boot_parameters).await?;
  } else if let Some(cli_add_kernel_parameters) =
    cli_add.subcommand_matches("kernel-parameters")
  {
    let hsm_group_name_arg_opt = cli_add_kernel_parameters
      .get_one::<String>("hsm-group")
      .map(String::as_str);
    let nodes_opt: Option<&str> = if hsm_group_name_arg_opt.is_none() {
      cli_add_kernel_parameters
        .get_one::<String>("nodes")
        .map(String::as_str)
    } else {
      None
    };
    let kernel_parameters = cli_add_kernel_parameters
      .get_one::<String>("VALUE")
      .context("'VALUE' argument is mandatory")?;
    let overwrite: bool = cli_add_kernel_parameters.get_flag("overwrite");
    let assume_yes: bool = cli_add_kernel_parameters.get_flag("assume-yes");
    let do_not_reboot: bool =
      cli_add_kernel_parameters.get_flag("do-not-reboot");
    let dryrun = cli_add_kernel_parameters.get_flag("dry-run");
    add_kernel_parameters::exec(
      ctx,
      kernel_parameters,
      nodes_opt,
      hsm_group_name_arg_opt,
      overwrite,
      assume_yes,
      do_not_reboot,
      dryrun,
    )
    .await?;
  } else if let Some(cli_add_redfish_endpoint) =
    cli_add.subcommand_matches("redfish-endpoint")
  {
    add_redfish_endpoint::exec(ctx, &token, cli_add_redfish_endpoint).await?;
  } else {
    bail!("Unknown 'add' subcommand");
  }
  Ok(())
}
