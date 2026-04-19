use anyhow::{Context, Error};

use crate::common::app_context::AppContext;
use crate::service::redfish_endpoints::{self, UpdateRedfishEndpointParams};

/// CLI adapter for `manta add redfish-endpoint`.
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  cli_args: &clap::ArgMatches,
) -> Result<(), Error> {
  let id: String = cli_args
    .get_one::<String>("id")
    .context("'id' argument is mandatory")?
    .to_string();
  let name: Option<String> = cli_args.get_one("name").cloned();
  let hostname: Option<String> = cli_args.get_one::<String>("hostname").cloned();
  let domain: Option<String> = cli_args.get_one::<String>("domain").cloned();
  let fqdn: Option<String> = cli_args.get_one::<String>("fqdn").cloned();
  let enabled: bool = cli_args.get_flag("enabled");
  let user: Option<String> = cli_args.get_one::<String>("user").cloned();
  let password: Option<String> = cli_args.get_one::<String>("password").cloned();
  let use_ssdp: bool = cli_args.get_flag("use-ssdp");
  let mac_required: bool = cli_args.get_flag("mac-required");
  let mac_addr: Option<String> = cli_args.get_one::<String>("macaddr").cloned();
  let ip_address: Option<String> = cli_args.get_one::<String>("ipaddress").cloned();
  let rediscover_on_update: bool = cli_args.get_flag("rediscover-on-update");
  let template_id: Option<String> =
    cli_args.get_one::<String>("template-id").cloned();

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

  redfish_endpoints::add_redfish_endpoint(&ctx.infra, token, params).await?;

  println!("Redfish endpoint for node '{}' added", id);

  Ok(())
}
