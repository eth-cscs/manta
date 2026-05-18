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

pub mod common;
pub mod shared;
