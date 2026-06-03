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

#[cfg(test)]
mod tests {
  use super::*;

  // The string-bearing variants are 1:1 renames. A test per variant
  // pins the variant name AND the payload preservation, so a mistyped
  // arm (NotFound → BadRequest, say) would surface immediately.
  #[test]
  #[allow(clippy::type_complexity)]
  fn string_variants_preserve_payload_and_variant() {
    let cases: &[(MantaError, fn(&BackendError) -> bool)] = &[
      (
        MantaError::NotFound("a".into()),
        |e| matches!(e, BackendError::NotFound(s) if s == "a"),
      ),
      (
        MantaError::MissingField("b".into()),
        |e| matches!(e, BackendError::MissingField(s) if s == "b"),
      ),
      (
        MantaError::JwtMalformed("c".into()),
        |e| matches!(e, BackendError::JwtMalformed(s) if s == "c"),
      ),
      (
        MantaError::KafkaError("d".into()),
        |e| matches!(e, BackendError::KafkaError(s) if s == "d"),
      ),
      (
        MantaError::InvalidPattern("e".into()),
        |e| matches!(e, BackendError::InvalidPattern(s) if s == "e"),
      ),
      (
        MantaError::TemplateError("f".into()),
        |e| matches!(e, BackendError::TemplateError(s) if s == "f"),
      ),
    ];
    for (input, predicate) in cases {
      let label = format!("{input:?}");
      let mapped = to_backend(match input {
        MantaError::NotFound(s) => MantaError::NotFound(s.clone()),
        MantaError::MissingField(s) => MantaError::MissingField(s.clone()),
        MantaError::JwtMalformed(s) => MantaError::JwtMalformed(s.clone()),
        MantaError::KafkaError(s) => MantaError::KafkaError(s.clone()),
        MantaError::InvalidPattern(s) => MantaError::InvalidPattern(s.clone()),
        MantaError::TemplateError(s) => MantaError::TemplateError(s.clone()),
        _ => unreachable!(),
      });
      assert!(
        predicate(&mapped),
        "wrong mapping for {label}: got {mapped:?}"
      );
    }
  }

  // `Other` is the only RENAMED arm: MantaError::Other → BackendError::Message.
  // Easy to silently change to `BackendError::Other` if someone "fixes" it
  // and breaks every caller that depends on the catch-all being 500.
  #[test]
  fn other_maps_to_message() {
    let mapped = to_backend(MantaError::Other("oops".into()));
    assert!(
      matches!(&mapped, BackendError::Message(s) if s == "oops"),
      "Other must map to Message (became {mapped:?})"
    );
  }

  // The `#[from]`-bearing variants forward their inner error. Pin the
  // variant name; the inner type is checked by the compiler at compile
  // time so we don't need to reconstruct an exact payload.
  #[test]
  fn io_error_maps_to_backend_io_error() {
    let inner = std::io::Error::other("disk on fire");
    let mapped = to_backend(MantaError::IoError(inner));
    assert!(
      matches!(mapped, BackendError::IoError(_)),
      "IoError must round-trip to BackendError::IoError"
    );
  }

  #[test]
  fn serde_error_maps_to_backend_serde_error() {
    let inner =
      serde_json::from_str::<serde_json::Value>("not json").unwrap_err();
    let mapped = to_backend(MantaError::SerdeError(inner));
    assert!(matches!(mapped, BackendError::SerdeError(_)));
  }

  #[test]
  fn yaml_error_maps_to_backend_yaml_error() {
    let inner =
      serde_yaml::from_str::<serde_yaml::Value>("\t:bad").unwrap_err();
    let mapped = to_backend(MantaError::YamlError(inner));
    assert!(matches!(mapped, BackendError::YamlError(_)));
  }
}
