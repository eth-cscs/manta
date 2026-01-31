use crate::{
    common::authentication::get_api_token,
    manta_backend_dispatcher::StaticBackendDispatcher,
};
use manta_backend_dispatcher::interfaces::hsm::redfish_endpoint::RedfishEndpointTrait;

pub async fn exec(
    backend: &StaticBackendDispatcher,
    site_name: &str,
    id: &str,
) -> Result<(), anyhow::Error> {
    let shasta_token = get_api_token(backend, site_name).await?;

    let result = backend.delete_redfish_endpoint(&shasta_token, id).await;

    match result {
        Ok(_) => {
            println!("Redfish endpoint for id '{}' deleted successfully", id)
        }
        Err(error) => eprintln!("{}", error),
    }

    Ok(())
}
