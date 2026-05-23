//! Routes top-level deprecated aliases (add-nodes-to-groups,
//! remove-nodes-from-groups). The canonical forms (`manta add nodes`
//! / `manta delete nodes`) live under the add/delete verb trees; this
//! module exists solely to keep the old top-level spellings working
//! during the deprecation grace period.

use crate::cli::commands::{
  add_nodes_to_hsm_groups, remove_nodes_from_hsm_groups,
};
use crate::cli::common::authentication::get_api_token;
use crate::cli::common::clap_ext::ArgMatchesExt;
use anyhow::{Error, bail};
use clap::ArgMatches;
use manta_shared::common::app_context::AppContext;

/// Dispatch the deprecated top-level commands (add-nodes-to-groups,
/// remove-nodes-from-groups).
pub async fn handle_misc(
  cli_root: &ArgMatches,
  ctx: &AppContext<'_>,
) -> Result<(), Error> {
  let token = get_api_token(ctx).await?;

  match cli_root.subcommand() {
    Some(("add-nodes-to-groups", m)) => {
      eprintln!(
        "warning: 'manta add-nodes-to-groups' is deprecated; \
         use 'manta add nodes' instead.",
      );
      let dryrun = m.get_flag("dry-run");
      let hosts_expression = m.req_str("nodes")?;
      let target_hsm_name = m.req_str("group")?;
      let output_opt = m.opt_str("output");
      add_nodes_to_hsm_groups::exec(
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
    Some(("remove-nodes-from-groups", m)) => {
      eprintln!(
        "warning: 'manta remove-nodes-from-groups' is deprecated; \
         use 'manta delete nodes' instead.",
      );
      let dryrun = m.get_flag("dry-run");
      let nodes = m.req_str("nodes")?;
      let target_hsm_name = m.req_str("group")?;
      let output_opt = m.opt_str("output");
      remove_nodes_from_hsm_groups::exec(
        ctx,
        &token,
        target_hsm_name,
        nodes,
        dryrun,
        ctx.kafka_audit_opt,
        output_opt,
      )
      .await?;
    }
    Some(("download-boot-image", _)) => println!("Download boot image"),
    Some(("upload-boot-image", _)) => println!("Upload boot image"),
    Some((other, _)) => bail!("Unknown command: {other}"),
    None => bail!("No command provided"),
  }
  Ok(())
}
