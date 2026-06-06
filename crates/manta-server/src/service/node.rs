//! HSM node registration and deletion, with rollback on partial failure.

use manta_backend_dispatcher::error::Error;
use manta_backend_dispatcher::types::{
  ComponentArrayPostArray, ComponentCreate, HWInventoryByLocationList,
};
use std::{fs::File, io::BufReader, path::PathBuf};

use crate::server::common::app_context::InfraContext;

/// Delete a node by its xname/ID.
pub async fn delete_node(
  infra: &InfraContext<'_>,
  token: &str,
  id: &str,
) -> Result<(), Error> {
  infra.delete_node(token, id).await
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
