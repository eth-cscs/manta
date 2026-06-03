//! Implements the `manta delete hardware` command.

use anyhow::{Error, anyhow};

use crate::cli::http_client::MantaClient;
use crate::cli::output::action_result;
use crate::cli::common::app_context::AppContext;

/// Remove hardware components from a cluster group.
#[allow(clippy::too_many_arguments)]
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  target_hsm_group_name_arg_opt: Option<&str>,
  parent_hsm_group_name_arg_opt: Option<&str>,
  pattern: &str,
  dryrun: bool,
  delete_hsm_group: bool,
  output_opt: Option<&str>,
) -> Result<(), Error> {
  let server_url = ctx.manta_server_url;
  let target = target_hsm_group_name_arg_opt
    .or(ctx.settings_hsm_group_name_opt)
    .ok_or_else(|| anyhow!("No target HSM group specified"))?;
  let parent = parent_hsm_group_name_arg_opt
    .or(ctx.settings_hsm_group_name_opt)
    .ok_or_else(|| anyhow!("No parent HSM group specified"))?;
  let result = MantaClient::new(server_url, ctx.site_name)?
    .delete_hw_component(
      token,
      target,
      parent,
      pattern,
      delete_hsm_group,
      dryrun,
    )
    .await?;
  let message = if dryrun {
    "Dry run enabled, not modifying the HSM groups on the system."
  } else {
    "Hardware components removed."
  };
  action_result::print_with_data(message, &result, output_opt)?;
  Ok(())
}
