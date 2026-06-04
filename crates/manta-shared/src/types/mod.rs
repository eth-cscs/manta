//! Wire types shared between the `manta-cli` and `manta-server`
//! binaries — the CLI↔server API contract.
//!
//! Everything in this module is "wire-shaped" data: request parameter
//! structs (`params/`), response DTOs (`dto`), auth wire shapes
//! (`auth`), plus pure helpers that operate on those types
//! (`cluster_status`). There is no business logic and no I/O; this
//! module depends only on `serde`, `utoipa`, and `manta-backend-dispatcher`
//! type re-exports. Anything that performs work — config loading,
//! tracing init, error conversion — lives in [`super::common`].

pub mod auth;
pub mod cluster_status;
pub mod dto;
pub mod params;
