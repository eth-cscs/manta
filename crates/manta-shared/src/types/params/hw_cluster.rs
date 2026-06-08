//! Shared types for hardware-cluster operations.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Whether the hw cluster operation moves nodes into the target (Pin) or
/// releases them back (Unpin).
#[derive(
  Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, ToSchema,
)]
#[serde(rename_all = "lowercase")]
pub enum HwClusterMode {
  /// Move nodes matching the hardware pattern from the parent cluster
  /// into the target cluster.
  #[default]
  Pin,
  /// Move nodes matching the hardware pattern from the target cluster
  /// back to the parent cluster.
  Unpin,
}
