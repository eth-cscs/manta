//! Structured error type returned by `manta-shared`'s pure helpers.
//!
//! Replaces the previous use of `manta_backend_dispatcher::error::Error`
//! in audit / jwt / kafka / config / sat-file / network-probe helpers,
//! so that this crate (and therefore `manta-cli`) no longer needs to
//! reach into backend-dispatcher types for its error surface.
//!
//! Server-side code keeps returning `manta_backend_dispatcher::error::Error`
//! and uses `?` to convert `MantaError` at the call site via the
//! `From<MantaError> for BackendError` impl in
//! `crates/manta-server/src/wire_conv.rs`.

use thiserror::Error;

/// Errors returned by `manta-shared`'s pure helpers.
#[derive(Error, Debug)]
pub enum MantaError {
  #[error("IO error: {0}")]
  IoError(#[from] std::io::Error),
  #[error("Config error: {0}")]
  ConfigError(#[from] config::ConfigError),
  #[error("TOML edit error: {0}")]
  TomlEditError(#[from] toml_edit::TomlError),
  #[error("Serde error: {0}")]
  SerdeError(#[from] serde_json::Error),
  #[error("Network error: {0}")]
  NetError(#[from] reqwest::Error),
  #[error("YAML error: {0}")]
  YamlError(#[from] serde_yaml::Error),

  #[error("Not found: {0}")]
  NotFound(String),
  #[error("Missing field: {0}")]
  MissingField(String),
  #[error("JWT malformed: {0}")]
  JwtMalformed(String),
  #[error("Kafka error: {0}")]
  KafkaError(String),
  #[error("Invalid pattern: {0}")]
  InvalidPattern(String),
  #[error("Template render error: {0}")]
  TemplateError(String),

  /// Catch-all for messages that don't fit any structured variant.
  #[error("{0}")]
  Other(String),
}
