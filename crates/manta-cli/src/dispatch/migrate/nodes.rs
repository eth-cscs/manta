//! Implements the `manta migrate nodes` command.

use anyhow::Error;

use crate::common::app_context::AppContext;
use crate::http_client::MantaClient;
use crate::output::action_result;

pub struct ExecParams<'a> {
  pub target_groups: &'a [String],
  pub parent_groups: &'a [String],
  pub hosts_expression: &'a str,
  pub dry_run: bool,
  pub create_group: bool,
  pub output: Option<&'a str>,
}

/// Move nodes between HSM groups with validation.
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  p: ExecParams<'_>,
) -> Result<(), Error> {
  let server_url = ctx.manta_server_url;
  let result = MantaClient::new(server_url, ctx.site_name)?
    .migrate_nodes(
      token,
      p.target_groups,
      p.parent_groups,
      p.hosts_expression,
      p.dry_run,
      p.create_group,
    )
    .await?;
  let message = if p.dry_run {
    "dry-run enabled, changes not persisted."
  } else {
    "Nodes migrated."
  };
  action_result::print_with_data(message, &result, p.output)?;

  Ok(())
}
