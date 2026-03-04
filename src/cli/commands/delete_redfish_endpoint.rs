use anyhow::Context;

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

  result.with_context(|| {
    format!("Failed to delete redfish endpoint for id '{}'", id)
  })?;

  println!("Redfish endpoint for id '{}' deleted successfully", id);

  Ok(())
}
