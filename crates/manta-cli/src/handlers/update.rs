//! Routes `manta update *` subcommands to their exec functions.

use crate::commands::{update_boot_parameters, update_redfish_endpoint};
use crate::common::authentication::get_api_token;
use crate::common::clap_ext::ArgMatchesExt;
use anyhow::{Context, Error, bail};
use clap::ArgMatches;
use crate::common::app_context::AppContext;

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
        ctx,
        &token,
        update_boot_parameters::ExecParams {
          xnames: hosts,
          nids: None,
          macs: None,
          boot_params: params,
          kernel,
          initrd,
          output: output_opt,
        },
      )
      .await?;
    }
    Some(("redfish-endpoints", m)) => {
      let id = m
        .opt_string("id")
        .context("The 'id' argument is mandatory")?;
      let params = manta_shared::shared::params::redfish_endpoints::UpdateRedfishEndpointParams {
        id,
        name: m.opt_string("name"),
        hostname: m.opt_string("hostname"),
        domain: m.opt_string("domain"),
        fqdn: m.opt_string("fqdn"),
        enabled: m.get_flag("enabled"),
        user: m.opt_string("user"),
        password: m.opt_string("password"),
        use_ssdp: m.get_flag("use-ssdp"),
        mac_required: m.get_flag("mac-required"),
        mac_addr: m.opt_string("macaddr"),
        ip_address: m.opt_string("ipaddress"),
        rediscover_on_update: m.get_flag("rediscover-on-update"),
        template_id: m.opt_string("template-id"),
      };
      update_redfish_endpoint::exec(ctx, &token, params, m.opt_str("output"))
        .await?;
    }
    Some((other, _)) => bail!("Unknown 'update' subcommand: {other}"),
    None => bail!("No 'update' subcommand provided"),
  }
  Ok(())
}
