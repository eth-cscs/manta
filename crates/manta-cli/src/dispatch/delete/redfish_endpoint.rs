//! Implements the `manta delete redfish-endpoint` command.

use anyhow::Error;

use crate::common::app_context::AppContext;
use crate::http_client::MantaClient;
use crate::output::action_result;

/// CLI adapter for `manta delete redfish-endpoint`.
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  id: &str,
  output_opt: Option<&str>,
) -> Result<(), Error> {
  MantaClient::from_app_ctx(ctx)?
    .delete_redfish_endpoint(token, id)
    .await?;
  action_result::print(
    &format!("Redfish endpoint for id '{id}' deleted successfully"),
    output_opt,
  )?;
  Ok(())
}
