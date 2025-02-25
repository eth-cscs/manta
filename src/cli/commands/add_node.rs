use anyhow::Result;
use backend_dispatcher::{
    interfaces::hsm::{
        component::ComponentTrait, group::GroupTrait, hardware_inventory::HardwareInventory,
    },
    types::{ComponentArrayPostArray, ComponentCreate, HWInventoryByLocationList},
};

use crate::backend_dispatcher::StaticBackendDispatcher;

pub async fn exec(
    backend: &StaticBackendDispatcher,
    shasta_token: &str,
    id: &str,
    enabled: bool,
    arch_opt: Option<String>,
    hw_inventory: HWInventoryByLocationList,
    group: &str,
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

    log::info!("Node saved '{}'. Try to add hardware", id);

    // Add hardware inventory
    backend
        .post_inventory_hardware(&shasta_token, hw_inventory)
        .await?;

    Ok(())
}
