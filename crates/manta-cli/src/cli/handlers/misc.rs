//! Routes miscellaneous top-level commands (add-nodes-to-groups, etc.).

use crate::cli::commands::{
  add_nodes_to_hsm_groups, remove_nodes_from_hsm_groups,
};
use crate::cli::common::authentication::get_api_token;
use crate::cli::common::clap_ext::ArgMatchesExt;
use anyhow::{Error, bail};
use clap::ArgMatches;
use manta_shared::common::app_context::AppContext;

/// Dispatch top-level misc commands (add-nodes-to-groups,
/// remove-nodes-from-groups).
pub async fn handle_misc(
  cli_root: &ArgMatches,
  ctx: &AppContext<'_>,
) -> Result<(), Error> {
  let token = get_api_token(ctx).await?;

  match cli_root.subcommand() {
    Some(("add-nodes-to-groups", m)) => {
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
      let dryrun = m.get_flag("dry-run");
      let nodes = m.req_str("nodes")?;
      let target_hsm_name = m.req_str("group")?;
      remove_nodes_from_hsm_groups::exec(
        ctx,
        &token,
        target_hsm_name,
        nodes,
        dryrun,
        ctx.kafka_audit_opt,
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
