//! Types shared between the CLI and the HTTP server.
//!
//! Everything in this module is "wire-shaped" data — request parameter
//! structs and re-exports of the response types returned by the manta
//! server. There is no business logic here. Both `crate::cli` and
//! `crate::server`/`crate::service` consume these types; nothing in
//! this module depends on either layer.
//!
//! Future-proofing: once the codebase is split into a Cargo workspace, this
//! module becomes the `manta-shared` library crate.

pub mod auth;
pub mod cluster_status;
pub mod dto;
pub mod params;
