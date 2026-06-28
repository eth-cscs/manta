//! Shared library used by both `manta-cli` and `manta-server`.
//!
//! Top-level modules:
//!
//! - [`types`] — wire-shaped data (request `*Params`, response DTOs,
//!   cluster-status helpers). The CLI↔server API contract — both
//!   binaries serialize and deserialize through these types.
//! - [`common`] — bi-binary behavioural helpers: the [`common::config`]
//!   loader (returns an untyped `::config::Config`),
//!   [`common::error::MantaError`], and
//!   [`common::log_ops::configure`]. Single-binary helpers (`audit`,
//!   `kafka`, `jwt_ops`, the SAT-file Jinja renderer) and the typed
//!   config schemas (`CliConfiguration`, `ServerConfiguration`,
//!   `Auditor`/`Kafka`) live with whichever binary uses them.
//!
//! The backend bridge (`StaticBackendDispatcher`, the CSM/OCHAMI trait
//! impls, and `authorization` helpers that take a `&StaticBackendDispatcher`)
//! lives in `manta-server`; the CLI never reaches it.
//!
//! # Quick start
//!
//! Initialise logging and load the CLI config; the `NotFound` error
//! variant carries a sample `cli.toml` and (if applicable) a
//! migration mapping from the legacy unified `config.toml`:
//!
//! ```no_run
//! use manta_shared::common::{config, error::MantaError, log_ops};
//!
//! fn main() -> Result<(), MantaError> {
//!   log_ops::configure("info", false);
//!
//!   let cfg = config::get_cli_configuration()?;
//!   let log_level = cfg.get_string("log").unwrap_or_else(|_| "info".into());
//!   tracing::info!("log level from cli.toml: {log_level}");
//!   Ok(())
//! }
//! ```
//!
//! The returned `::config::Config` is untyped on purpose — `manta-cli`
//! deserialises it into `CliConfiguration` and `manta-server` into
//! `ServerConfiguration`, so the same loader serves both binaries.

// Every public item in this crate must carry a `///` doc comment.
// We're a publishable crate (`publish = true`); the docs.rs page is
// the primary external interface and stale-or-missing docs there are
// user-facing. CI's `cargo doc` step keeps this honest.
#![deny(missing_docs)]

pub mod common;
pub mod types;
