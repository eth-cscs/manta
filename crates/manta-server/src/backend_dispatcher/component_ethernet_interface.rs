//! [`ComponentEthernetInterfaceTrait`] impl for
//! [`StaticBackendDispatcher`].
//!
//! Forwards to HSM's
//! `/apis/smd/hsm/v2/Inventory/EthernetInterfaces` endpoints. Both
//! CSM and Ochami implement these natively.
//!
//! No HTTP handler currently calls these methods; this file exists so
//! the dispatcher covers the full trait surface. Without it, a handler
//! added later would silently hit the trait's "not implemented" default
//! even when the backend actually does implement the method — the same
//! class of bug that caused the `apply_sat_image_create_session` issue.

use manta_backend_dispatcher::interfaces::hsm::component_ethernet_interface::ComponentEthernetInterfaceTrait;
use manta_backend_dispatcher::types::hsm::inventory::ComponentEthernetInterface;
use serde_json::Value;

use super::*;

impl ComponentEthernetInterfaceTrait for StaticBackendDispatcher {
  /// `GET /Inventory/EthernetInterfaces` — every interface.
  async fn get_all_component_ethernet_interfaces(
    &self,
    auth_token: &str,
  ) -> Result<Vec<ComponentEthernetInterface>, Error> {
    dispatch!(self, get_all_component_ethernet_interfaces, auth_token)
  }

  /// `GET /Inventory/EthernetInterfaces/{id}` — one interface.
  async fn get_component_ethernet_interface(
    &self,
    auth_token: &str,
    eth_interface_id: &str,
  ) -> Result<ComponentEthernetInterface, Error> {
    dispatch!(
      self,
      get_component_ethernet_interface,
      auth_token,
      eth_interface_id
    )
  }

  /// `POST /Inventory/EthernetInterfaces` — create.
  async fn add_component_ethernet_interface(
    &self,
    auth_token: &str,
    component_ethernet_interface: &ComponentEthernetInterface,
  ) -> Result<(), Error> {
    dispatch!(
      self,
      add_component_ethernet_interface,
      auth_token,
      component_ethernet_interface
    )
  }

  /// `PATCH /Inventory/EthernetInterfaces/{id}` — update
  /// description and/or the (current_ip, new_ip) mapping. Returns
  /// HSM's raw JSON response.
  async fn update_component_ethernet_interface(
    &self,
    auth_token: &str,
    eth_interface_id: &str,
    description: Option<&str>,
    ip_address_mapping: (&str, &str),
  ) -> Result<Value, Error> {
    dispatch!(
      self,
      update_component_ethernet_interface,
      auth_token,
      eth_interface_id,
      description,
      ip_address_mapping
    )
  }

  /// `DELETE /Inventory/EthernetInterfaces` — wipe the collection.
  /// Returns HSM's action-summary JSON.
  async fn delete_all_component_ethernet_interfaces(
    &self,
    auth_token: &str,
  ) -> Result<Value, Error> {
    dispatch!(self, delete_all_component_ethernet_interfaces, auth_token)
  }

  /// `DELETE /Inventory/EthernetInterfaces/{id}`.
  async fn delete_component_ethernet_interface(
    &self,
    auth_token: &str,
    eth_interface_id: &str,
  ) -> Result<Value, Error> {
    dispatch!(
      self,
      delete_component_ethernet_interface,
      auth_token,
      eth_interface_id
    )
  }
}
