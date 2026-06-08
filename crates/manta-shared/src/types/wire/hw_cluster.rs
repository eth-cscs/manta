//! Wire types for the `POST /api/v1/hardware-clusters/{target}/*`
//! and `DELETE /api/v1/hardware-clusters/{target}/members` endpoints.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

pub use crate::types::params::hw_cluster::HwClusterMode;

/// Request body for `POST /api/v1/hardware-clusters/{target}/members`.
///
/// Moves nodes matching `pattern` out of `parent_cluster` and into
/// the path-level target cluster.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct AddHwComponentRequest {
  /// Source HSM group that donates nodes matching `pattern`.
  pub parent_cluster: String,
  /// Hardware component pattern used to select which nodes to move.
  pub pattern: String,
  /// Create the target HSM group if it does not already exist.
  #[serde(default)]
  pub create_hsm_group: bool,
  /// When true, return the planned changes without modifying group
  /// membership.
  #[serde(default)]
  pub dry_run: bool,
}

/// Request body for `DELETE /api/v1/hardware-clusters/{target}/members`.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct DeleteHwComponentRequest {
  /// Destination HSM group that receives nodes moved out of the
  /// target cluster.
  pub parent_cluster: String,
  /// Hardware component pattern used to select which nodes to move
  /// back.
  pub pattern: String,
  /// Delete the target HSM group if it becomes empty after the
  /// operation.
  #[serde(default)]
  pub delete_hsm_group: bool,
  /// When true, return the planned changes without modifying group
  /// membership.
  #[serde(default)]
  pub dry_run: bool,
}

/// Request body for
/// `POST /api/v1/hardware-clusters/{target}/configuration`.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ApplyHwConfigurationRequest {
  /// Source (parent) HSM group supplying nodes.
  pub parent_cluster: String,
  /// Hardware component pattern selecting which nodes to pin/unpin.
  pub pattern: String,
  /// Whether to pin nodes into the target cluster or unpin them back
  /// to the parent. Defaults to `Pin`.
  #[serde(default)]
  pub mode: HwClusterMode,
  /// Create the target HSM group if absent (default `true`).
  #[serde(default = "default_true")]
  pub create_target_hsm_group: bool,
  /// Delete the parent HSM group if it becomes empty (default `true`).
  #[serde(default = "default_true")]
  pub delete_empty_parent_hsm_group: bool,
  /// When true, return the planned changes without modifying group
  /// membership.
  #[serde(default)]
  pub dry_run: bool,
}

fn default_true() -> bool {
  true
}
