//! Implements the `manta delete nodes` command (and the deprecated
//! `manta remove-nodes-from-groups` alias that forwards to it).

use anyhow::{Error, bail};

use crate::common;
use crate::common::app_context::AppContext;
use crate::http_client::MantaClient;
use crate::output::action_result;

/// Remove/unassign a list of xnames from a list of HSM groups
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
      "Nodes matching '{hosts_expression}' will be removed from HSM group '{target_hsm_name}'. Do you want to proceed?"
    ),
    false,
  ) {
    bail!("Operation cancelled by user");
  }

  if dryrun {
    action_result::print(
      &format!(
        "dryrun - Delete nodes matching '{hosts_expression}' in {target_hsm_name}"
      ),
      output_opt,
    )?;
    return Ok(());
  }

  MantaClient::new(server_url, ctx.site_name)?
    .delete_group_members(token, target_hsm_name, hosts_expression, dryrun)
    .await?;

  action_result::print(
    &format!(
      "Removed nodes matching '{hosts_expression}' from HSM group '{target_hsm_name}'"
    ),
    output_opt,
  )?;

  Ok(())
}
