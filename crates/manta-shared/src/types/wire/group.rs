//! Wire types for `POST /api/v1/groups/{name}/members` and
//! `DELETE /api/v1/groups/{name}/members`.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Request body for `POST /api/v1/groups/{name}/members`.
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
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct AddNodesToGroupResponse {
  /// Xnames that were added to the group as part of this request.
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
