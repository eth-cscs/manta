//! `manta add` subcommands.

pub mod boot_parameters;
pub mod group;
pub mod hardware;
pub mod kernel_parameters;
pub mod node;
pub mod nodes;
pub mod redfish_endpoint;

use crate::common::app_context::AppContext;
use crate::common::authentication::get_api_token;
use crate::common::clap_ext::ArgMatchesExt;
use anyhow::{Context, Error, bail};
use clap::ArgMatches;
use std::path::PathBuf;

/// Dispatch `manta add` subcommands (node, nodes, group, hardware,
/// boot-parameters, kernel-parameters, redfish-endpoints).
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
      nodes::exec(
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
      node::exec(
        ctx,
        &token,
        node::ExecParams {
          id: m.req_str("id")?,
          group: m.req_str("group")?,
          enabled: !m.get_flag("disabled"),
          arch: m.opt_string("arch"),
          hardware_file: m.get_one::<PathBuf>("hardware"),
          output: m.opt_str("output"),
          dry_run: m.get_flag("dry-run"),
        },
      )
      .await?;
    }
    Some(("group", m)) => {
      group::exec(
        ctx,
        &token,
        group::ExecParams {
          label: m.req_str("label")?,
          description: m.opt_str("description"),
          hosts_expression: m.opt_str("nodes"),
          assume_yes: true,
          dry_run: m.get_flag("dry-run"),
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
        .or(ctx.settings_group_name_opt)
        .context("'target-cluster' is required (no default in cli.toml)")?;
      let parent = m
        .opt_str("parent-group")
        .or(ctx.settings_group_name_opt)
        .context("'parent-cluster' is required (no default in cli.toml)")?;
      hardware::exec(
        ctx,
        &token,
        hardware::ExecParams {
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
      boot_parameters::exec(ctx, &token, m).await?;
    }
    Some(("kernel-parameters", m)) => {
      let hsm_group = m.opt_str("group");
      let nodes_opt: Option<&str> = if hsm_group.is_none() {
        m.opt_str("nodes")
      } else {
        None
      };
      kernel_parameters::exec(
        ctx,
        &token,
        kernel_parameters::ExecParams {
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
      redfish_endpoint::exec(
        ctx,
        &token,
        redfish_endpoint::ExecParams {
          id: m.req_str("id")?,
          name: m.opt_str("name"),
          hostname: m.opt_str("hostname"),
          domain: m.opt_str("domain"),
          fqdn: m.opt_str("fqdn"),
          enabled: m.get_flag("enabled"),
          user: m.opt_str("user"),
          password: m.opt_str("password"),
          use_ssdp: m.get_flag("use-ssdp"),
          mac_required: m.get_flag("mac-required"),
          mac_addr: m.opt_str("macaddr"),
          ip_address: m.opt_str("ipaddress"),
          rediscover_on_update: m.get_flag("rediscover-on-update"),
          template_id: m.opt_str("template-id"),
          output: m.opt_str("output"),
          dry_run: m.get_flag("dry-run"),
        },
      )
      .await?;
    }
    Some((other, _)) => bail!("Unknown 'add' subcommand: {other}"),
    None => bail!("No 'add' subcommand provided"),
  }
  Ok(())
}
