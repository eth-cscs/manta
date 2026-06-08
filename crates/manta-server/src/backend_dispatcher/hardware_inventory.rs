//! `HardwareInventory` impl for `StaticBackendDispatcher`.

use super::*;

impl HardwareInventory for StaticBackendDispatcher {
  async fn get_inventory_hardware(
    &self,
    auth_token: &str,
    xname: &str,
  ) -> Result<NodeSummary, Error> {
    dispatch!(self, get_inventory_hardware, auth_token, xname)
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
  ) -> Result<HWInventory, Error> {
    dispatch!(
      self,
      get_inventory_hardware_query,
      auth_token,
      xname,
      r#type,
      children,
      parents,
      partition,
      format
    )
  }

  async fn post_inventory_hardware(
    &self,
    auth_token: &str,
    hardware: HWInventoryByLocationList,
  ) -> Result<HsmActionResponse, Error> {
    dispatch!(self, post_inventory_hardware, auth_token, hardware)
  }
}
