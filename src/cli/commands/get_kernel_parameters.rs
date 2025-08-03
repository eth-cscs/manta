use manta_backend_dispatcher::{
  error::Error,
  interfaces::{bss::BootParametersTrait, hsm::component::ComponentTrait},
  types::bss::BootParameters,
};

use crate::{
  common::{self},
  manta_backend_dispatcher::StaticBackendDispatcher,
};

pub async fn exec(
  backend: &StaticBackendDispatcher,
  shasta_token: &str,
  hosts_expression: &str,
  filter: Option<&String>,
  output: &str,
) -> Result<(), Error> {
  // Get BSS boot parameters

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

  let boot_parameter_vec: Vec<BootParameters> = backend
    .get_bootparameters(shasta_token, &xname_vec)
    .await
    .unwrap();

  match output {
    "json" => println!(
      "{}",
      serde_json::to_string_pretty(&boot_parameter_vec).unwrap()
    ),
    "table" => {
      common::kernel_parameters_ops::print_table(boot_parameter_vec, filter)
    }
    _ => panic!("ERROR - 'output' argument value missing or not supported"),
  }

  Ok(())
}
