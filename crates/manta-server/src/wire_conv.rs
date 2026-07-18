//! Conversions between wire types (`manta-shared`) and backend types
//! (`manta-backend-dispatcher`).
//!
//! # Why this module exists
//!
//! Manta uses two error types (see `CLAUDE.md`'s two-tier error rule):
//!
//! - `manta_shared::common::error::MantaError` — produced by shared
//!   helpers (config loader, audit, JWT, kafka).
//! - `manta_backend_dispatcher::error::Error` — used everywhere in
//!   the server's service / backend_dispatcher layers.
//!
//! Service-layer code calls into `manta-shared` helpers but needs to
//! produce `BackendError` results. Rust's orphan rule blocks the
//! obvious `impl From<MantaError> for BackendError` because both
//! types are foreign to this crate, so this module exposes a free
//! function instead: callers write
//! `.map_err(wire_conv::to_backend)?`.
//!
//! Lives server-side rather than in `manta-shared` because
//! `manta-shared` has no knowledge of the backend dispatcher crate.
//!
//! # Scope
//!
//! Only error-type mapping is needed at runtime. A `NodeDetails`
//! converter isn't required: the only place the two `NodeDetails`
//! types meet is the HTTP wire, and the JSON serialisation is
//! identical between `csm_rs::node::types::NodeDetails` and
//! `manta_shared::types::dto::NodeDetails`.

use manta_backend_dispatcher::error::Error as BackendError;
use manta_shared::common::error::MantaError;
use manta_shared::common::error_code::ErrorCode;

/// Map a `MantaError` (returned by manta-shared's pure helpers) onto
/// the structured `BackendError` that the server's service layer uses.
///
/// **Exhaustiveness** is enforced at compile time: `MantaError` is
/// not `#[non_exhaustive]`, so adding a new variant breaks this
/// `match` with E0004. A reviewer suggesting the test suite is the
/// only line of defence misread the structure — the tests below pin
/// per-variant *payload* preservation, not exhaustiveness, and they
/// stay relevant only as a guard against silent renames within the
/// existing arms (e.g. `BackendError::Message` → `BackendError::Other`).
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

/// Classify a [`BackendError`] into its stable [`ErrorCode`]
/// (see `ERRORS.md` and the catalog in
/// `manta_shared::common::error_code`).
///
/// `BackendError` is a foreign, pinned crate, so codes cannot live on
/// the type itself; this function is the single place the mapping
/// exists. The match is deliberately **exhaustive** (no wildcard):
/// when a dependency bump adds a variant, compilation fails here and
/// forces a classification instead of silently falling into a
/// catch-all.
///
/// `NetError` is split by shape the same way `to_handler_error`
/// splits its status: timeout → [`ErrorCode::BackendTimeout`],
/// connect failure → [`ErrorCode::BackendConnectFailed`], anything
/// else → [`ErrorCode::NetworkError`].
#[must_use]
pub fn backend_error_code(e: &BackendError) -> ErrorCode {
  match e {
    BackendError::Message(_) => ErrorCode::Internal,
    BackendError::IoError(_) => ErrorCode::IoError,
    BackendError::SerdeError(_) => ErrorCode::SerdeError,
    BackendError::NetError(rqe) if rqe.is_timeout() => {
      ErrorCode::BackendTimeout
    }
    BackendError::NetError(rqe) if rqe.is_connect() => {
      ErrorCode::BackendConnectFailed
    }
    BackendError::NetError(_) => ErrorCode::NetworkError,
    BackendError::RequestError { .. } => ErrorCode::NetworkError,
    BackendError::CsmError { .. } => ErrorCode::BackendHttpError,
    BackendError::ConsoleError(_) => ErrorCode::ConsoleError,
    BackendError::ConfigurationAlreadyExistsError(_) => {
      ErrorCode::ConfigurationAlreadyExists
    }
    BackendError::ConfigurationNotFound => ErrorCode::ConfigurationNotFound,
    BackendError::SessionNotFound => ErrorCode::SessionNotFound,
    BackendError::AuthenticationTokenNotFound(_) => {
      ErrorCode::AuthTokenNotFound
    }
    BackendError::NotFound(_) => ErrorCode::NotFound,
    BackendError::BadRequest(_) => ErrorCode::BadRequest,
    BackendError::Conflict(_) => ErrorCode::Conflict,
    BackendError::TomlEditError(_) | BackendError::TomlSerError(_) => {
      ErrorCode::TomlError
    }
    BackendError::ConfigError(_) => ErrorCode::ConfigError,
    // Interactive-prompt failures make no sense server-side; if one
    // ever surfaces here it is an internal fault, not a user error.
    BackendError::DialoguerError(_) => ErrorCode::Internal,
    BackendError::K8sError(_) => ErrorCode::K8sError,
    BackendError::YamlError(_) => ErrorCode::YamlError,
    BackendError::TemplateError(_) => ErrorCode::TemplateError,
    BackendError::JwtMalformed(_) => ErrorCode::JwtMalformed,
    BackendError::InvalidPattern(_) => ErrorCode::InvalidPattern,
    BackendError::InsufficientResources(_) => ErrorCode::InsufficientResources,
    BackendError::MissingField(_) => ErrorCode::MissingField,
    BackendError::UnsupportedBackend(_) => ErrorCode::UnsupportedBackend,
    BackendError::InvalidNodeId(_) => ErrorCode::InvalidNodeId,
    BackendError::KafkaError(_) => ErrorCode::KafkaError,
    BackendError::HookError(_) => ErrorCode::HookError,
    BackendError::LocalGitError(_) => ErrorCode::GitError,
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

  // Spot-pins for `backend_error_code`. Exhaustiveness is already
  // compile-time enforced (no wildcard arm); these lock the arms
  // where the code name differs from the variant name or where a
  // mix-up would be silent (e.g. Message → Internal, LocalGitError →
  // GitError, CsmError → BackendHttpError).
  #[test]
  fn backend_error_code_pins_renamed_arms() {
    let cases: &[(BackendError, ErrorCode)] = &[
      (BackendError::Message("m".into()), ErrorCode::Internal),
      (
        BackendError::CsmError {
          status: 404,
          detail: "d".into(),
          body: None,
        },
        ErrorCode::BackendHttpError,
      ),
      (
        BackendError::AuthenticationTokenNotFound("f".into()),
        ErrorCode::AuthTokenNotFound,
      ),
      (
        BackendError::ConfigurationAlreadyExistsError("c".into()),
        ErrorCode::ConfigurationAlreadyExists,
      ),
      (BackendError::LocalGitError("g".into()), ErrorCode::GitError),
      (BackendError::NotFound("n".into()), ErrorCode::NotFound),
      (
        BackendError::InvalidPattern("p".into()),
        ErrorCode::InvalidPattern,
      ),
    ];
    for (input, expected) in cases {
      assert_eq!(
        backend_error_code(input),
        *expected,
        "wrong code for {input:?}"
      );
    }
  }

  #[test]
  fn backend_error_code_io_error_maps_to_io_code() {
    let e = BackendError::IoError(std::io::Error::other("disk on fire"));
    assert_eq!(backend_error_code(&e), ErrorCode::IoError);
  }
}
