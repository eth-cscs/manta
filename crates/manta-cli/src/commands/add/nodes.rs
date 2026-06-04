//! Implements the `manta add nodes` command (and the deprecated
//! `manta add-nodes-to-groups` alias that forwards to it).

use anyhow::{Error, bail};

use crate::common;
use crate::http_client::MantaClient;
use crate::output::action_result;
use crate::common::app_context::AppContext;

/// Add/assign a list of xnames to an HSM group.
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  target_hsm_name: &str,
  hosts_expression: &str,
  dryrun: bool,
  output_opt: Option<&str>,
) -> Result<(), Error> {
  let server_url = ctx.manta_server_url;

  if !common::confirm::confirm(
    &format!(
      "Nodes matching '{hosts_expression}' will be added to HSM group '{target_hsm_name}'. Do you want to proceed?"
    ),
    false,
  ) {
    bail!("Operation cancelled by user");
  }

  if dryrun {
    action_result::print(
      &format!(
        "dryrun - Add nodes matching '{hosts_expression}' to {target_hsm_name}"
      ),
      output_opt,
    )?;
    return Ok(());
  }

  let (_added, updated_members) = MantaClient::new(server_url, ctx.site_name)?
    .add_nodes_to_group(token, target_hsm_name, hosts_expression)
    .await?;

  action_result::print_with_data(
    &format!("HSM '{target_hsm_name}' members updated"),
    &updated_members,
    output_opt,
  )?;

  Ok(())
}
