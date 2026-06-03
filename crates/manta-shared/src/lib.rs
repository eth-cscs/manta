//! Shared library used by both `manta-cli` and `manta-server`.
//!
//! Top-level modules:
//!
//! - [`shared`] — wire-shaped data (request `*Params`, response DTOs,
//!   cluster-status helpers). Genuinely used by both binaries.
//! - [`common`] — behavioural helpers. The `config` loader and
//!   `error` types are bi-binary by use; `log_ops` and `sat_file` are
//!   CLI-only (kept here pending a per-binary split). Server-only
//!   helpers (`audit`, `kafka`, `jwt_ops`) and the typed config
//!   schemas (`CliConfiguration`, `ServerConfiguration`) now live in
//!   their respective binary crates.
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
