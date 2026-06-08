//! HSM node queries, registration, and deletion, with rollback on partial failure.

use manta_backend_dispatcher::error::Error;
use manta_backend_dispatcher::types::{
  ComponentArrayPostArray, ComponentCreate, HWInventoryByLocationList,
};
use manta_shared::types::dto::NodeDetails;
use std::path::PathBuf;

use crate::server::common::app_context::InfraContext;
use crate::service::authorization::validate_user_group_members_access;
use crate::service::node_details;
use crate::service::node_ops::from_user_hosts_expression_to_xname_vec;
pub use manta_shared::types::params::node::GetNodesParams;

/// Fetch HSM node details for the targets named by
/// `params.host_expression`.
///
/// The expression is parsed by
/// [`crate::service::node_ops::from_user_hosts_expression_to_xname_vec`];
/// when `params.include_siblings` is set, the resulting xnames are
/// expanded to cover every node on the same BMC. Access to the
/// resolved set is validated before the (relatively slow) per-node
/// detail fetch. The optional `status_filter` matches case-insensitively
/// against either the power or configuration status. Results are
/// sorted by xname for stable output.
pub async fn get_nodes(
  infra: &InfraContext<'_>,
  token: &str,
  params: &GetNodesParams,
) -> Result<Vec<NodeDetails>, Error> {
  let node_list = from_user_hosts_expression_to_xname_vec(
    infra,
    token,
    &params.host_expression,
    params.include_siblings,
  )
  .await?;

  if node_list.is_empty() {
    return Err(Error::BadRequest(
      "The list of nodes to operate is empty. Nothing to do".to_string(),
    ));
  }

  // Validate xnames
  validate_user_group_members_access(infra, token, &node_list).await?;

  let mut node_details_list =
    node_details::get_node_details(infra, token, &node_list).await?;

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

// `compute_summary_status` moved to `manta_shared::types::cluster_status` —
// only CLI display code calls it.

/// Remove the HSM component with id `id` (typically an xname).
///
/// The caller's group access to the node is validated before the
/// delete is dispatched.
pub async fn delete_node(
  infra: &InfraContext<'_>,
  token: &str,
  id: &str,
) -> Result<(), Error> {
  validate_user_group_members_access(infra, token, &[id.to_string()]).await?;

  infra.delete_node(token, id).await
}

/// Register a new HSM component, attach an optional hardware
/// inventory file, and add it to the named group.
///
/// The flow is three writes: `post_nodes`, then (if
/// `hardware_file_path` is supplied) `post_inventory_hardware` after
/// parsing the JSON file, then `post_member`. Each failure after the
/// initial create rolls back by deleting the node, so a partial
/// failure does not leave a stub component behind. Hardware-file
/// parse errors are reported as the original IO / serde error, with
/// the same rollback applied.
pub async fn add_node(
  infra: &InfraContext<'_>,
  token: &str,
  id: &str,
  group: &str,
  enabled: bool,
  arch_opt: Option<String>,
  hardware_file_path: Option<&PathBuf>,
) -> Result<(), Error> {
  validate_user_group_members_access(infra, token, &[id.to_string()]).await?;

  // Create node
  let component = ComponentCreate {
    id: id.to_string(),
    state: "Unknown".to_string(),
    flag: None,
    enabled: Some(enabled),
    software_status: None,
    role: None,
    sub_role: None,
    nid: None,
    subtype: None,
    net_type: None,
    arch: arch_opt,
    class: None,
  };

  let components = ComponentArrayPostArray {
    components: vec![component],
    force: Some(true),
  };

  infra.post_nodes(token, components).await?;

  tracing::info!("Node saved '{}'", id);

  // Parse and add hardware inventory if provided.
  //
  // HW inventory files are operator-supplied JSON that can run to
  // several MB. Reading them with the sync `std::fs::File` +
  // `serde_json::from_reader` chain parked the Tokio worker for the
  // duration of the read and parse, stalling unrelated requests
  // queued behind it on the same worker. `tokio::fs::read` does the
  // I/O on a blocking pool; the in-memory `from_slice` parse stays
  // on the worker but is bounded by file size.
  let hw_inventory_opt: Option<HWInventoryByLocationList> =
    if let Some(hardware_file) = hardware_file_path {
      match read_hw_inventory(hardware_file).await {
        Ok(inv) => Some(inv),
        Err(e) => {
          rollback_node(infra, token, id).await;
          return Err(e);
        }
      }
    } else {
      None
    };

  if let Some(hw_inventory) = hw_inventory_opt {
    tracing::info!("Adding hardware inventory for '{}'", id);
    if let Err(error) = infra.post_inventory_hardware(token, hw_inventory).await
    {
      rollback_node(infra, token, id).await;
      return Err(error);
    }
  }

  // Add node to group
  if let Err(error) = infra.post_member(token, group, id).await {
    rollback_node(infra, token, id).await;
    return Err(error);
  }

  Ok(())
}

/// Read and parse a hardware-inventory JSON file off the Tokio
/// reactor. The two-step `Value` → `from_value` round-trip is kept
/// so the surfaced parse error still names the bad field (csm-rs
/// uses `#[serde(rename = "ID")]` etc. and the direct `from_slice`
/// path produces less helpful errors when a key is mistyped).
async fn read_hw_inventory(
  path: &PathBuf,
) -> Result<HWInventoryByLocationList, Error> {
  let bytes = tokio::fs::read(path).await?;
  let value: serde_json::Value = serde_json::from_slice(&bytes)?;
  let inv = serde_json::from_value::<HWInventoryByLocationList>(value)?;
  Ok(inv)
}

/// Rollback helper: attempt to delete a node that was partially created.
async fn rollback_node(infra: &InfraContext<'_>, token: &str, id: &str) {
  tracing::warn!("Rolling back: attempting to delete node '{}'", id);
  let delete_node_rslt = infra.delete_node(token, id).await;
  if delete_node_rslt.is_ok() {
    tracing::info!("Rollback: node '{}' deleted", id);
  }
}
