//! [`GroupTrait`] impl for [`StaticBackendDispatcher`].
//!
//! Forwards to HSM's `/apis/smd/hsm/v2/groups` endpoints plus the
//! per-group `/members` subresource. Both CSM and Ochami implement
//! this trait natively.
//!
//! Method names come from the external `manta-backend-dispatcher`
//! crate's `GroupTrait`. Can't be renamed locally; cleaner names
//! would have to land upstream first.

use super::*;

impl GroupTrait for StaticBackendDispatcher {
  /// RBAC-filtered group listing: the subset of HSM groups the
  /// caller's JWT roles grant access to. Returns full `Group`
  /// objects.
  async fn get_group_available(
    &self,
    auth_token: &str,
  ) -> Result<Vec<Group>, Error> {
    dispatch!(self, get_group_available, auth_token)
  }

  /// RBAC-filtered group listing, names only — faster path used when
  /// the caller only needs the group label set (e.g. to validate a
  /// filter argument).
  async fn get_group_name_available(
    &self,
    jwt_token: &str,
  ) -> Result<Vec<String>, Error> {
    dispatch!(self, get_group_name_available, jwt_token)
  }

  /// `POST /groups` — create a group. Returns the persisted group
  /// echoed back by HSM.
  async fn add_group(
    &self,
    auth_token: &str,
    group_name: Group,
  ) -> Result<Group, Error> {
    dispatch!(self, add_group, auth_token, group_name)
  }

  /// Union of `/groups/{name}/members` across `group_name_vec`,
  /// deduplicated.
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

  /// Map of `group_name -> [member xnames]` restricted to the groups
  /// named in `group_name_vec`. One HSM call per group plus a
  /// merge.
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

  /// Map of `group_name -> [member xnames]` restricted to groups
  /// that contain any xname in `member_vec`. The dual of
  /// `get_group_map_and_filter_by_group_vec`.
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

  /// `GET /groups/{name}` — one group by exact label.
  async fn get_group(
    &self,
    auth_token: &str,
    group_name: &str,
  ) -> Result<Group, Error> {
    dispatch!(self, get_group, auth_token, group_name)
  }

  /// `GET /groups` — every group when `group_name_vec` is `None`,
  /// otherwise the named subset.
  async fn get_groups(
    &self,
    auth_token: &str,
    group_name_vec: Option<&[String]>,
  ) -> Result<Vec<Group>, Error> {
    dispatch!(self, get_groups, auth_token, group_name_vec)
  }

  /// `DELETE /groups/{name}`. Returns HSM's action-summary
  /// (records affected).
  async fn delete_group(
    &self,
    auth_token: &str,
    group_name: &str,
  ) -> Result<HsmActionResponse, Error> {
    dispatch!(self, delete_group, auth_token, group_name)
  }

  /// Alias for `get_group_map_and_filter_by_group_vec` kept for
  /// upstream-trait shape compatibility.
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

  /// `POST /groups/{name}/members` — add one xname.
  async fn post_member(
    &self,
    auth_token: &str,
    group_name: &str,
    xname: &str,
  ) -> Result<HsmActionResponse, Error> {
    dispatch!(self, post_member, auth_token, group_name, xname)
  }

  /// Batched [`post_member`](Self::post_member). Returns the xnames
  /// the backend reported as successfully added (some entries may
  /// be skipped if already members).
  async fn add_members_to_group(
    &self,
    auth_token: &str,
    group_name: &str,
    xnames: &[&str],
  ) -> Result<Vec<String>, Error> {
    dispatch!(self, add_members_to_group, auth_token, group_name, xnames)
  }

  /// `DELETE /groups/{name}/members/{xname}`.
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

  /// Move `new_target_group_members` from `parent_group_name` into
  /// `target_group_name`. Returns `(added, removed)` xname lists.
  /// When `dryrun` the backend computes the diff without mutating
  /// either group.
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

  /// In-place edit of a single group's membership — applies
  /// `members_to_remove` then `members_to_add` to `group_name` as
  /// two HSM calls. Used by the `manta group set-members` path
  /// where the caller has pre-computed the diff.
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
