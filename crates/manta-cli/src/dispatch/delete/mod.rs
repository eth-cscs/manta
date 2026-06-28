//! `manta delete` subcommands.
//!
//! Each leaf removes a resource (or batch of resources) via the
//! corresponding `DELETE` endpoint:
//!
//! - [`node`]              — `DELETE /api/v1/nodes/{id}`
//! - [`nodes`]             — `DELETE /api/v1/groups/{label}/members`
//! - [`group`]             — `DELETE /api/v1/groups/{label}`
//! - [`hardware`]          — `DELETE /api/v1/hardware-clusters/{target}/members`
//! - [`boot_parameters`]   — `DELETE /api/v1/boot-parameters`
//! - [`kernel_parameters`] — `DELETE /api/v1/kernel-parameters`
//! - [`configurations_and_derivatives`] — `DELETE /api/v1/configurations`
//!   (also cascades through CFS sessions / IMS images derived from
//!   each configuration; the name reflects the cascade)
//! - [`images`]            — `DELETE /api/v1/images?ids=…`
//! - [`session`]           — `DELETE /api/v1/sessions/{name}` (cancel a CFS session)
//! - [`redfish_endpoint`]  — `DELETE /api/v1/redfish-endpoints/{id}`
//!
//! `--dry-run` is supported on every leaf. Most leaves either forward
//! the flag to the server (when the endpoint accepts `?dry_run=true`)
//! or short-circuit client-side via
//! [`crate::output::action_result::preview_request`]; see each leaf's
//! doc for the exact policy. A few destructive leaves prompt before
//! firing — `nodes` and `session` always confirm.

pub mod boot_parameters;
pub mod configurations_and_derivatives;
pub mod group;
pub mod hardware;
pub mod images;
pub mod kernel_parameters;
pub mod node;
pub mod nodes;
pub mod redfish_endpoint;
pub mod session;

use crate::common::app_context::AppContext;
use crate::common::authentication::get_api_token;
use crate::common::clap_ext::ArgMatchesExt;
use anyhow::{Context, Error, bail};
use clap::ArgMatches;

/// Dispatch `manta delete` subcommands (group, node, nodes,
/// kernel-parameters, boot-parameters, configurations, session,
/// images, hardware, redfish-endpoints).
///
/// # Errors
///
/// Returns an error when the auth token cannot be obtained, when a
/// required clap argument is missing, when `--since` / `--until`
/// cannot be parsed as `YYYY-MM-DD`, when no subcommand is provided
/// or the name is unknown, or when the leaf handler itself fails.
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
      let dry_run: bool = m.get_flag("dry-run");
      let output_opt = m.opt_str("output");
      group::exec(ctx, &token, label, force, dry_run, output_opt).await?;
    }
    Some(("node", m)) => {
      let id = m.req_str("VALUE")?;
      let output_opt = m.opt_str("output");
      let dry_run = m.get_flag("dry-run");
      node::exec(ctx, &token, id, output_opt, dry_run).await?;
    }
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
    Some(("hardware", m)) => {
      hardware::exec(
        ctx,
        &token,
        hardware::ExecParams {
          target_group: m.opt_str("target-group"),
          parent_group: m.opt_str("parent-group"),
          pattern: m.req_str("pattern")?,
          dry_run: m.get_flag("dry-run"),
          delete_group: m.get_flag("delete-group"),
          output: m.opt_str("output"),
        },
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
      let dry_run = m.get_flag("dry-run");
      boot_parameters::exec(ctx, &token, hosts, output_opt, dry_run).await?;
    }
    Some(("redfish-endpoints", m)) => {
      let id = m.get_one::<String>("id").context(
        "Host argument is mandatory. \
           Please provide the host to delete",
      )?;
      let output_opt = m.opt_str("output");
      let dry_run = m.get_flag("dry-run");
      redfish_endpoint::exec(ctx, &token, id, output_opt, dry_run).await?;
    }
    Some(("kernel-parameters", m)) => {
      let hsm_group_name_arg_opt = m.opt_str("group");
      let nodes_arg = m.opt_str("nodes");
      let kernel_parameters_val = m.req_str("VALUE")?;
      let dryrun = m.get_flag("dry-run");
      let output_opt = m.opt_str("output");
      kernel_parameters::exec(
        ctx,
        &token,
        kernel_parameters::ExecParams {
          kernel_params: kernel_parameters_val,
          nodes: nodes_arg,
          hsm_group: hsm_group_name_arg_opt,
          dry_run: dryrun,
          output: output_opt,
        },
      )
      .await?;
    }
    Some(("session", m)) => {
      let session_name = m.req_str("SESSION_NAME")?;
      let assume_yes: bool = m.get_flag("assume-yes");
      let dry_run: bool = m.get_flag("dry-run");
      let output_opt = m.opt_str("output");
      if let Err(e) = session::exec(
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
          &(since.clone() + "T00:00:00"),
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
          &(until.clone() + "T00:00:00"),
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
      let dry_run = m.get_flag("dry-run");
      let output_opt = m.opt_str("output");
      if let Err(e) = configurations_and_derivatives::exec(
        ctx,
        &token,
        configurations_and_derivatives::ExecParams {
          configuration_name_pattern: cfs_configuration_name_pattern,
          since: since_opt,
          until: until_opt,
          output: output_opt,
          dry_run,
        },
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
      if let Err(e) =
        images::exec(ctx, &token, image_id_vec.as_slice(), dry_run, output_opt)
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
