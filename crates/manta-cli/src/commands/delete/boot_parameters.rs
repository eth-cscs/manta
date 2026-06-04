//! Implements the `manta delete boot-parameters` command.

use anyhow::Error;

use crate::common::app_context::AppContext;
use crate::http_client::MantaClient;
use crate::output::action_result;

/// CLI adapter for `manta delete boot-parameters`.
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  hosts: Vec<String>,
  output_opt: Option<&str>,
) -> Result<(), Error> {
  let server_url = ctx.manta_server_url;
  MantaClient::new(server_url, ctx.site_name)?
    .delete_boot_parameters(token, hosts)
    .await?;
  action_result::print("Boot parameters deleted successfully", output_opt)?;
  Ok(())
}
