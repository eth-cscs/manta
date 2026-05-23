//! Implements the `manta update redfish-endpoint` command.

use anyhow::Error;

use crate::cli::http_client::MantaClient;
use crate::cli::output::action_result;
use manta_shared::common::app_context::AppContext;
use manta_shared::shared::params::redfish_endpoints::UpdateRedfishEndpointParams;

/// CLI adapter for `manta update redfish-endpoint`.
#[allow(clippy::too_many_arguments)]
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  id: String,
  name: Option<String>,
  hostname: Option<String>,
  domain: Option<String>,
  fqdn: Option<String>,
  enabled: bool,
  user: Option<String>,
  password: Option<String>,
  use_ssdp: bool,
  mac_required: bool,
  mac_addr: Option<String>,
  ip_address: Option<String>,
  rediscover_on_update: bool,
  template_id: Option<String>,
  output_opt: Option<&str>,
) -> Result<(), Error> {
  let params = UpdateRedfishEndpointParams {
    id: id.clone(),
    name,
    hostname,
    domain,
    fqdn,
    enabled,
    user,
    password,
    use_ssdp,
    mac_required,
    mac_addr,
    ip_address,
    rediscover_on_update,
    template_id,
  };

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
