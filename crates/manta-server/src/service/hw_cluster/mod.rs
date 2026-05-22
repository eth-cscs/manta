//! Hardware cluster pin/unpin and hw-component add/delete service logic.
//!
//! Split into three (private) sub-modules:
//!
//! - `scoring` — pure-computation functions for component scarcity,
//!   per-node scoring, candidate selection, pattern parsing, and the
//!   parallel hw-inventory fetcher. Also hosts
//!   `resolve_hw_description_to_xnames`, which dispatches between
//!   pin and unpin.
//! - `pin_unpin` — the `calculate_target_hsm_pin` / `_unpin` node
//!   selection algorithms plus the shared coordination helpers used
//!   by `apply_hw_configuration` (pattern parsing, target-group
//!   existence check, resource-sufficiency validation, group-update
//!   orchestration).
//! - `apply` — high-level coordinators called by the server
//!   handlers: `apply_hw_configuration`, `add_hw_component`,
//!   `delete_hw_component`.
//!
//! Public types (`AddHwResult`, `DeleteHwResult`, `ApplyHwResult`,
//! `NodeHwCountVec`, `HwClusterMode`) and shared constants live here
//! so all three sub-modules can use them.

use std::collections::HashMap;

mod apply;
mod pin_unpin;
mod scoring;

/// LCM (Least Common Multiple) used to normalise memory capacity values.
/// Memory DIMMs come in multiples of 16 GiB (16384 MiB).
pub(in crate::service::hw_cluster) const MEMORY_CAPACITY_LCM: u64 = 16384;

/// Maximum number of concurrent hardware component queries.
pub(in crate::service::hw_cluster) const HW_COMPONENT_CONCURRENCY_LIMIT: usize =
  5;

// ── Public types ────────────────────────────────────────────────────────────

pub use manta_shared::shared::params::hw_cluster::HwClusterMode;

/// A list of nodes paired with their per-component counts.
pub type NodeHwCountVec = Vec<(String, HashMap<String, usize>)>;

/// Result of an `add hw-component` operation.
pub struct AddHwResult {
  pub nodes_moved: Vec<String>,
  pub target_nodes: Vec<String>,
  pub parent_nodes: Vec<String>,
}

/// Result of a `delete hw-component` operation.
pub struct DeleteHwResult {
  pub nodes_moved: Vec<String>,
  pub target_nodes: Vec<String>,
  pub parent_nodes: Vec<String>,
}

/// Result of an `apply hw-configuration` (pin/unpin) operation.
pub struct ApplyHwResult {
  pub target_nodes: Vec<String>,
  pub parent_nodes: Vec<String>,
}

// ── External API (re-exported from sub-modules) ─────────────────────────────

pub use apply::{
  add_hw_component, apply_hw_configuration, delete_hw_component,
};

#[cfg(test)]
mod tests;
