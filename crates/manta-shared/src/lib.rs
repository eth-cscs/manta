//! Shared library used by both `manta-cli` and `manta-server`.
//!
//! Top-level modules:
//!
//! - [`shared`] — wire-shaped data (request `*Params`, response DTOs, SAT
//!   YAML parser, cluster-status helpers).
//! - [`common`] — config loader / TOML schema, audit + Kafka producer,
//!   JWT helpers, `AppContext`/`InfraContext`, and tracing setup.
//!
//! The backend bridge (`StaticBackendDispatcher`, the CSM/OCHAMI trait
//! impls, and `authorization` helpers that take a `&StaticBackendDispatcher`)
//! lives in `manta-server`; the CLI never reaches it.

// Every public item in this crate must carry a `///` doc comment.
// We're a publishable crate (`publish = true`); the docs.rs page is
// the primary external interface and stale-or-missing docs there are
// user-facing. CI's `cargo doc` step keeps this honest.
#![deny(missing_docs)]

pub mod common;
pub mod shared;
