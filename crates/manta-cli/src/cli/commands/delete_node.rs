//! Implements the `manta delete node` command.

use anyhow::Error;

use crate::cli::http_client::MantaClient;
use manta_shared::common::app_context::AppContext;

/// CLI adapter for `manta delete node`.
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  id: &str,
) -> Result<(), Error> {
  let server_url = ctx.manta_server_url;
  MantaClient::new(server_url, ctx.site_name)?
    .delete_node(token, id)
    .await?;
  println!("Node deleted '{id}'");
  Ok(())
}
