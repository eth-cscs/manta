//! Implements the `manta apply redfish-endpoint` command.

use anyhow::Error;

use crate::common::app_context::AppContext;
use crate::http_client::{MantaClient, OpenApiResultExt};
use crate::openapi_client::types::UpdateRedfishEndpointParams;
use crate::output::action_result;

/// CLI adapter for `manta apply redfish-endpoint`. Takes
/// `UpdateRedfishEndpointParams` directly — the generated wire-type
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
  let client = MantaClient::from_app_ctx(ctx, Some(token))?;
  client
    .openapi
    .update_redfish_endpoint(client.site_name(), &params)
    .await
    .into_anyhow()?;

  action_result::print(
    &format!("Redfish endpoint '{id}' updated"),
    output_opt,
  )?;

  Ok(())
}
