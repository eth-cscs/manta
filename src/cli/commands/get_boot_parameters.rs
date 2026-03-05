use anyhow::Context;

use crate::common::authorization::get_groups_names_available;
use manta_backend_dispatcher::{
  interfaces::{bss::BootParametersTrait, hsm::group::GroupTrait},
  types::bss::BootParameters,
};

use crate::{common, manta_backend_dispatcher::StaticBackendDispatcher};

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
  let xname_vec = common::node_ops::resolve_hosts_expression(
    backend,
    &shasta_token,
    &nodes,
    false,
  )
  .await?;

  Ok(
    backend
      .get_bootparameters(&shasta_token, &xname_vec)
      .await?,
  )
}
