//! Library root for the `manta-server` crate.
//!
//! All modules live here. The `main.rs` binary is a thin bootstrap
//! shim; integration tests in `crates/manta-server/tests/` import the
//! library directly via `use manta_server::...`.

pub mod backend_dispatcher;
pub mod common;
pub mod manta_backend_dispatcher;
pub mod server;
pub mod service;
pub mod wire_conv;
