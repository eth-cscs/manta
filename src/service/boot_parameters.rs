use anyhow::{Context, Error};
use manta_backend_dispatcher::{
  interfaces::bss::BootParametersTrait,
  types::bss::BootParameters,
};

use crate::common;
use crate::common::app_context::InfraContext;
use crate::common::authorization::validate_target_hsm_members;

/// Typed parameters for fetching boot parameters.
pub struct GetBootParametersParams {
  pub hsm_group: Option<String>,
  pub nodes: Option<String>,
  pub settings_hsm_group_name: Option<String>,
}

/// Fetch boot parameters for the specified nodes.
///
/// Resolves target nodes from HSM group or node list, then
/// fetches their BSS boot parameters.
pub async fn get_boot_parameters(
  infra: &InfraContext<'_>,
  token: &str,
  params: &GetBootParametersParams,
) -> Result<Vec<BootParameters>, Error> {
  let xname_vec = common::node_ops::resolve_target_nodes(
    infra.backend,
    token,
    params.nodes.as_deref(),
    params.hsm_group.as_deref(),
    params.settings_hsm_group_name.as_deref(),
  )
  .await?;

  log::info!("Get boot parameters");

  infra.backend
    .get_bootparameters(token, &xname_vec)
    .await
    .map_err(|e| anyhow::anyhow!(e))
}

/// Delete boot parameters for the specified hosts.
pub async fn delete_boot_parameters(
  infra: &InfraContext<'_>,
  token: &str,
  hosts: Vec<String>,
) -> Result<(), Error> {
  let boot_parameters = BootParameters {
    hosts,
    macs: None,
    nids: None,
    params: String::new(),
    kernel: String::new(),
    initrd: String::new(),
    cloud_init: None,
  };

  infra
    .backend
    .delete_bootparameters(token, &boot_parameters)
    .await
    .context("Failed to delete boot parameters")?;

  Ok(())
}

/// Add (create) boot parameters for specified nodes.
pub async fn add_boot_parameters(
  infra: &InfraContext<'_>,
  token: &str,
  boot_parameters: &BootParameters,
) -> Result<(), Error> {
  infra
    .backend
    .add_bootparameters(token, boot_parameters)
    .await?;
  Ok(())
}

/// Typed parameters for updating boot parameters.
pub struct UpdateBootParametersParams {
  pub hosts: Vec<String>,
  pub nids: Option<Vec<u32>>,
  pub macs: Option<Vec<String>>,
  pub params: String,
  pub kernel: String,
  pub initrd: String,
}

/// Update boot parameters for specified nodes.
///
/// Validates target HSM membership, then updates BSS boot parameters.
pub async fn update_boot_parameters(
  infra: &InfraContext<'_>,
  token: &str,
  params: UpdateBootParametersParams,
) -> Result<(), Error> {
  validate_target_hsm_members(infra.backend, token, &params.hosts).await?;

  let boot_parameters = BootParameters {
    hosts: params.hosts,
    macs: params.macs,
    nids: params.nids,
    params: params.params,
    kernel: params.kernel,
    initrd: params.initrd,
    cloud_init: None,
  };

  log::debug!("new boot params: {:#?}", boot_parameters);

  infra
    .backend
    .update_bootparameters(token, &boot_parameters)
    .await?;

  Ok(())
}
