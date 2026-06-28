//! HTTP request/response bodies and CLI-built parameter structs for
//! the power endpoints (`/api/v1/power`).

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Request body for `POST /api/v1/power`.
///
/// The interpretation of `host_expression` depends on [`PowerTargetType`];
/// the action itself is one of [`PowerAction`]'s variants.
///
/// # Wire shape
///
/// ```json
/// {
///   "action": "on",
///   "host_expression": "x3000c0s1b0n[0-3]",
///   "target_type": "nodes",
///   "force": false
/// }
/// ```
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PowerRequest {
  /// Power operation to perform.
  pub action: PowerAction,
  /// For `Nodes`: hosts expression (xnames, NIDs, or hostlist
  /// notation). For `Cluster`: the HSM group name.
  pub host_expression: String,
  /// Whether `host_expression` is a node expression or a cluster
  /// name.
  pub target_type: PowerTargetType,
  /// Pass `--force` to the underlying power operation (forceful
  /// shutdown/reset).
  #[serde(default)]
  pub force: bool,
}

/// The power operation to apply to a list of xnames.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum PowerAction {
  /// Power on (cold start) the listed xnames.
  On,
  /// Power off the listed xnames; graceful unless `force` is set.
  Off,
  /// Power-cycle (reset) the listed xnames; graceful unless `force`
  /// is set.
  Reset,
}

/// Whether the caller's `host_expression` is a hosts expression
/// (xnames / NIDs / hostlist) or a single HSM group name whose
/// members should be targeted.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum PowerTargetType {
  /// `host_expression` is a hosts expression.
  Nodes,
  /// `host_expression` is a single HSM group name.
  Cluster,
}

/// Typed parameters for the power-action service call.
pub struct ApplyPowerParams {
  /// Power operation to perform on every entry in `xnames`.
  pub action: PowerAction,
  /// Resolved list of xnames (already expanded from any HSM-group
  /// or hostlist expression by the caller).
  pub xnames: Vec<String>,
  /// When true, perform a hard power off / reset without the
  /// graceful shutdown path.
  pub force: bool,
}
