use std::str::FromStr;

use anyhow::Context;
use log::LevelFilter;
use log4rs::{
  Config,
  append::console::ConsoleAppender,
  config::{Appender, Logger, Root},
  encode::pattern::PatternEncoder,
};

// Code base log4rs configuration to avoid having a separate file for this to keep portability
pub fn configure(log_level: String) -> Result<(), anyhow::Error> {
  let stdout = ConsoleAppender::builder()
    .encoder(Box::new(PatternEncoder::new("{h({l}):5.5} | {m}{n}")))
    .build();

  let config_builder = Config::builder()
    .appender(Appender::builder().build("stdout", Box::new(stdout)))
    .logger(
      Logger::builder()
        .appender("stdout")
        .build("app::backend", LevelFilter::Info),
    );

  let config = config_builder
    .build(
      Root::builder()
        .appender("stdout")
        .build(LevelFilter::from_str(&log_level).unwrap_or(LevelFilter::Error)),
    )
    .context("Failed to build log4rs configuration")?;

  let _handle =
    log4rs::init_config(config).context("Failed to initialize log4rs")?;

  // use handle to change logger configuration at runtime

  Ok(())
}
