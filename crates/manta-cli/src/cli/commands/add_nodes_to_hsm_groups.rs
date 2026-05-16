//! Implements the `manta add-nodes-to-groups` command.

use anyhow::{Context, Error, bail};

use crate::cli::http_client::MantaClient;
use crate::common::{self, audit, kafka::Kafka, app_context::AppContext};

/// Add/assign a list of xnames to an HSM group.
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  target_hsm_name: &str,
  hosts_expression: &str,
  dryrun: bool,
  kafka_audit_opt: Option<&Kafka>,
) -> Result<(), Error> {
  let server_url = ctx.cli.manta_server_url
    .context("manta server URL must be configured")?;

  if !common::user_interaction::confirm(
    &format!(
      "Nodes matching '{}' will be added to HSM group '{}'. Do you want to proceed?",
      hosts_expression, target_hsm_name
    ),
    false,
  ) {
    bail!("Operation cancelled by user");
  }

  if dryrun {
    println!(
      "dryrun - Add nodes matching '{}' to {}",
      hosts_expression, target_hsm_name
    );
    return Ok(());
  }

  let (added, updated_members) = MantaClient::new(server_url, ctx.infra.site_name)?
    .add_nodes_to_group(token, target_hsm_name, hosts_expression)
    .await?;

  println!("HSM '{}' members: {:?}", target_hsm_name, updated_members);

  audit::maybe_send_audit(
    kafka_audit_opt,
    token,
    format!("add nodes to group: {}", target_hsm_name),
    Some(serde_json::json!(added)),
    Some(serde_json::json!(vec![target_hsm_name])),
  )
  .await;

  Ok(())
}
