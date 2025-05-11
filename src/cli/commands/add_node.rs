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
  hw_inventory_opt: Option<HWInventoryByLocationList>,
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
  backend.post_nodes(shasta_token, components).await?;

  log::info!("Node saved '{}'", id);

  // Add hardware inventory
  if let Some(hw_inventory) = hw_inventory_opt {
    log::info!("Adding hardware inventory for '{}'", id);
    backend
      .post_inventory_hardware(&shasta_token, hw_inventory)
      .await?;
  }

  // Add node to group
  backend.post_member(shasta_token, group, id).await?;

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

  Ok(())
}
