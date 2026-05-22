//! Implements the `manta add redfish-endpoint` command.

use anyhow::Error;

use crate::cli::common::clap_ext::ArgMatchesExt;
use crate::cli::http_client::MantaClient;
use manta_shared::common::app_context::AppContext;
use manta_shared::shared::params::redfish_endpoints::UpdateRedfishEndpointParams;

/// CLI adapter for `manta add redfish-endpoint`.
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  cli_args: &clap::ArgMatches,
) -> Result<(), Error> {
  let id: String = cli_args.req_str("id")?.to_string();
  let name = cli_args.opt_string("name");
  let hostname = cli_args.opt_string("hostname");
  let domain = cli_args.opt_string("domain");
  let fqdn = cli_args.opt_string("fqdn");
  let enabled: bool = cli_args.get_flag("enabled");
  let user = cli_args.opt_string("user");
  let password = cli_args.opt_string("password");
  let use_ssdp: bool = cli_args.get_flag("use-ssdp");
  let mac_required: bool = cli_args.get_flag("mac-required");
  let mac_addr = cli_args.opt_string("macaddr");
  let ip_address = cli_args.opt_string("ipaddress");
  let rediscover_on_update: bool = cli_args.get_flag("rediscover-on-update");
  let template_id = cli_args.opt_string("template-id");

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
    .add_redfish_endpoint(token, params)
    .await?;

  println!("Redfish endpoint for node '{id}' added");

  Ok(())
}
