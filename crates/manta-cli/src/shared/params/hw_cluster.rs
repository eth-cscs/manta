//! Shared types for hardware-cluster operations.

/// Whether the hw cluster operation moves nodes into the target (Pin) or
/// releases them back (Unpin).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HwClusterMode {
  Pin,
  Unpin,
}
