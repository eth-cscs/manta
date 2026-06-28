//! HTTP request/response bodies and CLI-built parameter structs for
//! the HSM group endpoints (`/api/v1/groups`,
//! `/api/v1/groups/{name}/members`).

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Request body for `POST /api/v1/groups/{name}/members`.
///
/// Paired with [`AddNodesToGroupResponse`] on success.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct AddNodesToGroupRequest {
  /// Hostlist expression (xnames, NIDs, or hostlist notation)
  /// identifying the new member set for the group.
  pub hosts_expression: String,
}

/// Response body for `POST /api/v1/groups/{name}/members`.
///
/// The `removed` field name is retained for wire stability; its value
/// is the final, sorted membership of the group after the update —
/// **not** a list of removed nodes.
///
/// `added` and `final_members` are both sorted alphabetically by xname.
///
/// # Wire shape
///
/// ```json
/// {
///   "added": ["x3000c0s1b0n2", "x3000c0s1b0n3"],
///   "final_members": ["x3000c0s1b0n0", "x3000c0s1b0n1", "x3000c0s1b0n2", "x3000c0s1b0n3"],
///   "removed": ["x3000c0s1b0n0", "x3000c0s1b0n1", "x3000c0s1b0n2", "x3000c0s1b0n3"]
/// }
/// ```
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct AddNodesToGroupResponse {
  /// Xnames that were added to the group as part of this request,
  /// sorted alphabetically.
  pub added: Vec<String>,
  /// Final, sorted membership of the group after the update.
  pub final_members: Vec<String>,
  /// Deprecated alias for [`Self::final_members`]. Carries the same
  /// value so existing clients reading `removed` keep working for one
  /// release; new clients should read `final_members`. Scheduled for
  /// removal in the next major bump — at which point the
  /// `#[serde(alias = "removed")]` on `final_members` keeps inbound
  /// compatibility for anyone POSTing back the old name.
  #[serde(default)]
  pub removed: Vec<String>,
}

/// Request body for `DELETE /api/v1/groups/{name}/members`.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct DeleteGroupMembersRequest {
  /// Hosts expression (xnames, NIDs, or hostlist notation)
  /// identifying nodes to remove.
  pub xnames_expression: String,
  /// When true, validate the request without modifying group
  /// membership.
  #[serde(default)]
  pub dry_run: bool,
}

/// Typed parameters for fetching HSM groups.
#[derive(Debug)]
pub struct GetGroupParams {
  /// Exact group name to fetch; returns all groups when `None`.
  pub group_name: Option<String>,
  /// Operator default from `~/.config/manta/cli.toml`'s
  /// `group_name`; used to scope results when `group_name` is
  /// absent but a configured default exists.
  pub settings_group_name: Option<String>,
}

impl GetGroupParams {
  /// Returns the effective group name, preferring the explicit
  /// `--group` flag and falling back to the operator default from
  /// `cli.toml`.
  pub fn effective_group(&self) -> Option<&str> {
    self
      .group_name
      .as_deref()
      .or(self.settings_group_name.as_deref())
  }
}
