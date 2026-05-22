//! Routes `manta add *` subcommands to their exec functions.

use crate::cli::commands::{
  add_boot_parameters, add_group, add_hw_component_cluster,
  add_kernel_parameters, add_node, add_redfish_endpoint,
};
use crate::cli::common::authentication::get_api_token;
use crate::cli::common::clap_ext::ArgMatchesExt;
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
      let id = m.req_str("id")?;
      let group = m.req_str("group")?;
      let hardware_file_opt = m.get_one::<PathBuf>("hardware");
      let arch_opt = m.opt_string("arch");
      let enabled = !m.get_flag("disabled");
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
      let label = m.req_str("label")?;
      let description = m.opt_str("description");
      let node_expression = m.opt_str("nodes");
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
        .opt_str("target-cluster")
        .or(ctx.settings_hsm_group_name_opt)
        .context("'target-cluster' is required (no default in cli.toml)")?;
      let parent = m
        .opt_str("parent-cluster")
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
        m.req_str("pattern")?,
        dryrun,
        create_hsm_group,
      )
      .await?;
    }
    Some(("boot-parameters", m)) => {
      add_boot_parameters::exec(ctx, &token, m).await?;
    }
    Some(("kernel-parameters", m)) => {
      let hsm_group_name_arg_opt = m.opt_str("group");
      let nodes_opt: Option<&str> = if hsm_group_name_arg_opt.is_none() {
        m.opt_str("nodes")
      } else {
        None
      };
      let kernel_parameters = m.req_str("VALUE")?;
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
