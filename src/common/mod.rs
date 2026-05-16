//! Helpers still shared between CLI and server during the workspace split.
//!
//! Targeted for migration: `app_context` and `config` move out once each
//! binary gets its own context type (workspace Phase 4); `check_network_connectivity`
//! moves with `config` since it's only invoked by the config loader;
//! `log_ops` will be duplicated per binary (16 lines, not worth sharing).

/// Date-time format string used for displaying timestamps
/// throughout the application (e.g. "04/03/2026 14:30:00").
pub const DATETIME_FORMAT: &str = "%d/%m/%Y %H:%M:%S";

pub mod app_context;
pub mod audit;
pub mod check_network_connectivity;
pub mod config;
pub mod jwt_ops;
pub mod kafka;
pub mod log_ops;

// Compatibility re-exports for partitioned modules.
//
// Phase 3 of the workspace split moved layer-specific modules into
// `crate::cli::common::*` and `crate::server::common::*`. These re-exports
// let legacy `crate::common::X` import paths (especially the in-body
// `common::X::*` form that comes from `use crate::common;`) keep working
// until Phase 4 — the workspace move — rewrites each crate's import roots
// independently.
#[allow(unused_imports)]
pub use crate::cli::common::{
  authentication, hooks, kernel_parameters_ops, local_git_repo, user_interaction,
};
#[allow(unused_imports)]
pub use crate::server::common::{
  authorization, boot_parameters, hw_inventory_utils, ims_ops, node_ops, vault,
};
