//! Routes top-level deprecated aliases (add-nodes-to-groups,
//! remove-nodes-from-groups). The canonical forms (`manta add nodes`
//! / `manta delete nodes`) live under the add/delete verb trees; this
//! module exists solely to keep the old top-level spellings working
//! during the deprecation grace period.

use crate::commands::add::nodes as add_nodes;
use crate::commands::delete::nodes as delete_nodes;
use crate::common::authentication::get_api_token;
use crate::common::clap_ext::ArgMatchesExt;
use anyhow::{Error, bail};
use clap::ArgMatches;
use crate::common::app_context::AppContext;

/// Dispatch the deprecated top-level commands (add-nodes-to-groups,
/// remove-nodes-from-groups).
pub async fn handle_deprecated_aliases(
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
      add_nodes::exec(
        ctx,
        &token,
        target_hsm_name,
        hosts_expression,
        dryrun,
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
      delete_nodes::exec(
        ctx,
        &token,
        target_hsm_name,
        nodes,
        dryrun,
        output_opt,
      )
      .await?;
    }
    Some((other, _)) => bail!("Unknown command: {other}"),
    None => bail!("No command provided"),
  }
  Ok(())
}
