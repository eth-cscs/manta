use anyhow::Error;
use clap::ArgMatches;
use crate::manta_backend_dispatcher::StaticBackendDispatcher;
use manta_backend_dispatcher::interfaces::hsm::redfish_endpoint::RedfishEndpointTrait;
use crate::common::{authorization::validate_target_hsm_members, authentication::get_api_token, kafka::Kafka};
use manta_backend_dispatcher::types::hsm::inventory::RedfishEndpoint;
use crate::cli::commands::update_boot_parameters;

pub async fn handle_update(
    cli_update: &ArgMatches,
    backend: &StaticBackendDispatcher,
    site_name: &str,
    kafka_audit_opt: Option<&Kafka>,
) -> Result<(), Error> {
    if let Some(cli_update_boot_parameters) = cli_update.subcommand_matches("boot-parameters") {
        let shasta_token = get_api_token(backend, site_name).await?;
        let hosts: &String = cli_update_boot_parameters
            .get_one("hosts")
            .expect("ERROR - 'hosts' argument is mandatory");
        let params: Option<&String> = cli_update_boot_parameters.get_one("params");
        let kernel: Option<&String> = cli_update_boot_parameters.get_one("kernel");
        let initrd: Option<&String> = cli_update_boot_parameters.get_one("initrd");
        let xname_vec = hosts
            .split(",")
            .map(|value| value.trim().to_string())
            .collect();
        validate_target_hsm_members(backend, &shasta_token, &xname_vec).await;
        let result = update_boot_parameters::exec(
            backend,
            &shasta_token,
            hosts,
            None,
            None,
            params,
            kernel,
            initrd,
            kafka_audit_opt,
        )
        .await;
        match result {
            Ok(_) => {}
            Err(error) => eprintln!("{}", error),
        }
    } else if let Some(cli_update_redfish_endpoint) = cli_update.subcommand_matches("redfish-endpoint")
    {
        let shasta_token = get_api_token(backend, site_name).await?;
        let id: String = cli_update_redfish_endpoint
            .get_one("id")
            .cloned()
            .expect("ERROR - 'id' argument is mandatory");
        let name: Option<String> = cli_update_redfish_endpoint.get_one("name").cloned();
        let hostname: Option<String> = cli_update_redfish_endpoint.get_one("hostname").cloned();
        let domain: Option<String> = cli_update_redfish_endpoint.get_one("domain").cloned();
        let fqdn: Option<String> = cli_update_redfish_endpoint.get_one("fqdn").cloned();
        let enabled: bool = cli_update_redfish_endpoint.get_flag("enabled");
        let user: Option<String> = cli_update_redfish_endpoint.get_one("user").cloned();
        let password: Option<String> = cli_update_redfish_endpoint.get_one("password").cloned();
        let use_ssdp: bool = cli_update_redfish_endpoint.get_flag("use-ssdp");
        let mac_required: bool = cli_update_redfish_endpoint.get_flag("mac-required");
        let mac_addr: Option<String> = cli_update_redfish_endpoint.get_one("macaddr").cloned();
        let ip_address: Option<String> = cli_update_redfish_endpoint.get_one("ipaddress").cloned();
        let rediscover_on_update: bool =
            cli_update_redfish_endpoint.get_flag("rediscover-on-update");
        let template_id: Option<String> =
            cli_update_redfish_endpoint.get_one("template-id").cloned();
        let redfish_endpoint = RedfishEndpoint {
            id,
            name,
            hostname,
            domain,
            fqdn,
            enabled: Some(enabled),
            user,
            password,
            use_ssdp: Some(use_ssdp),
            mac_required: Some(mac_required),
            mac_addr,
            ip_address,
            rediscover_on_update: Some(rediscover_on_update),
            template_id,
            r#type: None,
            uuid: None,
            discovery_info: None,
        };
        backend
            .update_redfish_endpoint(&shasta_token, &redfish_endpoint)
            .await?;
    }
    Ok(())
}
