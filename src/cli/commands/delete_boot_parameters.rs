//! Implements the `manta delete boot-parameters` command.

use anyhow::Error;

use crate::cli::http_client::MantaClient;
use crate::common::app_context::AppContext;
use crate::service::boot_parameters;

/// CLI adapter for `manta delete boot-parameters`.
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  hosts: Vec<String>,
) -> Result<(), Error> {
  if let Some(server_url) = ctx.infra.manta_server_url {
    MantaClient::new(server_url, ctx.infra.site_name)?
      .delete_boot_parameters(token, hosts)
      .await?;
  } else {
    boot_parameters::delete_boot_parameters(&ctx.infra, token, hosts).await?;
  }
  println!("Boot parameters deleted successfully");
  Ok(())
}
