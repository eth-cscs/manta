use anyhow::Error;

use crate::common::app_context::AppContext;
use crate::service::redfish_endpoints::{self, UpdateRedfishEndpointParams};

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

  redfish_endpoints::update_redfish_endpoint(&ctx.infra, token, params).await?;

  Ok(())
}
