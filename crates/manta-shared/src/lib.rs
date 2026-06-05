//! Shared library used by both `manta-cli` and `manta-server`.
//!
//! Top-level modules:
//!
//! - [`types`] — wire-shaped data (request `*Params`, response DTOs,
//!   cluster-status helpers). The CLI↔server API contract — both
//!   binaries serialize and deserialize through these types.
//! - [`common`] — bi-binary behavioural helpers: the `config` loader
//!   (returns an untyped `::config::Config`), `MantaError`, and
//!   `log_ops::configure(...)`. Single-binary helpers (`audit`,
//!   `kafka`, `jwt_ops`, the SAT-file Jinja renderer) and the typed
//!   config schemas (`CliConfiguration`, `ServerConfiguration`,
//!   `Auditor`/`Kafka`) live with whichever binary uses them.
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
/// Provenance metadata written to an IMS image after a successful CFS
/// session (`base`, `groups`, `configuration`).
pub mod image_session;
pub mod types;
