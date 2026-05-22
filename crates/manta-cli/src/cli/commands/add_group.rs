//! Implements the `manta add group` command.

use anyhow::{Context, Error, bail};

use crate::cli::common;
use crate::cli::http_client::MantaClient;
use crate::cli::output::action_result;
use manta_shared::common::{app_context::AppContext, audit};
use manta_shared::shared::dto::Group;

/// CLI adapter for `manta add group`.
#[allow(clippy::too_many_arguments)]
pub async fn exec(
  ctx: &AppContext<'_>,
  auth_token: &str,
  label: &str,
  description: Option<&str>,
  hosts_expression_opt: Option<&str>,
  assume_yes: bool,
  dryrun: bool,
  output_opt: Option<&str>,
) -> Result<(), Error> {
  let server_url = ctx.manta_server_url;
  let grp = Group {
    label: label.to_string(),
    description: description.map(String::from),
    tags: None,
    members: None,
    exclusive_group: Some("false".to_string()),
  };

  if !common::user_interaction::confirm(
    &format!(
      "This operation will create the group below:\n{}\nPlease confirm to proceed",
      serde_json::to_string_pretty(&grp)
        .context("Failed to serialize group")?
    ),
    assume_yes,
  ) {
    bail!("Operation cancelled by user");
  }

  if dryrun {
    println!(
      "Dryrun mode: The group below would be created:\n{}",
      serde_json::to_string_pretty(&grp)
        .context("Failed to serialize group")?
    );
    return Ok(());
  }

  let client = MantaClient::new(server_url, ctx.site_name)?;
  client.create_group(auth_token, grp).await?;

  let mut added = vec![];
  if let Some(expr) = hosts_expression_opt {
    let (members, _) =
      client.add_nodes_to_group(auth_token, label, expr).await?;
    added = members;
  }

  action_result::print(&format!("Group '{label}' created"), output_opt)?;

  audit::maybe_send_audit(
    ctx.kafka_audit_opt,
    auth_token,
    format!("Create Group '{label}'"),
    Some(serde_json::json!(added)),
    Some(serde_json::json!(label)),
  )
  .await;

  Ok(())
}
