//! Shared library for `manta-cli` and (future) `manta-server`.
//!
//! Top-level modules:
//!
//! - [`shared`] — wire-shaped data (request `*Params`, response DTOs, SAT
//!   YAML parser, cluster-status helpers).
//! - [`common`] — config loader / TOML schema, audit + Kafka producer,
//!   JWT helpers, `InfraContext`, tracing setup, the network-reachability
//!   probe and shared authorization helpers.
//! - [`manta_backend_dispatcher`] + [`backend_dispatcher`] — the
//!   `StaticBackendDispatcher` enum (CSM / OCHAMI variants) and its trait
//!   impls.

pub mod shared;
pub mod common;

pub mod backend_dispatcher;
pub mod manta_backend_dispatcher;
