//! Implements the `manta add hardware` command.

use crate::http_client::MantaClient;
use crate::output::action_result;
use crate::common::app_context::AppContext;

pub struct ExecParams<'a> {
  pub target_group: &'a str,
  pub parent_group: &'a str,
  pub pattern: &'a str,
  pub dry_run: bool,
  pub create_group: bool,
  pub output: Option<&'a str>,
}

/// Add hardware components to a cluster group (CLI entry point).
pub async fn exec(
  ctx: &AppContext<'_>,
  shasta_token: &str,
  p: ExecParams<'_>,
) -> anyhow::Result<()> {
  let server_url = ctx.manta_server_url;
  let result = MantaClient::new(server_url, ctx.site_name)?
    .add_hw_component(
      shasta_token,
      p.target_group,
      p.parent_group,
      p.pattern,
      p.create_group,
      p.dry_run,
    )
    .await?;
  let message = if p.dry_run {
    "Dryrun enabled, not modifying the groups on the system."
  } else {
    "Hardware components added."
  };
  action_result::print_with_data(message, &result, p.output)?;
  Ok(())
}
