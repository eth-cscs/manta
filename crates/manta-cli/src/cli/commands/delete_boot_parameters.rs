//! Implements the `manta delete boot-parameters` command.

use anyhow::{Context, Error};

use crate::cli::http_client::MantaClient;
use crate::common::app_context::AppContext;

/// CLI adapter for `manta delete boot-parameters`.
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  hosts: Vec<String>,
) -> Result<(), Error> {
  let server_url = ctx
    .cli
    .manta_server_url
    .context("manta server URL must be configured")?;
  MantaClient::new(server_url, ctx.infra.site_name)?
    .delete_boot_parameters(token, hosts)
    .await?;
  println!("Boot parameters deleted successfully");
  Ok(())
}
