use manta_backend_dispatcher::{
  error::Error, interfaces::hsm::hardware_inventory::HardwareInventory,
  types::HWInventoryByLocationList,
};

use StaticBackendDispatcher::*;

use serde_json::Value;

use crate::manta_backend_dispatcher::StaticBackendDispatcher;

impl HardwareInventory for StaticBackendDispatcher {
  async fn get_inventory_hardware(
    &self,
    auth_token: &str,
    xname: &str,
  ) -> Result<Value, Error> {
    match self {
      CSM(b) => b.get_inventory_hardware(auth_token, xname).await,
      OCHAMI(b) => b.get_inventory_hardware(auth_token, xname).await,
    }
  }

  async fn get_inventory_hardware_query(
    &self,
    auth_token: &str,
    xname: &str,
    r#type: Option<&str>,
    children: Option<bool>,
    parents: Option<bool>,
    partition: Option<&str>,
    format: Option<&str>,
  ) -> Result<Value, Error> {
    match self {
      CSM(b) => {
        b.get_inventory_hardware_query(
          auth_token, xname, r#type, children, parents, partition, format,
        )
        .await
      }
      OCHAMI(b) => {
        b.get_inventory_hardware_query(
          auth_token, xname, r#type, children, parents, partition, format,
        )
        .await
      }
    }
  }

  async fn post_inventory_hardware(
    &self,
    auth_token: &str,
    hardware: HWInventoryByLocationList,
  ) -> Result<Value, Error> {
    match self {
      CSM(b) => b.post_inventory_hardware(auth_token, hardware).await,
      OCHAMI(b) => b.post_inventory_hardware(auth_token, hardware).await,
    }
  }
}
