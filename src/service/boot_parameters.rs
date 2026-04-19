use anyhow::Error;
use manta_backend_dispatcher::{
  interfaces::bss::BootParametersTrait,
  types::bss::BootParameters,
};

use crate::common;
use crate::common::app_context::InfraContext;

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
