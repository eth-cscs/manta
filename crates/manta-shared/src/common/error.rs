//! Structured error type returned by `manta-shared`'s pure helpers.
//!
//! Used by the shared `config` loader and re-used by binary-side
//! helpers that build on it: `manta-cli`'s SAT-file Jinja renderer and
//! `manta-server`'s `audit`, `jwt_ops`, and `kafka` modules. Lets
//! `manta-shared` (and therefore `manta-cli`) avoid pulling in
//! `manta_backend_dispatcher::error::Error` for its own error surface.
//!
//! Server-side code keeps returning `manta_backend_dispatcher::error::Error`
//! and bridges `MantaError` at call sites via the free function
//! `crates/manta-server/src/wire_conv.rs::to_backend`, used as
//! `.map_err(wire_conv::to_backend)?`. The orphan rule prevents a
//! `From<MantaError> for BackendError` impl in the server crate (both
//! types are foreign there).

use thiserror::Error;

/// Errors returned by `manta-shared`'s pure helpers.
///
/// Most helpers return `Result<T, MantaError>`. The server-side code
/// converts these to its richer `BackendError` via
/// `crates/manta-server/src/wire_conv.rs::to_backend`, which then
/// maps to HTTP status codes.
///
/// # Examples
///
/// Pattern-match a NotFound to log a custom message before propagating:
///
/// ```
/// use manta_shared::common::error::MantaError;
///
/// fn lookup_thing() -> Result<(), MantaError> {
///   Err(MantaError::NotFound("thing 42".into()))
/// }
///
/// match lookup_thing() {
///   Err(MantaError::NotFound(detail)) => {
///     // Maps to HTTP 404 server-side.
///     assert_eq!(detail, "thing 42");
///   }
///   _ => unreachable!(),
/// }
/// ```
#[derive(Error, Debug)]
pub enum MantaError {
  /// Filesystem I/O failure (config-file read, token cache write, etc.).
  #[error("IO error: {0}")]
  IoError(#[from] std::io::Error),
  /// `config` crate failure: bad TOML, env-var parse, or schema
  /// mismatch on `cli.toml` / `server.toml`.
  #[error("Config error: {0}")]
  ConfigError(#[from] config::ConfigError),
  /// `toml_edit` parse / serialize failure when editing a config
  /// file in place (e.g. `manta config set`).
  #[error("TOML edit error: {0}")]
  TomlEditError(#[from] toml_edit::TomlError),
  /// JSON serialize / deserialize failure (most often during JWT
  /// claim extraction or audit payload construction).
  #[error("Serde error: {0}")]
  SerdeError(#[from] serde_json::Error),
  /// `reqwest` failure on outbound HTTP (DNS, TLS handshake, body
  /// stream, etc.).
  #[error("Network error: {0}")]
  NetError(#[from] reqwest::Error),
  /// YAML parse / serialize failure (SAT-file rendering).
  #[error("YAML error: {0}")]
  YamlError(#[from] serde_yaml::Error),

  /// Resource lookup failed (config file missing, group not in
  /// backend, etc.). Maps to HTTP 404 server-side.
  #[error("Not found: {0}")]
  NotFound(String),
  /// A required field is absent (e.g. JWT lacks `preferred_username`,
  /// node config lacks `boot_image_id`).
  #[error("Missing field: {0}")]
  MissingField(String),
  /// JWT was structurally invalid (wrong number of dots, undecodable
  /// claims, non-UTF-8 payload). Maps to HTTP 401.
  #[error("JWT malformed: {0}")]
  JwtMalformed(String),
  /// Kafka producer construction or delivery failed.
  #[error("Kafka error: {0}")]
  KafkaError(String),
  /// User-supplied pattern (hardware pattern, hostlist expression,
  /// glob) didn't parse. Maps to HTTP 400.
  #[error("Invalid pattern: {0}")]
  InvalidPattern(String),
  /// Jinja2 / minijinja render failed during SAT-file processing.
  #[error("Template render error: {0}")]
  TemplateError(String),

  /// Catch-all for messages that don't fit any structured variant.
  /// Server-side this maps to HTTP 500 — prefer a typed variant when
  /// adding new failure modes.
  #[error("{0}")]
  Other(String),
}
