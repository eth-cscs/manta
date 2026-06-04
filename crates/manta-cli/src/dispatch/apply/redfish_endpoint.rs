//! Implements the `manta apply redfish-endpoint` command.

use anyhow::Error;

use crate::common::app_context::AppContext;
use crate::http_client::MantaClient;
use crate::output::action_result;
use manta_shared::types::params::redfish_endpoints::UpdateRedfishEndpointParams;

/// CLI adapter for `manta apply redfish-endpoint`. Takes
/// `UpdateRedfishEndpointParams` directly — the shared wire-type
/// struct is the natural request body and already groups every
/// field. The wire type retains the `Update` prefix because that's
/// the HTTP-API operation name (`PUT /redfish-endpoints`).
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
