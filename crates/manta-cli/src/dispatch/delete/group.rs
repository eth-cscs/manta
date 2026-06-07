//! Implements the `manta delete group` command.

use anyhow::Error;

use crate::common::app_context::AppContext;
use crate::http_client::MantaClient;
use crate::output::action_result;

/// CLI adapter for `manta delete group`.
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  label: &str,
  force: bool,
  output_opt: Option<&str>,
) -> Result<(), Error> {
  MantaClient::from_app_ctx(ctx)?
    .delete_group(token, label, force)
    .await?;

  action_result::print(&format!("Group '{label}' deleted"), output_opt)?;

  Ok(())
}
