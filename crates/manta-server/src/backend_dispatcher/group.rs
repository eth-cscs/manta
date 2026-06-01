//! Dispatches `GroupTrait` methods to csm-rs or ochami-rs.

use std::collections::HashMap;

use manta_backend_dispatcher::{
  error::Error,
  interfaces::hsm::group::GroupTrait,
  types::{Group, HsmActionResponse},
};

use StaticBackendDispatcher::*;

use crate::manta_backend_dispatcher::StaticBackendDispatcher;

impl GroupTrait for StaticBackendDispatcher {
  async fn get_group_available(
    &self,
    auth_token: &str,
  ) -> Result<Vec<Group>, Error> {
    dispatch!(self, get_group_available, auth_token)
  }

  async fn get_group_name_available(
    &self,
    jwt_token: &str,
  ) -> Result<Vec<String>, Error> {
    dispatch!(self, get_group_name_available, jwt_token)
  }

  async fn add_group(
    &self,
    auth_token: &str,
    hsm_group: Group,
  ) -> Result<Group, Error> {
    dispatch!(self, add_group, auth_token, hsm_group)
  }

  // Function name (and the `get_member_vec_from_group_name_vec`
  // trait method it dispatches to) come from the external
  // `manta-backend-dispatcher` crate's `GroupTrait`. Can't be
  // renamed locally; a cleaner name like `get_hsm_group_members`
  // would have to land upstream first.
  async fn get_member_vec_from_group_name_vec(
    &self,
    auth_token: &str,
    hsm_group_name_vec: &[String],
  ) -> Result<Vec<String>, Error> {
    dispatch!(
      self,
      get_member_vec_from_group_name_vec,
      auth_token,
      hsm_group_name_vec
    )
  }

  async fn get_group_map_and_filter_by_group_vec(
    &self,
    auth_token: &str,
    hsm_name_vec: &[&str],
  ) -> Result<HashMap<String, Vec<String>>, Error> {
    dispatch!(
      self,
      get_group_map_and_filter_by_group_vec,
      auth_token,
      hsm_name_vec
    )
  }

  async fn get_group_map_and_filter_by_member_vec(
    &self,
    auth_token: &str,
    member_vec: &[&str],
  ) -> Result<HashMap<String, Vec<String>>, Error> {
    dispatch!(
      self,
      get_group_map_and_filter_by_member_vec,
      auth_token,
      member_vec
    )
  }

  async fn get_all_groups(
    &self,
    auth_token: &str,
  ) -> Result<Vec<Group>, Error> {
    dispatch!(self, get_all_groups, auth_token)
  }

  async fn get_group(
    &self,
    auth_token: &str,
    hsm_name: &str,
  ) -> Result<Group, Error> {
    dispatch!(self, get_group, auth_token, hsm_name)
  }

  async fn get_groups(
    &self,
    auth_token: &str,
    hsm_name_vec: Option<&[String]>,
  ) -> Result<Vec<Group>, Error> {
    dispatch!(self, get_groups, auth_token, hsm_name_vec)
  }

  async fn delete_group(
    &self,
    auth_token: &str,
    hsm_group_label: &str,
  ) -> Result<HsmActionResponse, Error> {
    dispatch!(self, delete_group, auth_token, hsm_group_label)
  }

  async fn get_hsm_map_and_filter_by_hsm_name_vec(
    &self,
    auth_token: &str,
    hsm_name_vec: &[&str],
  ) -> Result<HashMap<String, Vec<String>>, Error> {
    dispatch!(
      self,
      get_hsm_map_and_filter_by_hsm_name_vec,
      auth_token,
      hsm_name_vec
    )
  }

  async fn post_member(
    &self,
    auth_token: &str,
    group_label: &str,
    xname: &str,
  ) -> Result<HsmActionResponse, Error> {
    dispatch!(self, post_member, auth_token, group_label, xname)
  }

  // Add members to group.
  // Returns the final list of members in the group.
  async fn add_members_to_group(
    &self,
    auth_token: &str,
    group_label: &str,
    xnames: &[&str],
  ) -> Result<Vec<String>, Error> {
    dispatch!(self, add_members_to_group, auth_token, group_label, xnames)
  }

  async fn delete_member_from_group(
    &self,
    auth_token: &str,
    group_label: &str,
    xname: &str,
  ) -> Result<(), Error> {
    dispatch!(
      self,
      delete_member_from_group,
      auth_token,
      group_label,
      xname
    )
  }

  // HSM/GROUP
  async fn migrate_group_members(
    &self,
    auth_token: &str,
    target_hsm_group_name: &str,
    parent_hsm_group_name: &str,
    new_target_hsm_members: &[&str],
    dryrun: bool,
  ) -> Result<(Vec<String>, Vec<String>), Error> {
    dispatch!(
      self,
      migrate_group_members,
      auth_token,
      target_hsm_group_name,
      parent_hsm_group_name,
      new_target_hsm_members,
      dryrun
    )
  }

  // HSM/GROUP
  async fn update_group_members(
    &self,
    auth_token: &str,
    group_name: &str,
    members_to_remove: &[&str],
    members_to_add: &[&str],
  ) -> Result<(), Error> {
    dispatch!(
      self,
      update_group_members,
      auth_token,
      group_name,
      members_to_remove,
      members_to_add
    )
  }
}
