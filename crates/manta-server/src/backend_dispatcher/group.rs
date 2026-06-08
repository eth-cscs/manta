//! `GroupTrait` impl for `StaticBackendDispatcher`.
//!
//! Method names come from the external `manta-backend-dispatcher`
//! crate's `GroupTrait`. Can't be renamed locally; cleaner names
//! would have to land upstream first.

use super::*;

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
    group_name: Group,
  ) -> Result<Group, Error> {
    dispatch!(self, add_group, auth_token, group_name)
  }

  async fn get_member_vec_from_group_name_vec(
    &self,
    auth_token: &str,
    group_name_vec: &[String],
  ) -> Result<Vec<String>, Error> {
    dispatch!(
      self,
      get_member_vec_from_group_name_vec,
      auth_token,
      group_name_vec
    )
  }

  async fn get_group_map_and_filter_by_group_vec(
    &self,
    auth_token: &str,
    group_name_vec: &[&str],
  ) -> Result<HashMap<String, Vec<String>>, Error> {
    dispatch!(
      self,
      get_group_map_and_filter_by_group_vec,
      auth_token,
      group_name_vec
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

  async fn get_group(
    &self,
    auth_token: &str,
    group_name: &str,
  ) -> Result<Group, Error> {
    dispatch!(self, get_group, auth_token, group_name)
  }

  async fn get_groups(
    &self,
    auth_token: &str,
    group_name_vec: Option<&[String]>,
  ) -> Result<Vec<Group>, Error> {
    dispatch!(self, get_groups, auth_token, group_name_vec)
  }

  async fn delete_group(
    &self,
    auth_token: &str,
    group_name: &str,
  ) -> Result<HsmActionResponse, Error> {
    dispatch!(self, delete_group, auth_token, group_name)
  }

  async fn get_group_map_and_filter_by_group_name_vec(
    &self,
    auth_token: &str,
    group_name_vec: &[&str],
  ) -> Result<HashMap<String, Vec<String>>, Error> {
    dispatch!(
      self,
      get_group_map_and_filter_by_group_name_vec,
      auth_token,
      group_name_vec
    )
  }

  async fn post_member(
    &self,
    auth_token: &str,
    group_name: &str,
    xname: &str,
  ) -> Result<HsmActionResponse, Error> {
    dispatch!(self, post_member, auth_token, group_name, xname)
  }

  async fn add_members_to_group(
    &self,
    auth_token: &str,
    group_name: &str,
    xnames: &[&str],
  ) -> Result<Vec<String>, Error> {
    dispatch!(self, add_members_to_group, auth_token, group_name, xnames)
  }

  async fn delete_member_from_group(
    &self,
    auth_token: &str,
    group_name: &str,
    xname: &str,
  ) -> Result<(), Error> {
    dispatch!(
      self,
      delete_member_from_group,
      auth_token,
      group_name,
      xname
    )
  }

  async fn migrate_group_members(
    &self,
    auth_token: &str,
    target_group_name: &str,
    parent_group_name: &str,
    new_target_group_members: &[&str],
    dryrun: bool,
  ) -> Result<(Vec<String>, Vec<String>), Error> {
    dispatch!(
      self,
      migrate_group_members,
      auth_token,
      target_group_name,
      parent_group_name,
      new_target_group_members,
      dryrun
    )
  }

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
