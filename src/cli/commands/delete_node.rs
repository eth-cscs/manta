use manta_backend_dispatcher::{
  error::Error, interfaces::hsm::component::ComponentTrait,
};

use crate::{
  common::authentication::get_api_token,
  manta_backend_dispatcher::StaticBackendDispatcher,
};

pub async fn exec(
  backend: &StaticBackendDispatcher,
  site_name: &str,
  id: &str,
) -> Result<(), Error> {
  let auth_token = get_api_token(backend, site_name).await.unwrap();
  // Delete node
  backend.delete_node(&auth_token, id).await?;

  // Delete hsm hardware inventory related to a node
  //
  // Delete hsm network interfaces related to a node
  //
  // Delete hsm redfish interfaces related to a node
  //
  // Delete BSS boot parameters related to a node

  println!("Node deleted '{}'", id);

  Ok(())
}
