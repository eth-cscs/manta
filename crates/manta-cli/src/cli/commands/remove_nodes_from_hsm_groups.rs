//! Implements the `manta remove-nodes-from-groups` command.

use anyhow::{Error, bail};

use crate::cli::common;
use crate::cli::http_client::MantaClient;
use manta_shared::common::{app_context::AppContext, audit, kafka::Kafka};

/// Remove/unassign a list of xnames to a list of HSM groups
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  target_hsm_name: &str,
  hosts_expression: &str,
  dryrun: bool,
  kafka_audit_opt: Option<&Kafka>,
) -> Result<(), Error> {
  let server_url = ctx.manta_server_url;

  if !common::user_interaction::confirm(
    &format!(
      "Nodes matching '{hosts_expression}' will be removed from HSM group '{target_hsm_name}'. Do you want to proceed?"
    ),
    false,
  ) {
    bail!("Operation cancelled by user");
  }

  if dryrun {
    println!(
      "dryrun - Delete nodes matching '{hosts_expression}' in {target_hsm_name}"
    );
    return Ok(());
  }

  MantaClient::new(server_url, ctx.site_name)?
    .delete_group_members(token, target_hsm_name, hosts_expression, dryrun)
    .await?;

  audit::maybe_send_audit(
    kafka_audit_opt,
    token,
    format!("Remove nodes from group '{target_hsm_name}'"),
    Some(serde_json::json!(hosts_expression)),
    Some(serde_json::json!(vec![target_hsm_name])),
  )
  .await;

  Ok(())
}
