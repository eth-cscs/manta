//! Library root for the `manta-server` crate.
//!
//! All modules live here. The `main.rs` binary is a thin bootstrap
//! shim; integration tests in `crates/manta-server/tests/` import the
//! library directly via `use manta_server::...`.

// Warn (not deny) on undocumented pub items. The server isn't
// docs.rs-bound (publish = false), so the goal here is contributor
// onboarding rather than a public API contract. CI still surfaces
// these warnings via the rustdoc build step.
#![warn(missing_docs)]

pub mod backend_dispatcher;
pub mod config;
pub mod manta_backend_dispatcher;
pub mod server;
pub mod service;
pub mod wire_conv;
