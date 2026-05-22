//! Shared types for hardware-cluster operations.

/// Whether the hw cluster operation moves nodes into the target (Pin) or
/// releases them back (Unpin).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HwClusterMode {
  /// Move nodes matching the hardware pattern from the parent cluster
  /// into the target cluster.
  Pin,
  /// Move nodes matching the hardware pattern from the target cluster
  /// back to the parent cluster.
  Unpin,
}
