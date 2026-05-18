//! Routes `manta add *` subcommands to their exec functions.

use crate::cli::commands::{
  add_boot_parameters, add_group, add_hw_component_cluster,
  add_kernel_parameters, add_node, add_redfish_endpoint,
};
use crate::cli::common::authentication::get_api_token;
use anyhow::{Context, Error, bail};
use clap::ArgMatches;
use manta_shared::common::app_context::AppContext;
use std::path::PathBuf;

/// Dispatch `manta add` subcommands (node, group,
/// kernel-parameters, hardware cluster, boot-parameters,
/// redfish-endpoint).
pub async fn handle_add(
  cli_add: &ArgMatches,
  ctx: &AppContext<'_>,
) -> Result<(), Error> {
  let token = get_api_token(ctx).await?;

  match cli_add.subcommand() {
    Some(("node", m)) => {
      let id = m
        .get_one::<String>("id")
        .context("'id' argument is mandatory")?;
      let group = m
        .get_one::<String>("group")
        .context("'group' argument is mandatory")?;
      let hardware_file_opt = m.get_one::<PathBuf>("hardware");
      let arch_opt = m.get_one::<String>("arch").cloned();
      let enabled = m.get_one::<bool>("disabled").cloned().unwrap_or(true);
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
    }
    Some(("group", m)) => {
      let label = m
        .get_one::<String>("label")
        .context("'label' argument is mandatory")?;
      let description = m.get_one::<String>("description").map(String::as_str);
      let node_expression = m.get_one::<String>("nodes").map(String::as_str);
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
    }
    Some(("hardware", m)) => {
      // Authorization (target + parent HSM group access) is enforced by
      // POST /api/v1/hardware-clusters/{target}/members on the server.
      let target = m
        .get_one::<String>("target-cluster")
        .map(String::as_str)
        .or(ctx.settings_hsm_group_name_opt)
        .context("'target-cluster' is required (no default in cli.toml)")?;
      let parent = m
        .get_one::<String>("parent-cluster")
        .map(String::as_str)
        .or(ctx.settings_hsm_group_name_opt)
        .context("'parent-cluster' is required (no default in cli.toml)")?;
      let dryrun = m.get_flag("dry-run");
      let create_hsm_group =
        *m.get_one::<bool>("create-hsm-group").unwrap_or(&false);
      add_hw_component_cluster::exec(
        ctx,
        &token,
        target,
        parent,
        m.get_one::<String>("pattern")
          .context("'pattern' argument is mandatory")?,
        dryrun,
        create_hsm_group,
      )
      .await?;
    }
    Some(("boot-parameters", m)) => {
      add_boot_parameters::exec(ctx, &token, m).await?;
    }
    Some(("kernel-parameters", m)) => {
      let hsm_group_name_arg_opt =
        m.get_one::<String>("hsm-group").map(String::as_str);
      let nodes_opt: Option<&str> = if hsm_group_name_arg_opt.is_none() {
        m.get_one::<String>("nodes").map(String::as_str)
      } else {
        None
      };
      let kernel_parameters = m
        .get_one::<String>("VALUE")
        .context("'VALUE' argument is mandatory")?;
      let overwrite: bool = m.get_flag("overwrite");
      let assume_yes: bool = m.get_flag("assume-yes");
      let do_not_reboot: bool = m.get_flag("do-not-reboot");
      let dryrun = m.get_flag("dry-run");
      add_kernel_parameters::exec(
        ctx,
        &token,
        kernel_parameters,
        nodes_opt,
        hsm_group_name_arg_opt,
        overwrite,
        assume_yes,
        do_not_reboot,
        dryrun,
      )
      .await?;
    }
    Some(("redfish-endpoint", m)) => {
      add_redfish_endpoint::exec(ctx, &token, m).await?;
    }
    Some((other, _)) => bail!("Unknown 'add' subcommand: {other}"),
    None => bail!("No 'add' subcommand provided"),
  }
  Ok(())
}
