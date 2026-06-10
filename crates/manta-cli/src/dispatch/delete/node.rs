//! Implements the `manta delete node` command.

use anyhow::Error;

use crate::common::app_context::AppContext;
use crate::http_client::{MantaClient, OpenApiResultExt};
use crate::output::action_result;

/// CLI adapter for `manta delete node`.
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  id: &str,
  output_opt: Option<&str>,
) -> Result<(), Error> {
  let client = MantaClient::from_app_ctx(ctx, Some(token))?;
  client
    .openapi
    .delete_node(id, client.site_name())
    .await
    .into_anyhow()?;
  action_result::print(&format!("Node deleted '{id}'"), output_opt)?;
  Ok(())
}
