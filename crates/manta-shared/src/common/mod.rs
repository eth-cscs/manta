//! Behavioural helpers used by both binaries: config loader, error
//! type, log-init helper. `log_ops` is bi-binary; both `manta-cli` and
//! `manta-server` `configure(...)` the tracing subscriber on startup.

/// Date-time format string used for displaying timestamps
/// throughout the application (e.g. "04/03/2026 14:30:00").
pub const DATETIME_FORMAT: &str = "%d/%m/%Y %H:%M:%S";

pub mod config;
pub mod error;
pub mod log_ops;
