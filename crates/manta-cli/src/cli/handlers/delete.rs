//! Routes `manta delete *` subcommands to their exec functions.

use crate::cli::commands::{
  delete_and_cancel_session, delete_boot_parameters,
  delete_configurations_and_derivatives, delete_group,
  delete_hw_component_cluster, delete_images, delete_kernel_parameters,
  delete_node, delete_redfish_endpoint, remove_nodes_from_hsm_groups,
};
use crate::cli::common::authentication::get_api_token;
use crate::cli::common::clap_ext::ArgMatchesExt;
use anyhow::{Context, Error, bail};
use clap::ArgMatches;
use manta_shared::common::app_context::AppContext;

/// Dispatch `manta delete` subcommands (group, node,
/// session, configuration, image, boot-parameters,
/// kernel-parameters, hardware, redfish-endpoint).
pub async fn handle_delete(
  cli_delete: &ArgMatches,
  ctx: &AppContext<'_>,
) -> Result<(), Error> {
  let token = get_api_token(ctx).await?;

  match cli_delete.subcommand() {
    Some(("group", m)) => {
      let label = m.req_str("VALUE")?;
      let force: bool = *m
        .get_one("force")
        .context("'force' argument must have a value")?;
      let output_opt = m.opt_str("output");
      delete_group::exec(
        ctx,
        &token,
        label,
        force,
        ctx.kafka_audit_opt,
        output_opt,
      )
      .await?;
    }
    Some(("node", m)) => {
      let id = m.req_str("VALUE")?;
      let output_opt = m.opt_str("output");
      delete_node::exec(ctx, &token, id, output_opt).await?;
    }
    Some(("nodes", m)) => {
      let target_hsm_name = m.req_str("group")?;
      let hosts_expression = m.req_str("nodes")?;
      let dryrun = m.get_flag("dry-run");
      let output_opt = m.opt_str("output");
      remove_nodes_from_hsm_groups::exec(
        ctx,
        &token,
        target_hsm_name,
        hosts_expression,
        dryrun,
        ctx.kafka_audit_opt,
        output_opt,
      )
      .await?;
    }
    Some(("hardware", m)) => {
      let dryrun = m.get_flag("dry-run");
      let delete_hsm_group = m.get_flag("delete-hsm-group");
      let target_hsm_group_name_arg_opt = m.opt_str("target-cluster");
      let parent_hsm_group_name_arg_opt = m.opt_str("parent-cluster");
      let pattern = m.req_str("pattern")?;
      let output_opt = m.opt_str("output");
      delete_hw_component_cluster::exec(
        ctx,
        &token,
        target_hsm_group_name_arg_opt,
        parent_hsm_group_name_arg_opt,
        pattern,
        dryrun,
        delete_hsm_group,
        output_opt,
      )
      .await?;
    }
    Some(("boot-parameters", m)) => {
      let hosts: Vec<String> = m
        .opt_str("hosts")
        .unwrap_or_default()
        .split(',')
        .map(String::from)
        .collect();
      let output_opt = m.opt_str("output");
      delete_boot_parameters::exec(ctx, &token, hosts, output_opt).await?;
    }
    Some(("redfish-endpoints", m)) => {
      let id = m.get_one::<String>("id").context(
        "Host argument is mandatory. \
           Please provide the host to delete",
      )?;
      let output_opt = m.opt_str("output");
      delete_redfish_endpoint::exec(ctx, &token, id, output_opt).await?;
    }
    Some(("kernel-parameters", m)) => {
      let hsm_group_name_arg_opt = m.opt_str("group");
      let nodes = m.opt_str("nodes");
      let kernel_parameters = m.req_str("VALUE")?;
      let assume_yes: bool = m.get_flag("assume-yes");
      let do_not_reboot: bool = m.get_flag("do-not-reboot");
      let dryrun = m.get_flag("dry-run");
      let output_opt = m.opt_str("output");
      delete_kernel_parameters::exec(
        ctx,
        &token,
        hsm_group_name_arg_opt,
        nodes,
        kernel_parameters,
        assume_yes,
        do_not_reboot,
        dryrun,
        output_opt,
      )
      .await?;
    }
    Some(("session", m)) => {
      let session_name = m.req_str("SESSION_NAME")?;
      let assume_yes: bool = m.get_flag("assume-yes");
      let dry_run: bool = m.get_flag("dry-run");
      let output_opt = m.opt_str("output");
      if let Err(e) = delete_and_cancel_session::exec(
        ctx,
        &token,
        session_name,
        dry_run,
        assume_yes,
        output_opt,
      )
      .await
      {
        bail!("Failed to delete session: {e}");
      }
    }
    Some(("configurations", m)) => {
      let since_opt = if let Some(since) = m.get_one::<String>("since") {
        let date_time = chrono::NaiveDateTime::parse_from_str(
          &(since.to_string() + "T00:00:00"),
          "%Y-%m-%dT%H:%M:%S",
        )
        .context(format!(
          "Could not parse 'since' date '{since}'. Expected format: YYYY-MM-DD"
        ))?;
        Some(date_time)
      } else {
        None
      };
      let until_opt = if let Some(until) = m.get_one::<String>("until") {
        let date_time = chrono::NaiveDateTime::parse_from_str(
          &(until.to_string() + "T00:00:00"),
          "%Y-%m-%dT%H:%M:%S",
        )
        .context(format!(
          "Could not parse 'until' date '{until}'. Expected format: YYYY-MM-DD"
        ))?;
        Some(date_time)
      } else {
        None
      };
      let cfs_configuration_name_pattern = m.opt_str("configuration-name");
      let assume_yes = m.get_flag("assume-yes");
      let output_opt = m.opt_str("output");
      if let Err(e) = delete_configurations_and_derivatives::exec(
        ctx,
        &token,
        cfs_configuration_name_pattern,
        since_opt,
        until_opt,
        assume_yes,
        output_opt,
      )
      .await
      {
        bail!("Failed to delete configurations: {e}");
      }
    }
    Some(("images", m)) => {
      let image_id_vec: Vec<&str> =
        m.req_str("IMAGE_LIST")?.split(',').map(str::trim).collect();
      let dry_run: bool = m.get_flag("dry-run");
      let output_opt = m.opt_str("output");
      if let Err(e) = delete_images::exec(
        ctx,
        &token,
        image_id_vec.as_slice(),
        dry_run,
        output_opt,
      )
      .await
      {
        bail!("Failed to delete images: {e}");
      }
    }
    Some((other, _)) => bail!("Unknown 'delete' subcommand: {other}"),
    None => bail!("No 'delete' subcommand provided"),
  }
  Ok(())
}
