//! Implements the `manta delete boot-parameters` command.

use anyhow::Error;

use crate::common::app_context::AppContext;
use crate::http_client::{MantaClient, OpenApiResultExt};
use crate::openapi_client::types::DeleteBootParametersRequest;
use crate::output::action_result;

/// CLI adapter for `manta delete boot-parameters`.
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  hosts: Vec<String>,
  output_opt: Option<&str>,
) -> Result<(), Error> {
  let client = MantaClient::from_app_ctx(ctx, Some(token))?;
  client
    .openapi
    .delete_boot_parameters(
      client.site_name(),
      &DeleteBootParametersRequest { hosts },
    )
    .await
    .into_anyhow()?;
  action_result::print("Boot parameters deleted successfully", output_opt)?;
  Ok(())
}
