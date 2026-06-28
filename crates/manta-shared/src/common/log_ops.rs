//! Tracing-subscriber initialisation shared by both binaries.
//!
//! Both `manta-cli` and `manta-server` call [`configure`] exactly once
//! at startup, after the config file has supplied a `log` directive.
//! Centralising the setup here keeps target filtering, the `log →
//! tracing` bridge, and the timestamp toggle consistent across the two
//! binaries — useful when grepping logs from a CLI invocation that
//! triggered a server-side action.

use tracing_subscriber::EnvFilter;

/// Configure the global tracing subscriber and bridge `log::` calls into it.
///
/// `log_level` is an `EnvFilter` directive string, e.g. `"info"`, `"debug"`,
/// or `"manta=debug,hyper=warn"`. Falls back to `"error"` on parse failure.
///
/// `with_timestamps` controls whether each emitted line is prefixed with
/// the local time. The long-running server enables this so operators can
/// correlate events across requests; the interactive CLI disables it to
/// keep terminal output uncluttered.
///
/// Call this exactly once per process; subsequent calls are no-ops
/// (the subscriber is global and `init()` is idempotent-ish — it
/// panics on a second install, so guard with `OnceCell` if you need
/// to re-configure).
///
/// # Examples
///
/// Typical CLI startup — no timestamps, simple level:
///
/// ```no_run
/// use manta_shared::common::log_ops;
///
/// log_ops::configure("info", false);
/// tracing::info!("manta-cli starting");
/// ```
///
/// Server startup with per-target filtering:
///
/// ```no_run
/// use manta_shared::common::log_ops;
///
/// log_ops::configure("manta=debug,hyper=warn,tower_http=info", true);
/// ```
pub fn configure(log_level: &str, with_timestamps: bool) {
  let filter =
    EnvFilter::try_new(log_level).unwrap_or_else(|_| EnvFilter::new("error"));

  let builder = tracing_subscriber::fmt()
    .with_env_filter(filter)
    .with_target(false);

  if with_timestamps {
    builder.init();
  } else {
    builder.without_time().init();
  }
}
