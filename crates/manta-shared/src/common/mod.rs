//! Behavioural helpers. Bi-binary by use: `config`, `error`.
//! CLI-only by use: `log_ops`, `sat_file`.
//! Server-only by use: `audit`, `kafka`, `jwt_ops`. The single-binary
//! entries live here pending a per-binary split.

/// Date-time format string used for displaying timestamps
/// throughout the application (e.g. "04/03/2026 14:30:00").
pub const DATETIME_FORMAT: &str = "%d/%m/%Y %H:%M:%S";

pub mod audit;
pub mod config;
pub mod error;
pub mod jwt_ops;
pub mod kafka;
pub mod log_ops;
pub mod sat_file;
