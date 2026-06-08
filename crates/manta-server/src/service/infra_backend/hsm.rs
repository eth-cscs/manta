//! HSM (group + component + hardware inventory) backend methods on
//! `InfraContext`.

use std::collections::HashMap;

use manta_backend_dispatcher::error::Error;
use manta_backend_dispatcher::interfaces::hsm::component::ComponentTrait;
use manta_backend_dispatcher::interfaces::hsm::group::GroupTrait;
use manta_backend_dispatcher::interfaces::hsm::hardware_inventory::HardwareInventory;
use manta_backend_dispatcher::types::{
  Component as HsmComponent, ComponentArrayPostArray, Group,
  HWInventoryByLocationList, HsmActionResponse,
};

use crate::server::common::app_context::InfraContext;

impl InfraContext<'_> {
  /// List the HSM groups the caller's token can access.
  pub async fn get_group_available(
    &self,
    token: &str,
  ) -> Result<Vec<Group>, Error> {
    self.backend.get_group_available(token).await
  }

  /// List the groups the caller's token can access (names only).
  pub async fn get_group_name_available(
    &self,
    token: &str,
  ) -> Result<Vec<String>, Error> {
    self.backend.get_group_name_available(token).await
  }

  /// Resolve a list of HSM group names to their member xnames.
  pub async fn get_member_vec_from_group_name_vec(
    &self,
    token: &str,
    group_names: &[String],
  ) -> Result<Vec<String>, Error> {
    self
      .backend
      .get_member_vec_from_group_name_vec(token, group_names)
      .await
  }

  /// Delete a node by xname.
  pub async fn delete_node(&self, token: &str, id: &str) -> Result<(), Error> {
    self.backend.delete_node(token, id).await.map(|_| ())
  }

  /// Register one or more nodes with HSM.
  pub async fn post_nodes(
    &self,
    token: &str,
    components: ComponentArrayPostArray,
  ) -> Result<(), Error> {
    self.backend.post_nodes(token, components).await
  }

  /// Upload hardware inventory records.
  pub async fn post_inventory_hardware(
    &self,
    token: &str,
    hw_inventory: HWInventoryByLocationList,
  ) -> Result<(), Error> {
    self
      .backend
      .post_inventory_hardware(token, hw_inventory)
      .await
      .map(|_| ())
  }

  /// Add a node to an HSM group.
  pub async fn post_member(
    &self,
    token: &str,
    group: &str,
    id: &str,
  ) -> Result<(), Error> {
    self.backend.post_member(token, group, id).await.map(|_| ())
  }

  /// Fetch a single HSM group by name.
  pub async fn get_group(
    &self,
    token: &str,
    name: &str,
  ) -> Result<Group, Error> {
    self.backend.get_group(token, name).await
  }

  /// Move nodes from `parent` HSM group into `target`.
  pub async fn migrate_group_members(
    &self,
    token: &str,
    target_hsm_name: &str,
    parent_hsm_name: &str,
    xnames: &[&str],
    dry_run: bool,
  ) -> Result<(Vec<String>, Vec<String>), Error> {
    self
      .backend
      .migrate_group_members(
        token,
        target_hsm_name,
        parent_hsm_name,
        xnames,
        dry_run,
      )
      .await
  }

  /// Fetch metadata for every HSM node the caller can access.
  pub async fn get_node_metadata_available(
    &self,
    token: &str,
  ) -> Result<Vec<HsmComponent>, Error> {
    self.backend.get_node_metadata_available(token).await
  }

  /// List HSM groups, optionally restricted to a name set.
  pub async fn get_groups(
    &self,
    token: &str,
    hsm_name_vec: Option<&[String]>,
  ) -> Result<Vec<Group>, Error> {
    self.backend.get_groups(token, hsm_name_vec).await
  }

  /// For each xname, return the HSM groups it belongs to.
  pub async fn get_group_map_and_filter_by_group_vec(
    &self,
    token: &str,
    hsm_name_vec: &[&str],
  ) -> Result<HashMap<String, Vec<String>>, Error> {
    self
      .backend
      .get_group_map_and_filter_by_group_vec(token, hsm_name_vec)
      .await
  }

  /// Delete an HSM group by label.
  pub async fn delete_group(
    &self,
    token: &str,
    label: &str,
  ) -> Result<HsmActionResponse, Error> {
    self.backend.delete_group(token, label).await
  }

  /// Create an HSM group.
  pub async fn add_group(
    &self,
    token: &str,
    group: Group,
  ) -> Result<Group, Error> {
    self.backend.add_group(token, group).await
  }

  /// Remove a single xname from an HSM group.
  pub async fn delete_member_from_group(
    &self,
    token: &str,
    group_label: &str,
    xname: &str,
  ) -> Result<(), Error> {
    self
      .backend
      .delete_member_from_group(token, group_label, xname)
      .await
  }

  /// Add xnames to an HSM group; returns the updated member list.
  pub async fn add_members_to_group(
    &self,
    token: &str,
    group_label: &str,
    members: &[&str],
  ) -> Result<Vec<String>, Error> {
    self
      .backend
      .add_members_to_group(token, group_label, members)
      .await
  }

  /// Replace an HSM group's membership: remove `members_to_remove`, add `members_to_add`.
  pub async fn update_group_members(
    &self,
    token: &str,
    group_name: &str,
    members_to_remove: &[&str],
    members_to_add: &[&str],
  ) -> Result<(), Error> {
    self
      .backend
      .update_group_members(
        token,
        group_name,
        members_to_remove,
        members_to_add,
      )
      .await
  }
}
