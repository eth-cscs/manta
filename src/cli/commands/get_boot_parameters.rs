use manta_backend_dispatcher::{
  error::Error,
  interfaces::{bss::BootParametersTrait, hsm::component::ComponentTrait},
  types::bss::BootParameters,
};

use crate::{common, manta_backend_dispatcher::StaticBackendDispatcher};

pub async fn exec(
  backend: &StaticBackendDispatcher,
  shasta_token: &str,
  hosts_expression: &str,
  _nids: Option<&String>,
  _macs: Option<&String>,
  _params: Option<&String>,
  _kernel: Option<&String>,
  _initrd: Option<&String>,
) -> Result<Vec<BootParameters>, Error> {
  // Get BSS boot parameters
  println!("Get boot parameters");

  // Convert user input to xname
  let node_metadata_available_vec = backend
    .get_node_metadata_available(shasta_token)
    .await
    .unwrap_or_else(|e| {
      eprintln!("ERROR - Could not get node metadata. Reason:\n{e}\nExit");
      std::process::exit(1);
    });

  let xname_vec = common::node_ops::from_hosts_expression_to_xname_vec(
    hosts_expression,
    false,
    node_metadata_available_vec,
  )
  .await
  .unwrap_or_else(|e| {
    eprintln!(
      "ERROR - Could not convert user input to list of xnames. Reason:\n{}",
      e
    );
    std::process::exit(1);
  });

  /* let boot_parameter_vec: Vec<BootParameters> = backend
    .get_bootparameters(shasta_token, &xname_vec)
    .await
    .unwrap();
  let hosts: Vec<String> = xnames.split(',').map(String::from).collect(); */

  backend.get_bootparameters(shasta_token, &xname_vec).await
}
