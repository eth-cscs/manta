//! Implements the `manta remove-nodes-from-groups` command.

use anyhow::{Context, Error, bail};

use crate::{
  cli::http_client::MantaClient,
  common::{self, app_context::AppContext, audit, kafka::Kafka},
};

/// Remove/unassign a list of xnames to a list of HSM groups
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  target_hsm_name: &str,
  hosts_expression: &str,
  dryrun: bool,
  kafka_audit_opt: Option<&Kafka>,
) -> Result<(), Error> {
  let server_url = ctx
    .cli
    .manta_server_url
    .context("manta server URL must be configured")?;

  if !common::user_interaction::confirm(
    &format!(
      "Nodes matching '{}' will be removed from HSM group '{}'. Do you want to proceed?",
      hosts_expression, target_hsm_name
    ),
    false,
  ) {
    bail!("Operation cancelled by user");
  }

  if dryrun {
    println!(
      "dryrun - Delete nodes matching '{}' in {}",
      hosts_expression, target_hsm_name
    );
    return Ok(());
  }

  MantaClient::new(server_url, ctx.infra.site_name)?
    .delete_group_members(token, target_hsm_name, hosts_expression, dryrun)
    .await?;

  audit::maybe_send_audit(
    kafka_audit_opt,
    token,
    format!("Remove nodes from group '{}'", target_hsm_name),
    Some(serde_json::json!(hosts_expression)),
    Some(serde_json::json!(vec![target_hsm_name])),
  )
  .await;

  Ok(())
}
