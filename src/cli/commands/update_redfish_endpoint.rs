//! Implements the `manta update redfish-endpoint` command.

use anyhow::{Context, Error};

use crate::cli::http_client::MantaClient;
use crate::common::app_context::AppContext;
use crate::service::redfish_endpoints::UpdateRedfishEndpointParams;

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
) -> Result<(), Error> {
  let params = UpdateRedfishEndpointParams {
    id,
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

  let server_url = ctx.cli.manta_server_url
    .context("manta server URL must be configured")?;
  MantaClient::new(server_url, ctx.infra.site_name)?
    .update_redfish_endpoint(token, &params)
    .await?;

  Ok(())
}
