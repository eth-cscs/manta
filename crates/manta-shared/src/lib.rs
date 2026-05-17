//! Shared library for `manta-cli` and (future) `manta-server`.
//!
//! Two top-level modules:
//!
//! - [`shared`] — wire-shaped data (request `*Params`, response DTOs, SAT
//!   YAML parser, cluster-status helpers).
//! - [`manta_backend_dispatcher`] + [`backend_dispatcher`] — the
//!   `StaticBackendDispatcher` enum (CSM / OCHAMI variants) and its trait
//!   impls. The CLI uses it for authentication; the server (once
//!   extracted) uses it for every upstream call.

pub mod shared;

pub mod backend_dispatcher;
pub mod manta_backend_dispatcher;
