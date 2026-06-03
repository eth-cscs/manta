//! Implements the `manta migrate nodes` command.

use anyhow::Error;

use crate::cli::http_client::MantaClient;
use crate::cli::output::action_result;
use crate::cli::common::app_context::AppContext;

/// Move nodes between HSM groups with validation.
#[allow(clippy::too_many_arguments)]
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  target_hsm_name_vec: &[String],
  parent_hsm_name_vec: &[String],
  hosts_expression: &str,
  dry_run: bool,
  create_hsm_group: bool,
  output_opt: Option<&str>,
) -> Result<(), Error> {
  let server_url = ctx.manta_server_url;
  let result = MantaClient::new(server_url, ctx.site_name)?
    .migrate_nodes(
      token,
      target_hsm_name_vec,
      parent_hsm_name_vec,
      hosts_expression,
      dry_run,
      create_hsm_group,
    )
    .await?;
  let message = if dry_run {
    "dry-run enabled, changes not persisted."
  } else {
    "Nodes migrated."
  };
  action_result::print_with_data(message, &result, output_opt)?;

  Ok(())
}
