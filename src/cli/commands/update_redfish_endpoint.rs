use anyhow::Error;
use manta_backend_dispatcher::{
    interfaces::hsm::redfish_endpoint::RedfishEndpointTrait,
    types::hsm::inventory::RedfishEndpoint,
};

use crate::{
    common::authentication::get_api_token,
    manta_backend_dispatcher::StaticBackendDispatcher,
};

pub async fn exec(
    backend: &StaticBackendDispatcher,
    site_name: &str,
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
    let shasta_token = get_api_token(backend, site_name).await?;

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

    Ok(())
}
