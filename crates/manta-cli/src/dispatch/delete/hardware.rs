//! Implements the `manta delete hardware` command.

use anyhow::{Error, anyhow};

use crate::common::app_context::AppContext;
use crate::http_client::MantaClient;
use crate::output::action_result;

pub struct ExecParams<'a> {
  pub target_group: Option<&'a str>,
  pub parent_group: Option<&'a str>,
  pub pattern: &'a str,
  pub dry_run: bool,
  pub delete_group: bool,
  pub output: Option<&'a str>,
}

/// Remove hardware components from a cluster group.
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  p: ExecParams<'_>,
) -> Result<(), Error> {
  let server_url = ctx.manta_server_url;
  let target = p
    .target_group
    .or(ctx.settings_hsm_group_name_opt)
    .ok_or_else(|| anyhow!("No target HSM group specified"))?;
  let parent = p
    .parent_group
    .or(ctx.settings_hsm_group_name_opt)
    .ok_or_else(|| anyhow!("No parent HSM group specified"))?;
  let result = MantaClient::new(server_url, ctx.site_name)?
    .delete_hw_component(
      token,
      target,
      parent,
      p.pattern,
      p.delete_group,
      p.dry_run,
    )
    .await?;
  let message = if p.dry_run {
    "Dry run enabled, not modifying the HSM groups on the system."
  } else {
    "Hardware components removed."
  };
  action_result::print_with_data(message, &result, p.output)?;
  Ok(())
}
