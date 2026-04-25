use manta_backend_dispatcher::error::Error;
use csm_rs::node::types::NodeDetails;
use manta_backend_dispatcher::interfaces::hsm::component::ComponentTrait;
use manta_backend_dispatcher::interfaces::hsm::group::GroupTrait;
use manta_backend_dispatcher::interfaces::hsm::hardware_inventory::HardwareInventory;
use manta_backend_dispatcher::types::{
  ComponentArrayPostArray, ComponentCreate, HWInventoryByLocationList,
};
use std::{fs::File, io::BufReader, path::PathBuf};

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
  .await
  .map_err(|e| Error::Message(e.to_string()))?;

  if node_list.is_empty() {
    return Err(Error::BadRequest(
      "The list of nodes to operate is empty. Nothing to do".to_string(),
    ));
  }

  let mut node_details_list = csm_rs::node::utils::get_node_details(
    token,
    infra.shasta_base_url,
    infra.shasta_root_cert,
    node_list.to_vec(),
  )
  .await
  .map_err(|e: csm_rs::error::Error| Error::Message(e.to_string()))?;

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

/// Delete a node by its xname/ID.
pub async fn delete_node(
  infra: &InfraContext<'_>,
  token: &str,
  id: &str,
) -> Result<(), Error> {
  infra.backend.delete_node(token, id).await.map(|_| ())
}

/// Register a new node, optionally add hardware inventory,
/// and assign it to an HSM group.
///
/// Rolls back (deletes the node) if any step after creation fails.
pub async fn add_node(
  infra: &InfraContext<'_>,
  token: &str,
  id: &str,
  group: &str,
  enabled: bool,
  arch_opt: Option<String>,
  hardware_file_path: Option<&PathBuf>,
) -> Result<(), Error> {
  let backend = infra.backend;

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

  if let Err(error) = backend.post_nodes(token, components).await {
    return Err(error.into());
  }

  tracing::info!("Node saved '{}'", id);

  // Parse and add hardware inventory if provided
  let hw_inventory_opt: Option<HWInventoryByLocationList> =
    if let Some(hardware_file) = hardware_file_path {
      let file = match File::open(hardware_file) {
        Ok(f) => f,
        Err(e) => {
          rollback_node(backend, token, id).await;
          return Err(e.into());
        }
      };
      let reader = BufReader::new(file);
      let hw_inventory_value: serde_json::Value =
        match serde_json::from_reader(reader) {
          Ok(v) => v,
          Err(e) => {
            rollback_node(backend, token, id).await;
            return Err(e.into());
          }
        };
      Some(
        match serde_json::from_value::<HWInventoryByLocationList>(
          hw_inventory_value,
        ) {
          Ok(v) => v,
          Err(e) => {
            rollback_node(backend, token, id).await;
            return Err(e.into());
          }
        },
      )
    } else {
      None
    };

  if let Some(hw_inventory) = hw_inventory_opt {
    tracing::info!("Adding hardware inventory for '{}'", id);
    if let Err(error) = backend
      .post_inventory_hardware(token, hw_inventory)
      .await
    {
      rollback_node(backend, token, id).await;
      return Err(error.into());
    }
  }

  // Add node to group
  if let Err(error) = backend.post_member(token, group, id).await {
    rollback_node(backend, token, id).await;
    return Err(error.into());
  }

  Ok(())
}

/// Rollback helper: attempt to delete a node that was partially created.
async fn rollback_node(
  backend: &crate::manta_backend_dispatcher::StaticBackendDispatcher,
  token: &str,
  id: &str,
) {
  tracing::warn!("Rolling back: attempting to delete node '{}'", id);
  let delete_node_rslt = backend.delete_node(token, id).await;
  if delete_node_rslt.is_ok() {
    tracing::info!("Rollback: node '{}' deleted", id);
  }
}
