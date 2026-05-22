//! Shared library used by both `manta-cli` and `manta-server`.
//!
//! Top-level modules:
//!
//! - [`shared`] — wire-shaped data (request `*Params`, response DTOs, SAT
//!   YAML parser, cluster-status helpers).
//! - [`common`] — config loader / TOML schema, audit + Kafka producer,
//!   JWT helpers, `AppContext`/`InfraContext`, tracing setup, and the
//!   network-reachability probe.
//!
//! The backend bridge (`StaticBackendDispatcher`, the CSM/OCHAMI trait
//! impls, and `authorization` helpers that take a `&StaticBackendDispatcher`)
//! lives in `manta-server`; the CLI never reaches it.

// Track missing rustdoc coverage at WARN level for now (the workspace
// docs are mid-ramp-up). Flip to `deny` once the per-file passes
// land and CI's `cargo doc` step is wired in.
#![warn(missing_docs)]

pub mod common;
pub mod shared;
