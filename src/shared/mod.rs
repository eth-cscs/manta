//! Types shared between the CLI and the HTTP server.
//!
//! Everything in this module is "wire-shaped" data — request parameter
//! structs, deserialization helpers for SAT YAML files, and re-exports of the
//! response types returned by the manta server. There is no business logic
//! here. Both `crate::cli` and `crate::server`/`crate::service` consume these
//! types; nothing in this module depends on either layer.
//!
//! Future-proofing: once the codebase is split into a Cargo workspace, this
//! module becomes the `manta-shared` library crate.

pub mod dto;
pub mod params;
pub mod sat_file;
