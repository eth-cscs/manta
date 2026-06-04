//! Routes `manta add *` subcommands to their exec functions.

use crate::cli::commands::{
  add_boot_parameters, add_group, add_hw_component_group,
  add_kernel_parameters, add_node, add_nodes_to_hsm_groups,
  add_redfish_endpoint,
};
use crate::cli::common::authentication::get_api_token;
use crate::cli::common::clap_ext::ArgMatchesExt;
use anyhow::{Context, Error, bail};
use clap::ArgMatches;
use crate::cli::common::app_context::AppContext;
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
    Some(("nodes", m)) => {
      let target_hsm_name = m.req_str("group")?;
      let hosts_expression = m.req_str("nodes")?;
      let dryrun = m.get_flag("dry-run");
      let output_opt = m.opt_str("output");
      add_nodes_to_hsm_groups::exec(
        ctx,
        &token,
        target_hsm_name,
        hosts_expression,
        dryrun,
        output_opt,
      )
      .await?;
    }
    Some(("node", m)) => {
      add_node::exec(
        ctx,
        &token,
        add_node::ExecParams {
          id: m.req_str("id")?,
          group: m.req_str("group")?,
          enabled: !m.get_flag("disabled"),
          arch: m.opt_string("arch"),
          hardware_file: m.get_one::<PathBuf>("hardware"),
          output: m.opt_str("output"),
        },
      )
      .await?;
    }
    Some(("group", m)) => {
      add_group::exec(
        ctx,
        &token,
        add_group::ExecParams {
          label: m.req_str("label")?,
          description: m.opt_str("description"),
          hosts_expression: m.opt_str("nodes"),
          assume_yes: true,
          dry_run: false,
          output: m.opt_str("output"),
        },
      )
      .await?;
    }
    Some(("hardware", m)) => {
      // Authorization (target + parent HSM group access) is enforced by
      // POST /api/v1/hardware-clusters/{target}/members on the server.
      let target = m
        .opt_str("target-group")
        .or(ctx.settings_hsm_group_name_opt)
        .context("'target-cluster' is required (no default in cli.toml)")?;
      let parent = m
        .opt_str("parent-group")
        .or(ctx.settings_hsm_group_name_opt)
        .context("'parent-cluster' is required (no default in cli.toml)")?;
      add_hw_component_group::exec(
        ctx,
        &token,
        add_hw_component_group::ExecParams {
          target_group: target,
          parent_group: parent,
          pattern: m.req_str("pattern")?,
          dry_run: m.get_flag("dry-run"),
          create_group: *m.get_one::<bool>("create-group").unwrap_or(&false),
          output: m.opt_str("output"),
        },
      )
      .await?;
    }
    Some(("boot-parameters", m)) => {
      add_boot_parameters::exec(ctx, &token, m).await?;
    }
    Some(("kernel-parameters", m)) => {
      let hsm_group = m.opt_str("group");
      let nodes_opt: Option<&str> = if hsm_group.is_none() {
        m.opt_str("nodes")
      } else {
        None
      };
      add_kernel_parameters::exec(
        ctx,
        &token,
        add_kernel_parameters::ExecParams {
          kernel_params: m.req_str("VALUE")?,
          hosts_expression: nodes_opt,
          hsm_group,
          overwrite: m.get_flag("overwrite"),
          dry_run: m.get_flag("dry-run"),
          output: m.opt_str("output"),
        },
      )
      .await?;
    }
    Some(("redfish-endpoints", m)) => {
      add_redfish_endpoint::exec(ctx, &token, m).await?;
    }
    Some((other, _)) => bail!("Unknown 'add' subcommand: {other}"),
    None => bail!("No 'add' subcommand provided"),
  }
  Ok(())
}
