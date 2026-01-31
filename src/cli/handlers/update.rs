use crate::cli::commands::{update_boot_parameters, update_redfish_endpoint};
use crate::common::kafka::Kafka;
use crate::manta_backend_dispatcher::StaticBackendDispatcher;
use anyhow::Error;
use clap::ArgMatches;

pub async fn handle_update(
  cli_update: &ArgMatches,
  backend: &StaticBackendDispatcher,
  site_name: &str,
  kafka_audit_opt: Option<&Kafka>,
) -> Result<(), Error> {
  if let Some(cli_update_boot_parameters) =
    cli_update.subcommand_matches("boot-parameters")
  {
    let hosts: &String = cli_update_boot_parameters
      .get_one("hosts")
      .expect("ERROR - 'hosts' argument is mandatory");
    let params: Option<&String> = cli_update_boot_parameters.get_one("params");
    let kernel: Option<&String> = cli_update_boot_parameters.get_one("kernel");
    let initrd: Option<&String> = cli_update_boot_parameters.get_one("initrd");

    update_boot_parameters::exec(
      backend,
      site_name,
      hosts,
      None,
      None,
      params,
      kernel,
      initrd,
      kafka_audit_opt,
    )
    .await?;
  } else if let Some(cli_update_redfish_endpoint) =
    cli_update.subcommand_matches("redfish-endpoint")
  {
    let id: String = cli_update_redfish_endpoint
      .get_one("id")
      .cloned()
      .expect("ERROR - 'id' argument is mandatory");
    let name: Option<String> =
      cli_update_redfish_endpoint.get_one("name").cloned();
    let hostname: Option<String> =
      cli_update_redfish_endpoint.get_one("hostname").cloned();
    let domain: Option<String> =
      cli_update_redfish_endpoint.get_one("domain").cloned();
    let fqdn: Option<String> =
      cli_update_redfish_endpoint.get_one("fqdn").cloned();
    let enabled: bool = cli_update_redfish_endpoint.get_flag("enabled");
    let user: Option<String> =
      cli_update_redfish_endpoint.get_one("user").cloned();
    let password: Option<String> =
      cli_update_redfish_endpoint.get_one("password").cloned();
    let use_ssdp: bool = cli_update_redfish_endpoint.get_flag("use-ssdp");
    let mac_required: bool =
      cli_update_redfish_endpoint.get_flag("mac-required");
    let mac_addr: Option<String> =
      cli_update_redfish_endpoint.get_one("macaddr").cloned();
    let ip_address: Option<String> =
      cli_update_redfish_endpoint.get_one("ipaddress").cloned();
    let rediscover_on_update: bool =
      cli_update_redfish_endpoint.get_flag("rediscover-on-update");
    let template_id: Option<String> =
      cli_update_redfish_endpoint.get_one("template-id").cloned();

    update_redfish_endpoint::exec(
      backend,
      site_name,
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
  Ok(())
}
