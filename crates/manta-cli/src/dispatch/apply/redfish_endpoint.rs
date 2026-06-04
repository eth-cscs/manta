//! Implements the `manta apply redfish-endpoint` command.

use anyhow::Error;

use crate::common::app_context::AppContext;
use crate::http_client::MantaClient;
use crate::output::action_result;
use manta_shared::types::params::redfish_endpoints::UpdateRedfishEndpointParams;

/// CLI adapter for `manta apply redfish-endpoint`. Takes
/// `UpdateRedfishEndpointParams` directly — the shared wire-type
/// struct is the natural request body and already groups every
/// field. (The struct name still says "Update" because that's the
/// underlying HTTP operation against the wire endpoint; this CLI
/// command is the canonical surface for it.)
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
