use manta_backend_dispatcher::{
  error::Error, interfaces::hsm::component::ComponentTrait,
};

use crate::manta_backend_dispatcher::StaticBackendDispatcher;

pub async fn exec(
  backend: &StaticBackendDispatcher,
  auth_token: &str,
  id: &str,
) -> Result<(), Error> {
  // Delete node
  backend.delete_node(auth_token, id).await?;

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
