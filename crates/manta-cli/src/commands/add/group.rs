//! Implements the `manta add group` command.

use anyhow::{Context, Error, bail};

use crate::common;
use crate::common::app_context::AppContext;
use crate::http_client::MantaClient;
use crate::output::action_result;
use manta_shared::types::dto::Group;

pub struct ExecParams<'a> {
  pub label: &'a str,
  pub description: Option<&'a str>,
  pub hosts_expression: Option<&'a str>,
  pub assume_yes: bool,
  pub dry_run: bool,
  pub output: Option<&'a str>,
}

/// CLI adapter for `manta add group`.
pub async fn exec(
  ctx: &AppContext<'_>,
  auth_token: &str,
  p: ExecParams<'_>,
) -> Result<(), Error> {
  let server_url = ctx.manta_server_url;
  let grp = Group {
    label: p.label.to_string(),
    description: p.description.map(String::from),
    tags: None,
    members: None,
    exclusive_group: Some("false".to_string()),
  };

  if !common::confirm::confirm(
    &format!(
      "This operation will create the group below:\n{}\nPlease confirm to proceed",
      serde_json::to_string_pretty(&grp)
        .context("Failed to serialize group")?
    ),
    p.assume_yes,
  ) {
    bail!("Operation cancelled by user");
  }

  if p.dry_run {
    println!(
      "Dryrun mode: The group below would be created:\n{}",
      serde_json::to_string_pretty(&grp)
        .context("Failed to serialize group")?
    );
    return Ok(());
  }

  let client = MantaClient::new(server_url, ctx.site_name)?;
  client.create_group(auth_token, grp).await?;

  if let Some(expr) = p.hosts_expression {
    client.add_nodes_to_group(auth_token, p.label, expr).await?;
  }

  action_result::print(&format!("Group '{}' created", p.label), p.output)?;

  Ok(())
}
