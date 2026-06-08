//! Wire types for `POST /api/v1/power` (start a PCS power transition).

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

pub use crate::types::params::power::{PowerAction, PowerTargetType};

/// Request body for `POST /api/v1/power`.
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
