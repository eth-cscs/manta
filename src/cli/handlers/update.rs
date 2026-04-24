use crate::cli::commands::{update_boot_parameters, update_redfish_endpoint};
use crate::common::app_context::AppContext;
use crate::common::authentication::get_api_token;
use anyhow::{Context, Error, bail};
use clap::ArgMatches;

/// Dispatch `manta update` subcommands (boot-parameters,
/// redfish-endpoint).
pub async fn handle_update(
  cli_update: &ArgMatches,
  ctx: &AppContext<'_>,
) -> Result<(), Error> {
  let token = get_api_token(ctx.infra.backend, ctx.infra.site_name).await?;

  match cli_update.subcommand() {
    Some(("boot-parameters", m)) => {
      let hosts: &str = m
        .get_one::<String>("hosts")
        .map(String::as_str)
        .context("The 'hosts' argument is mandatory")?;
      let params = m.get_one::<String>("params").map(String::as_str);
      let kernel = m.get_one::<String>("kernel").map(String::as_str);
      let initrd = m.get_one::<String>("initrd").map(String::as_str);
      update_boot_parameters::exec(ctx, &token, hosts, None, None, params, kernel, initrd)
        .await?;
    }
    Some(("redfish-endpoint", m)) => {
      let id: String = m
        .get_one("id")
        .cloned()
        .context("The 'id' argument is mandatory")?;
      let name: Option<String> = m.get_one("name").cloned();
      let hostname: Option<String> = m.get_one("hostname").cloned();
      let domain: Option<String> = m.get_one("domain").cloned();
      let fqdn: Option<String> = m.get_one("fqdn").cloned();
      let enabled: bool = m.get_flag("enabled");
      let user: Option<String> = m.get_one("user").cloned();
      let password: Option<String> = m.get_one("password").cloned();
      let use_ssdp: bool = m.get_flag("use-ssdp");
      let mac_required: bool = m.get_flag("mac-required");
      let mac_addr: Option<String> = m.get_one("macaddr").cloned();
      let ip_address: Option<String> = m.get_one("ipaddress").cloned();
      let rediscover_on_update: bool = m.get_flag("rediscover-on-update");
      let template_id: Option<String> = m.get_one("template-id").cloned();
      update_redfish_endpoint::exec(
        ctx,
        &token,
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
      )
      .await?;
    }
    Some((other, _)) => bail!("Unknown 'update' subcommand: {other}"),
    None => bail!("No 'update' subcommand provided"),
  }
  Ok(())
}
