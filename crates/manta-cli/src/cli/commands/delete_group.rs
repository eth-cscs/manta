//! Implements the `manta delete group` command.

use anyhow::Error;

use crate::cli::http_client::MantaClient;
use crate::cli::output::action_result;
use manta_shared::common::app_context::AppContext;

/// CLI adapter for `manta delete group`.
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  label: &str,
  force: bool,
  output_opt: Option<&str>,
) -> Result<(), Error> {
  let server_url = ctx.manta_server_url;
  MantaClient::new(server_url, ctx.site_name)?
    .delete_group(token, label, force)
    .await?;

  action_result::print(&format!("Group '{label}' deleted"), output_opt)?;

  Ok(())
}
