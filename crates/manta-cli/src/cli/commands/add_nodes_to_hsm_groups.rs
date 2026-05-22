//! Implements the `manta add-nodes-to-groups` command.

use anyhow::{Error, bail};

use crate::cli::common;
use crate::cli::http_client::MantaClient;
use crate::cli::output::action_result;
use manta_shared::common::{app_context::AppContext, audit, kafka::Kafka};

/// Add/assign a list of xnames to an HSM group.
#[allow(clippy::too_many_arguments)]
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  target_hsm_name: &str,
  hosts_expression: &str,
  dryrun: bool,
  kafka_audit_opt: Option<&Kafka>,
  output_opt: Option<&str>,
) -> Result<(), Error> {
  let server_url = ctx.manta_server_url;

  if !common::user_interaction::confirm(
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

  let (added, updated_members) = MantaClient::new(server_url, ctx.site_name)?
    .add_nodes_to_group(token, target_hsm_name, hosts_expression)
    .await?;

  action_result::print_with_data(
    &format!("HSM '{target_hsm_name}' members updated"),
    &updated_members,
    output_opt,
  )?;

  audit::maybe_send_audit(
    kafka_audit_opt,
    token,
    format!("add nodes to group: {target_hsm_name}"),
    Some(serde_json::json!(added)),
    Some(serde_json::json!(vec![target_hsm_name])),
  )
  .await;

  Ok(())
}
