//! Shared library for `manta-cli` and (future) `manta-server`.
//!
//! Wire-shaped data only — request `*Params`, response DTOs, SAT YAML
//! parser, cluster-status helpers. No business logic, no HTTP client, no
//! Axum types. Both binaries consume everything here through
//! `manta_shared::shared::*`.

pub mod shared;
