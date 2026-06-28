//! Hardware-cluster pin/unpin and hw-component add/delete service logic.
//!
//! # Model
//!
//! CSM groups nodes into HSM groups. A *parent* group holds the
//! shared pool; a *target* group represents a sub-pool reserved for a
//! particular workload. "Pinning" moves nodes from the parent into
//! the target so that the target satisfies a user-supplied hardware
//! pattern (e.g. `a100:8,milan:2` — eight A100s and two Milan CPUs).
//! "Unpinning" is the reverse: nodes are released back to the parent.
//!
//! Both operations are framed as a search over candidate moves where
//! each candidate is scored by how *useful* a node is for the target
//! workload, with a scarcity weighting that prefers giving up common
//! components and keeping rare ones. See `scoring` for the rubric.
//!
//! # Layout
//!
//! Split into four (private) sub-modules plus shared types:
//!
//! - `scoring` — pure-computation functions for component scarcity,
//!   per-node scoring, candidate selection, pattern parsing, and the
//!   parallel hw-inventory fetcher. Also hosts
//!   `resolve_hw_description_to_xnames`, which dispatches between
//!   pin and unpin.
//! - `pin_unpin` — the `calculate_target_group_pin` / `_unpin` node
//!   selection algorithms plus the shared coordination helpers used
//!   by `apply_hw_configuration` (pattern parsing, target-group
//!   existence check, resource-sufficiency validation, group-update
//!   orchestration).
//! - `apply` — high-level coordinators called by the server
//!   handlers: `apply_hw_configuration`, `add_hw_component`,
//!   `delete_hw_component`.
//! - `hw_inventory_utils` — JSON-pointer helpers that extract memory,
//!   processor, and accelerator data from raw HSM inventory payloads.
//!
//! Public types (`AddHwResult`, `DeleteHwResult`, `ApplyHwResult`,
//! `NodeHwCountVec`, `HwClusterMode`) and shared constants live here
//! so all sub-modules can use them. The public surface is re-exported
//! at the bottom of this file; callers (the `hw_cluster` handlers
//! under `crate::server::handlers`) should depend only on those
//! re-exports.

use std::collections::HashMap;

mod apply;
mod hw_inventory_utils;
mod pin_unpin;
mod scoring;

/// LCM (Least Common Multiple) used to normalise memory capacity values.
/// Memory DIMMs come in multiples of 16 GiB (16384 MiB).
pub(in crate::service::hw_cluster) const MEMORY_CAPACITY_LCM: u64 = 16384;

/// Maximum number of concurrent hardware component queries.
pub(in crate::service::hw_cluster) const HW_COMPONENT_CONCURRENCY_LIMIT: usize =
  5;

// ── Public types ────────────────────────────────────────────────────────────

pub use manta_shared::types::api::hw_cluster::HwClusterMode;

/// A list of nodes paired with their per-component counts.
pub type NodeHwCountVec = Vec<(String, HashMap<String, usize>)>;

/// Result of an `add hw-component` operation.
pub struct AddHwResult {
  /// Xnames moved from the parent group into the target group as
  /// part of this operation.
  pub nodes_moved: Vec<String>,
  /// Final membership of the target group after the move.
  pub target_nodes: Vec<String>,
  /// Final membership of the parent group after the move.
  pub parent_nodes: Vec<String>,
}

/// Result of a `delete hw-component` operation.
pub struct DeleteHwResult {
  /// Xnames moved out of the target group and returned to the
  /// parent group as part of this operation.
  pub nodes_moved: Vec<String>,
  /// Final membership of the target group after the move.
  pub target_nodes: Vec<String>,
  /// Final membership of the parent group after the move.
  pub parent_nodes: Vec<String>,
}

/// Result of an `apply hw-configuration` (pin/unpin) operation.
pub struct ApplyHwResult {
  /// Final membership of the target group after pin/unpin completes.
  pub target_nodes: Vec<String>,
  /// Final membership of the parent group after pin/unpin completes.
  pub parent_nodes: Vec<String>,
}

// ── External API (re-exported from sub-modules) ─────────────────────────────

pub use apply::{
  ApplyHwConfigurationParams, add_hw_component, apply_hw_configuration,
  delete_hw_component,
};

#[cfg(test)]
mod tests;
