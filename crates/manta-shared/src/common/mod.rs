//! Behavioural helpers shared by `manta-cli` and `manta-server`.
//!
//! The three submodules are intentionally narrow:
//!
//! - [`config`] — locates and parses `cli.toml` / `server.toml`,
//!   honouring `MANTA_CLI_CONFIG` / `MANTA_SERVER_CONFIG` and merging
//!   `MANTA_*`-prefixed environment overrides. Returns an untyped
//!   `::config::Config` so each binary owns its own typed schema.
//! - [`error`] — the [`error::MantaError`] enum returned by every
//!   fallible helper in this crate; the server bridges it to its
//!   `BackendError` at call sites.
//! - [`log_ops`] — single `configure(...)` entry point both binaries
//!   call once at startup to install the tracing subscriber.

/// Date-time format string used for displaying timestamps
/// throughout the application (e.g. "04/03/2026 14:30:00").
pub const DATETIME_FORMAT: &str = "%d/%m/%Y %H:%M:%S";

pub mod config;
pub mod error;
pub mod log_ops;
