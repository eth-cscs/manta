use anyhow::{Context, Error};
use tracing_subscriber::EnvFilter;

/// Configure the global tracing subscriber and bridge `log::` calls into it.
///
/// `log_level` is an `EnvFilter` directive string, e.g. `"info"`, `"debug"`,
/// or `"manta=debug,hyper=warn"`. Falls back to `"error"` on parse failure.
pub fn configure(log_level: String) -> Result<(), Error> {
  let filter = EnvFilter::try_new(&log_level)
    .unwrap_or_else(|_| EnvFilter::new("error"));

  tracing_subscriber::fmt()
    .with_env_filter(filter)
    .without_time()
    .with_target(false)
    .init();

  // Route all log:: macro calls from dependencies into tracing.
  tracing_log::LogTracer::init()
    .context("Failed to install log→tracing bridge")?;

  Ok(())
}
