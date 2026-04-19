use anyhow::{Context, Error};
use manta_backend_dispatcher::interfaces::bss::BootParametersTrait;
use manta_backend_dispatcher::types::bss::BootParameters;

use crate::common;
use crate::common::app_context::InfraContext;

/// Typed parameters for fetching kernel boot parameters.
pub struct GetKernelParametersParams {
  pub hsm_group: Option<String>,
  pub nodes: Option<String>,
  pub settings_hsm_group_name: Option<String>,
}

/// Fetch kernel boot parameters for the specified nodes.
///
/// Resolves target nodes from HSM group or node list, then
/// fetches their BSS boot parameters.
pub async fn get_kernel_parameters(
  infra: &InfraContext<'_>,
  token: &str,
  params: &GetKernelParametersParams,
) -> Result<Vec<BootParameters>, Error> {
  let xname_vec = common::node_ops::resolve_target_nodes(
    infra.backend,
    token,
    params.nodes.as_deref(),
    params.hsm_group.as_deref(),
    params.settings_hsm_group_name.as_deref(),
  )
  .await?;

  let boot_parameter_vec = infra.backend
    .get_bootparameters(token, &xname_vec)
    .await
    .context("Could not get boot parameters")?;

  Ok(boot_parameter_vec)
}
