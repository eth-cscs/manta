//! HSM node queries, registration, and deletion, with rollback on partial failure.

use csm_rs::node::types::NodeDetails;
use manta_backend_dispatcher::error::Error;
use manta_backend_dispatcher::types::{
  ComponentArrayPostArray, ComponentCreate, HWInventoryByLocationList,
};
use std::{fs::File, io::BufReader, path::PathBuf};

use crate::server::common::app_context::InfraContext;
use crate::service::authorization::validate_user_group_members_access;
use crate::service::node_ops::from_hosts_expression_to_xname_vec;
pub use manta_shared::types::params::node::GetNodesParams;

/// Fetch HSM node details for the targets named by
/// `params.host_expression`.
///
/// The expression is parsed by [`from_hosts_expression_to_xname_vec`];
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
  let node_metadata_available_vec =
    infra.get_node_metadata_available(token).await?;

  let node_list = from_hosts_expression_to_xname_vec(
    &params.host_expression,
    params.include_siblings,
    &node_metadata_available_vec,
  )?;

  if node_list.is_empty() {
    return Err(Error::BadRequest(
      "The list of nodes to operate is empty. Nothing to do".to_string(),
    ));
  }

  // Validate xnames
  validate_user_group_members_access(infra, token, &node_list).await?;

  let mut node_details_list = csm_rs::node::utils::get_node_details(
    token,
    infra.shasta_base_url,
    infra.shasta_root_cert,
    infra.socks5_proxy,
    node_list.to_vec(),
  )
  .await
  .map_err(|e: csm_rs::error::Error| -> Error { e.into() })?;

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

  // Parse and add hardware inventory if provided
  let hw_inventory_opt: Option<HWInventoryByLocationList> =
    if let Some(hardware_file) = hardware_file_path {
      let file = match File::open(hardware_file) {
        Ok(f) => f,
        Err(e) => {
          rollback_node(infra, token, id).await;
          return Err(e.into());
        }
      };
      let reader = BufReader::new(file);
      let hw_inventory_value: serde_json::Value =
        match serde_json::from_reader(reader) {
          Ok(v) => v,
          Err(e) => {
            rollback_node(infra, token, id).await;
            return Err(e.into());
          }
        };
      Some(
        match serde_json::from_value::<HWInventoryByLocationList>(
          hw_inventory_value,
        ) {
          Ok(v) => v,
          Err(e) => {
            rollback_node(infra, token, id).await;
            return Err(e.into());
          }
        },
      )
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

/// Rollback helper: attempt to delete a node that was partially created.
async fn rollback_node(infra: &InfraContext<'_>, token: &str, id: &str) {
  tracing::warn!("Rolling back: attempting to delete node '{}'", id);
  let delete_node_rslt = infra.delete_node(token, id).await;
  if delete_node_rslt.is_ok() {
    tracing::info!("Rollback: node '{}' deleted", id);
  }
}
