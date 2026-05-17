//! Implements the `manta delete redfish-endpoint` command.

use anyhow::{Context, Error};

use crate::cli::http_client::MantaClient;
use crate::common::app_context::AppContext;

/// CLI adapter for `manta delete redfish-endpoint`.
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  id: &str,
) -> Result<(), Error> {
  let server_url = ctx
    .cli
    .manta_server_url
    .context("manta server URL must be configured")?;
  MantaClient::new(server_url, ctx.infra.site_name)?
    .delete_redfish_endpoint(token, id)
    .await?;
  println!("Redfish endpoint for id '{}' deleted successfully", id);
  Ok(())
}
