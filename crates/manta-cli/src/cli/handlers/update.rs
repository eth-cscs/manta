//! Routes `manta update *` subcommands to their exec functions.

use crate::cli::commands::{update_boot_parameters, update_redfish_endpoint};
use crate::cli::common::authentication::get_api_token;
use crate::cli::common::clap_ext::ArgMatchesExt;
use anyhow::{Context, Error, bail};
use clap::ArgMatches;
use manta_shared::common::app_context::AppContext;

/// Dispatch `manta update` subcommands (boot-parameters,
/// redfish-endpoint).
pub async fn handle_update(
  cli_update: &ArgMatches,
  ctx: &AppContext<'_>,
) -> Result<(), Error> {
  let token = get_api_token(ctx).await?;

  match cli_update.subcommand() {
    Some(("boot-parameters", m)) => {
      let hosts = m.req_str("hosts")?;
      let params = m.opt_str("params");
      let kernel = m.opt_str("kernel");
      let initrd = m.opt_str("initrd");
      let output_opt = m.opt_str("output");
      update_boot_parameters::exec(
        ctx, &token, hosts, None, None, params, kernel, initrd, output_opt,
      )
      .await?;
    }
    Some(("redfish-endpoints", m)) => {
      let id = m
        .opt_string("id")
        .context("The 'id' argument is mandatory")?;
      let name = m.opt_string("name");
      let hostname = m.opt_string("hostname");
      let domain = m.opt_string("domain");
      let fqdn = m.opt_string("fqdn");
      let enabled: bool = m.get_flag("enabled");
      let user = m.opt_string("user");
      let password = m.opt_string("password");
      let use_ssdp: bool = m.get_flag("use-ssdp");
      let mac_required: bool = m.get_flag("mac-required");
      let mac_addr = m.opt_string("macaddr");
      let ip_address = m.opt_string("ipaddress");
      let rediscover_on_update: bool = m.get_flag("rediscover-on-update");
      let template_id = m.opt_string("template-id");
      let output_opt = m.opt_str("output");
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
        output_opt,
      )
      .await?;
    }
    Some((other, _)) => bail!("Unknown 'update' subcommand: {other}"),
    None => bail!("No 'update' subcommand provided"),
  }
  Ok(())
}
