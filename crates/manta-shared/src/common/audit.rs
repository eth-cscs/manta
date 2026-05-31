//! Audit trail helpers: build and send structured JSON messages to Kafka.

use serde::{Deserialize, Serialize};

use super::{error::MantaError, kafka::Kafka};

#[derive(Serialize, Deserialize, Debug, Clone)]
/// Wraps a [`Kafka`] instance for sending audit messages.
pub struct Auditor {
  /// Kafka producer configured from `[auditor.kafka]` in the binary's
  /// config file.
  pub kafka: Kafka,
}

/// Trait for producing audit messages to a message broker.
pub trait Audit {
  /// Publish a single audit message payload. Implementations are
  /// expected to be fire-and-forget — failures should be logged but
  /// not propagated to the caller, since audit failures must not
  /// abort the outer operation.
  #[allow(async_fn_in_trait)]
  async fn produce_message(&self, data: &[u8]) -> Result<(), MantaError>;
}

/// Serialize a JSON audit message and send it to Kafka.
///
/// Logs a warning on failure instead of propagating the
/// error, since audit failures should not abort the
/// operation.
async fn send_audit_message(kafka: &Kafka, msg_json: serde_json::Value) {
  let msg_data = match serde_json::to_string(&msg_json) {
    Ok(data) => data,
    Err(e) => {
      tracing::warn!("Failed serializing audit message: {}", e);
      return;
    }
  };

  if let Err(e) = kafka.produce_message(msg_data.as_bytes()).await {
    tracing::warn!("Failed producing audit message: {}", e);
  }
}

/// Build the JSON payload that [`send_auth_audit`] sends to Kafka.
///
/// Split out so unit tests can pin the wire shape (notably: NO
/// password field, by construction — the function doesn't take one).
pub(crate) fn build_auth_audit_message(
  outcome: &str,
  username: &str,
  source_ip: &str,
  site: &str,
) -> serde_json::Value {
  serde_json::json!({
    "event": "auth_attempt",
    "outcome": outcome,
    "username": username,
    "source_ip": source_ip,
    "site": site,
  })
}

/// Send a structured audit event for an `/api/v1/auth/token` attempt.
///
/// Used by the server's auth handler — there is no JWT yet (the user is
/// asking for one), so identity is captured from the submitted username
/// rather than extracted from a token. The password is never logged.
///
/// Always Kafka-only; failures log a warning and do not abort the
/// outer auth flow.
pub async fn send_auth_audit(
  kafka_opt: Option<&Kafka>,
  outcome: &str,
  username: &str,
  source_ip: &str,
  site: &str,
) {
  let Some(kafka) = kafka_opt else { return };
  send_audit_message(
    kafka,
    build_auth_audit_message(outcome, username, source_ip, site),
  )
  .await;
}

#[cfg(test)]
mod tests {
  use super::*;

  // ---- build_auth_audit_message ----

  #[test]
  fn auth_audit_has_expected_wire_shape() {
    let msg = build_auth_audit_message("success", "alice", "10.0.0.1", "alps");
    assert_eq!(msg["event"], "auth_attempt");
    assert_eq!(msg["outcome"], "success");
    assert_eq!(msg["username"], "alice");
    assert_eq!(msg["source_ip"], "10.0.0.1");
    assert_eq!(msg["site"], "alps");
  }

  #[test]
  fn auth_audit_payload_has_no_password_field_by_construction() {
    // The function doesn't take a password — pin via the wire shape
    // that no `password` / `passwd` / `secret` key sneaks in.
    let msg = build_auth_audit_message("failure", "alice", "10.0.0.1", "alps");
    let obj = msg.as_object().expect("payload is an object");
    for forbidden in ["password", "passwd", "secret", "token"] {
      assert!(
        !obj.contains_key(forbidden),
        "auth audit payload must not contain `{forbidden}`"
      );
    }
  }

  #[test]
  fn auth_audit_handles_empty_strings_without_panicking() {
    // Some auth-failure paths pass empty source_ip or site (when not
    // resolvable). The function should still produce a well-formed
    // JSON object, not panic or omit keys.
    let msg = build_auth_audit_message("failure", "", "", "");
    assert_eq!(msg["username"], "");
    assert_eq!(msg["source_ip"], "");
    assert_eq!(msg["site"], "");
    assert_eq!(msg["event"], "auth_attempt");
  }
}
