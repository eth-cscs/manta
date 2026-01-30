use anyhow::Result;
use manta_backend_dispatcher::{
  interfaces::hsm::{
    component::ComponentTrait, group::GroupTrait,
    hardware_inventory::HardwareInventory,
  },
  types::{
    ComponentArrayPostArray, ComponentCreate, HWInventoryByLocationList,
  },
};
use std::{fs::File, io::BufReader, path::PathBuf};

use crate::{
  common::{audit::Audit, jwt_ops, kafka::Kafka},
  manta_backend_dispatcher::StaticBackendDispatcher,
};

pub async fn exec(
  backend: &StaticBackendDispatcher,
  shasta_token: &str,
  id: &str,
  group: &str,
  enabled: bool,
  arch_opt: Option<String>,
  hardware_file_path: Option<&PathBuf>,
  kafka_audit_opt: Option<&Kafka>,
) -> Result<()> {
  // Create node api payload
  let component: ComponentCreate = ComponentCreate {
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

  // Add node to backend
  if let Err(error) = backend.post_nodes(shasta_token, components).await {
    eprintln!(
      "ERROR - operation to add node '{id}' to group '{group}' failed. Reason:\n{error}"
    );
    return Err(error.into());
  }

  log::info!("Node saved '{}'", id);

  let hw_inventory_opt: Option<HWInventoryByLocationList> =
    if let Some(hardware_file) = hardware_file_path {
      let file = match File::open(hardware_file) {
        Ok(f) => f,
        Err(e) => {
          eprintln!("ERROR - Could not open hardware inventory file. Reason:\n{}", e);
          rollback(backend, shasta_token, id).await;
          return Err(e.into());
        }
      };
      let reader = BufReader::new(file);
      let hw_inventory_value: serde_json::Value =
        match serde_json::from_reader(reader) {
          Ok(v) => v,
          Err(e) => {
            eprintln!("ERROR - Could not parse hardware inventory file. Reason:\n{}", e);
            rollback(backend, shasta_token, id).await;
            return Err(e.into());
          }
        };
      Some(
        match serde_json::from_value::<HWInventoryByLocationList>(hw_inventory_value) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("ERROR - Could not parse hardware inventory file content. Reason:\n{}", e);
                rollback(backend, shasta_token, id).await;
                return Err(e.into());
            }
        }
      )
    } else {
      None
    };

  // Add hardware inventory
  if let Some(hw_inventory) = hw_inventory_opt {
    log::info!("Adding hardware inventory for '{}'", id);
    if let Err(error) = backend
      .post_inventory_hardware(&shasta_token, hw_inventory)
      .await
    {
        eprintln!(
            "ERROR - operation to add hardware inventory for node '{id}' failed. Reason:\n{error}\nRollback operation"
        );
        rollback(backend, shasta_token, id).await;
        return Err(error.into());
    }
  }

  // Add node to group
  if let Err(error) = backend.post_member(shasta_token, group, id).await {
      eprintln!(
          "ERROR - operation to add node '{id}' to group '{group}' failed. Reason:\n{error}\nRollback operation"
      );
      rollback(backend, shasta_token, id).await;
      return Err(error.into());
  }

  // Audit
  if let Some(kafka_audit) = kafka_audit_opt {
    let username = jwt_ops::get_name(shasta_token).unwrap_or_default();
    let user_id =
      jwt_ops::get_preferred_username(shasta_token).unwrap_or_default();

    let msg_json = serde_json::json!(
        { "user": {"id": user_id, "name": username}, "host": {"hostname": id}, "group": [], "message": "add node"});

    let msg_data = serde_json::to_string(&msg_json)
      .expect("Could not serialize audit message data");

    if let Err(e) = kafka_audit.produce_message(msg_data.as_bytes()).await {
      log::warn!("Failed producing messages: {}", e);
    }
  }

  println!("Node '{}' created and added to group '{}'", id, group);

  Ok(())
}

async fn rollback(backend: &StaticBackendDispatcher, shasta_token: &str, id: &str) {
    let delete_node_rslt = backend
        .delete_node(shasta_token, id)
        .await;
    eprintln!("Try to delete node '{}'", id);
    if delete_node_rslt.is_ok() {
        eprintln!("Node '{}' deleted", id);
    }
}
