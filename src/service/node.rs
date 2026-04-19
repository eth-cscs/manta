use anyhow::{Error, bail};
use csm_rs::node::types::NodeDetails;

use crate::common;
use crate::common::app_context::InfraContext;

/// Typed parameters for fetching node details.
pub struct GetNodesParams {
  pub xname: String,
  pub include_siblings: bool,
  pub status_filter: Option<String>,
}

/// Fetch node details for the given xname expression.
pub async fn get_nodes(
  infra: &InfraContext<'_>,
  token: &str,
  params: &GetNodesParams,
) -> Result<Vec<NodeDetails>, Error> {
  let node_list = common::node_ops::resolve_hosts_expression(
    infra.backend,
    token,
    &params.xname,
    params.include_siblings,
  )
  .await?;

  if node_list.is_empty() {
    bail!("The list of nodes to operate is empty. Nothing to do");
  }

  let mut node_details_list = csm_rs::node::utils::get_node_details(
    token,
    infra.shasta_base_url,
    infra.shasta_root_cert,
    node_list.to_vec(),
  )
  .await
  .map_err(|e| anyhow::anyhow!("{e}"))?;

  // Apply status filter
  if let Some(ref status) = params.status_filter {
    node_details_list.retain(|nd| {
      nd.power_status.eq_ignore_ascii_case(status)
        || nd.configuration_status.eq_ignore_ascii_case(status)
    });
  }

  node_details_list.sort_by(|a, b| a.xname.cmp(&b.xname));

  Ok(node_details_list)
}

/// Compute a summary status from a list of node details.
///
/// Priority order: FAILED > OFF > ON > STANDBY > UNCONFIGURED > OK
pub fn compute_summary_status(nodes: &[NodeDetails]) -> &'static str {
  if nodes
    .iter()
    .any(|n| n.configuration_status.eq_ignore_ascii_case("failed"))
  {
    "FAILED"
  } else if nodes
    .iter()
    .any(|n| n.power_status.eq_ignore_ascii_case("OFF"))
  {
    "OFF"
  } else if nodes
    .iter()
    .any(|n| n.power_status.eq_ignore_ascii_case("on"))
  {
    "ON"
  } else if nodes
    .iter()
    .any(|n| n.power_status.eq_ignore_ascii_case("standby"))
  {
    "STANDBY"
  } else if nodes
    .iter()
    .any(|n| !n.configuration_status.eq_ignore_ascii_case("configured"))
  {
    "UNCONFIGURED"
  } else {
    "OK"
  }
}
