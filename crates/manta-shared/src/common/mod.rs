//! Helpers shared by both binaries (manta-cli today; manta-server soon).

/// Date-time format string used for displaying timestamps
/// throughout the application (e.g. "04/03/2026 14:30:00").
pub const DATETIME_FORMAT: &str = "%d/%m/%Y %H:%M:%S";

pub mod app_context;
pub mod audit;
pub mod check_network_connectivity;
pub mod config;
pub mod jwt_ops;
pub mod kafka;
pub mod log_ops;
