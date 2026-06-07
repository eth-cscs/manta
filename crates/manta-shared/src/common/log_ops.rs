//! Tracing-subscriber initialisation shared by both binaries.

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
