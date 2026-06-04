//! Implements the `manta update redfish-endpoint` command.

use anyhow::Error;

use crate::http_client::MantaClient;
use crate::output::action_result;
use crate::common::app_context::AppContext;
use manta_shared::shared::params::redfish_endpoints::UpdateRedfishEndpointParams;

/// CLI adapter for `manta update redfish-endpoint`. Takes
/// `UpdateRedfishEndpointParams` directly — the shared param struct is
/// the natural request body and already groups every field.
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  params: UpdateRedfishEndpointParams,
  output_opt: Option<&str>,
) -> Result<(), Error> {
  let id = params.id.clone();
  let server_url = ctx.manta_server_url;
  MantaClient::new(server_url, ctx.site_name)?
    .update_redfish_endpoint(token, &params)
    .await?;

  action_result::print(
    &format!("Redfish endpoint '{id}' updated"),
    output_opt,
  )?;

  Ok(())
}
