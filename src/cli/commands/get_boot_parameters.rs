use anyhow::Context;

use crate::common::authorization::get_groups_names_available;
use manta_backend_dispatcher::{
  interfaces::{
    bss::BootParametersTrait, hsm::component::ComponentTrait,
    hsm::group::GroupTrait,
  },
  types::bss::BootParameters,
};

use crate::{common, manta_backend_dispatcher::StaticBackendDispatcher};

pub async fn exec(
  backend: &StaticBackendDispatcher,
  site_name: &str,
  cli_get_boot_parameters: &clap::ArgMatches,
  settings_hsm_group_name_opt: Option<&String>,
) -> Result<Vec<BootParameters>, anyhow::Error> {
  let shasta_token =
    common::authentication::get_api_token(backend, site_name).await?;

  let hsm_group_name_arg_opt: Option<&String> =
    cli_get_boot_parameters.get_one("hsm-group");
  let nodes: String = if hsm_group_name_arg_opt.is_some() {
    let hsm_group_name_vec = get_groups_names_available(
      backend,
      &shasta_token,
      hsm_group_name_arg_opt,
      settings_hsm_group_name_opt,
    )
    .await
    .context("Failed to get available HSM group names")?;
    backend
      .get_member_vec_from_group_name_vec(&shasta_token, &hsm_group_name_vec)
      .await
      .context("Could not fetch HSM groups members")?
      .join(",")
  } else {
    cli_get_boot_parameters
      .get_one::<String>("nodes")
      .ok_or_else(|| anyhow::anyhow!("Neither HSM group nor nodes defined"))?
      .clone()
  };

  // Get BSS boot parameters
  println!("Get boot parameters");

  // Convert user input to xname
  let node_metadata_available_vec = backend
    .get_node_metadata_available(&shasta_token)
    .await
    .context("Could not get node metadata")?;

  let xname_vec = common::node_ops::from_hosts_expression_to_xname_vec(
    &nodes,
    false,
    node_metadata_available_vec,
  )
  .await
  .context("Could not convert user input to list of xnames")?;

  Ok(
    backend
      .get_bootparameters(&shasta_token, &xname_vec)
      .await?,
  )
}
