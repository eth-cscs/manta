//! Implements the `manta delete boot-parameters` command.

use anyhow::Error;

use crate::cli::http_client::MantaClient;
use manta_shared::common::app_context::AppContext;

/// CLI adapter for `manta delete boot-parameters`.
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  hosts: Vec<String>,
) -> Result<(), Error> {
  let server_url = ctx.manta_server_url;
  MantaClient::new(server_url, ctx.site_name)?
    .delete_boot_parameters(token, hosts)
    .await?;
  println!("Boot parameters deleted successfully");
  Ok(())
}
