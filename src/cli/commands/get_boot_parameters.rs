use manta_backend_dispatcher::{
  interfaces::bss::BootParametersTrait,
  types::bss::BootParameters,
};

use crate::{common, manta_backend_dispatcher::StaticBackendDispatcher};

/// Display boot parameters for specified nodes.
pub async fn exec(
  backend: &StaticBackendDispatcher,
  site_name: &str,
  cli_get_boot_parameters: &clap::ArgMatches,
  settings_hsm_group_name_opt: Option<&str>,
) -> Result<Vec<BootParameters>, anyhow::Error> {
  let shasta_token =
    common::authentication::get_api_token(backend, site_name).await?;

  let hsm_group_name_arg_opt: Option<&str> = cli_get_boot_parameters
    .get_one::<String>("hsm-group")
    .map(String::as_str);

  let nodes_arg: Option<&str> = cli_get_boot_parameters
    .get_one::<String>("nodes")
    .map(String::as_str);

  let xname_vec = common::node_ops::resolve_target_nodes(
    backend,
    &shasta_token,
    nodes_arg,
    hsm_group_name_arg_opt,
    settings_hsm_group_name_opt,
  )
  .await?;

  // Get BSS boot parameters
  println!("Get boot parameters");

  Ok(
    backend
      .get_bootparameters(&shasta_token, &xname_vec)
      .await?,
  )
}
