//! Conversions between wire types (`manta-shared`) and backend types
//! (`manta-backend-dispatcher`).
//!
//! Lives server-side because manta-shared has no knowledge of the
//! backend crates. Orphan rules prevent us from writing
//! `impl From<MantaError> for BackendError` (both types are foreign
//! to this crate), so we expose a free function used at call sites
//! via `.map_err(wire_conv::to_backend)?`.
//!
//! A NodeDetails conversion isn't needed in-process: the type
//! boundary is HTTP, and the JSON wire shape is identical between
//! `csm_rs::node::types::NodeDetails` and
//! `manta_shared::shared::dto::NodeDetails`.

use manta_backend_dispatcher::error::Error as BackendError;
use manta_shared::common::error::MantaError;

/// Map a `MantaError` (returned by manta-shared's pure helpers) onto
/// the structured `BackendError` that the server's service layer uses.
pub fn to_backend(e: MantaError) -> BackendError {
  match e {
    MantaError::IoError(e) => BackendError::IoError(e),
    MantaError::ConfigError(e) => BackendError::ConfigError(e),
    MantaError::TomlEditError(e) => BackendError::TomlEditError(e),
    MantaError::SerdeError(e) => BackendError::SerdeError(e),
    MantaError::NetError(e) => BackendError::NetError(e),
    MantaError::YamlError(e) => BackendError::YamlError(e),
    MantaError::NotFound(s) => BackendError::NotFound(s),
    MantaError::MissingField(s) => BackendError::MissingField(s),
    MantaError::JwtMalformed(s) => BackendError::JwtMalformed(s),
    MantaError::KafkaError(s) => BackendError::KafkaError(s),
    MantaError::InvalidPattern(s) => BackendError::InvalidPattern(s),
    MantaError::TemplateError(s) => BackendError::TemplateError(s),
    MantaError::Other(s) => BackendError::Message(s),
  }
}
