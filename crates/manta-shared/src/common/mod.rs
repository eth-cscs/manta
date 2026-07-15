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

/// Parse an IMS `created` timestamp into a comparable value.
///
/// CSM returns `created` in more than one shape, so try
/// [`chrono::NaiveDateTime`] first, then [`chrono::DateTime<Local>`],
/// normalising a zoned timestamp to local naive time. Returns `None`
/// when neither parses.
///
/// Lives here rather than in either binary because the server filters
/// on this value (`service::image::get_images`) while the CLI renders
/// it (`output::image`). If the two disagreed on what parses, a row
/// could display a creation date that the filter silently drops.
///
/// ```
/// use manta_shared::common::parse_ims_timestamp;
///
/// assert!(parse_ims_timestamp("2026-06-04T12:30:00").is_some());
/// assert!(parse_ims_timestamp("2026-06-04T12:30:00+00:00").is_some());
/// assert!(parse_ims_timestamp("not-a-real-date").is_none());
/// ```
#[must_use]
pub fn parse_ims_timestamp(raw: &str) -> Option<chrono::NaiveDateTime> {
  if let Ok(v) = raw.parse::<chrono::NaiveDateTime>() {
    return Some(v);
  }
  if let Ok(v) = raw.parse::<chrono::DateTime<chrono::Local>>() {
    return Some(v.naive_local());
  }
  None
}

pub mod config;
pub mod error;
pub mod jwt_ops;
pub mod log_ops;
