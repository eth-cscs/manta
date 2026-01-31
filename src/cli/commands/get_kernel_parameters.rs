use crate::common::authorization::get_groups_names_available;
use manta_backend_dispatcher::{
  error::Error,
  interfaces::{
    bss::BootParametersTrait, hsm::component::ComponentTrait,
    hsm::group::GroupTrait,
  },
  types::bss::BootParameters,
};

use crate::{
  common::{self},
  manta_backend_dispatcher::StaticBackendDispatcher,
};

pub async fn exec(
  backend: &StaticBackendDispatcher,
  site_name: &str,
  cli_get_kernel_parameters: &clap::ArgMatches,
  settings_hsm_group_name_opt: Option<&String>,
) -> Result<(), Error> {
  let shasta_token = common::authentication::get_api_token(backend, site_name).await?;

  let hsm_group_name_arg_opt: Option<&String> =
    cli_get_kernel_parameters.get_one("hsm-group");
  let filter_opt: Option<&String> = cli_get_kernel_parameters.get_one("filter");
  let output: &String = cli_get_kernel_parameters
    .get_one("output")
    .expect("ERROR - output value missing");

  let nodes: String = if hsm_group_name_arg_opt.is_some() {
    let hsm_group_name_vec = get_groups_names_available(
      backend,
      &shasta_token,
      hsm_group_name_arg_opt,
      settings_hsm_group_name_opt,
    )
    .await
    .map_err(|e| Error::Message(e.to_string()))?;
    let hsm_members_rslt = backend
      .get_member_vec_from_group_name_vec(&shasta_token, &hsm_group_name_vec)
      .await;
    match hsm_members_rslt {
      Ok(hsm_members) => hsm_members.join(","),
      Err(e) => {
        eprintln!(
          "ERROR - could not fetch HSM groups members. Reason:\n{}",
          e.to_string()
        );
        std::process::exit(1);
      }
    }
  } else {
    cli_get_kernel_parameters
      .get_one::<String>("nodes")
      .expect("Neither HSM group nor nodes defined")
      .clone()
  };

  // Get BSS boot parameters

  // Convert user input to xname
  let node_metadata_available_vec = backend
    .get_node_metadata_available(&shasta_token)
    .await
    .map_err(|e| {
      Error::Message(format!(
        "ERROR - Could not get node metadata. Reason:\n{e}\nExit"
      ))
    })?;

  let xname_vec = common::node_ops::from_hosts_expression_to_xname_vec(
    &nodes,
    false,
    node_metadata_available_vec,
  )
  .await
  .map_err(|e| {
    Error::Message(format!(
      "ERROR - Could not convert user input to list of xnames. Reason:\n{e}"
    ))
  })?;

  let boot_parameter_vec: Vec<BootParameters> = backend
    .get_bootparameters(&shasta_token, &xname_vec)
    .await
    .unwrap();

  match output.as_str() {
    "json" => println!(
      "{}",
      serde_json::to_string_pretty(&boot_parameter_vec).unwrap()
    ),
    "table" => {
      common::kernel_parameters_ops::print_table(boot_parameter_vec, filter_opt)
    }
    _ => {
      return Err(Error::Message(
        "ERROR - 'output' argument value missing or not supported".to_string(),
      ));
    }
  }

  Ok(())
}
