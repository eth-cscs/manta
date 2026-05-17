//! Implements the `manta delete node` command.

use anyhow::{Context, Error};

use crate::cli::http_client::MantaClient;
use crate::common::app_context::AppContext;

/// CLI adapter for `manta delete node`.
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
    .delete_node(token, id)
    .await?;
  println!("Node deleted '{}'", id);
  Ok(())
}
