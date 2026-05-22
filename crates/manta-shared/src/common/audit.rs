//! Audit trail helpers: build and send structured JSON messages to Kafka.

use serde::{Deserialize, Serialize};

use super::{error::MantaError, jwt_ops, kafka::Kafka};

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

/// Build the JSON payload that [`send_audit`] sends to Kafka.
///
/// Split out as a pure function so unit tests can pin the wire
/// shape (key names, optional-field inclusion, JWT fallback) without
/// needing a Kafka broker.
pub(crate) fn build_audit_message(
  token: &str,
  message: impl Into<String>,
  host: Option<serde_json::Value>,
  group: Option<serde_json::Value>,
) -> serde_json::Value {
  let username = jwt_ops::get_name(token).unwrap_or_else(|e| {
    tracing::warn!("Failed to extract user name from JWT for audit: {}", e);
    String::new()
  });
  let user_id = jwt_ops::get_preferred_username(token).unwrap_or_else(|e| {
    tracing::warn!("Failed to extract user ID from JWT for audit: {}", e);
    String::new()
  });

  let mut msg = serde_json::json!({
    "user": {"id": user_id, "name": username},
    "message": message.into(),
  });

  if let Some(h) = host {
    msg["host"] = serde_json::json!({"hostname": h});
  }
  if let Some(g) = group {
    msg["group"] = g;
  }

  msg
}

/// Build and send an audit message to Kafka.
///
/// Extracts user identity from the JWT token (falling
/// back to empty strings on parse failure) and
/// constructs a JSON message with the provided fields.
///
/// Both `host` and `group` are optional — they are only
/// included in the JSON if `Some`.
pub async fn send_audit(
  kafka: &Kafka,
  token: &str,
  message: impl Into<String>,
  host: Option<serde_json::Value>,
  group: Option<serde_json::Value>,
) {
  send_audit_message(kafka, build_audit_message(token, message, host, group))
    .await;
}

/// Send an audit message if a Kafka instance is configured.
///
/// This is a convenience wrapper around [`send_audit`] that
/// handles the common `if let Some(kafka) = kafka_opt { ... }`
/// pattern found at every audit call site.
pub async fn maybe_send_audit(
  kafka_opt: Option<&Kafka>,
  token: &str,
  message: impl Into<String>,
  host: Option<serde_json::Value>,
  group: Option<serde_json::Value>,
) {
  if let Some(kafka) = kafka_opt {
    send_audit(kafka, token, message, host, group).await;
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
  use serde_json::json;

  /// Build a minimal JWT whose `name` and `preferred_username` claims
  /// can be extracted by `jwt_ops::get_name` / `get_preferred_username`.
  fn jwt_with(name: &str, preferred_username: &str) -> String {
    use base64::prelude::*;
    let header = BASE64_URL_SAFE_NO_PAD.encode(r#"{"alg":"none","typ":"JWT"}"#);
    let body = BASE64_URL_SAFE_NO_PAD.encode(
      json!({"name": name, "preferred_username": preferred_username})
        .to_string(),
    );
    format!("{header}.{body}.sig")
  }

  // ---- build_audit_message ----

  #[test]
  fn audit_includes_user_message_keys_unconditionally() {
    let msg = build_audit_message(
      &jwt_with("Alice", "alice"),
      "deleted node",
      None,
      None,
    );
    assert_eq!(msg["user"]["name"], "Alice");
    assert_eq!(msg["user"]["id"], "alice");
    assert_eq!(msg["message"], "deleted node");
    // No `host` or `group` keys when both are None.
    assert!(msg.get("host").is_none(), "host must be omitted when None");
    assert!(
      msg.get("group").is_none(),
      "group must be omitted when None"
    );
  }

  #[test]
  fn audit_wraps_host_in_hostname_object() {
    // Host is wrapped as `{"hostname": <provided>}` — a "simplification"
    // that flattened this to `"host": <provided>` would break log
    // ingestion downstream. Pin the structure.
    let msg = build_audit_message(
      &jwt_with("a", "a"),
      "m",
      Some(json!("x3000c0s1b0n0")),
      None,
    );
    assert_eq!(msg["host"], json!({"hostname": "x3000c0s1b0n0"}));
  }

  #[test]
  fn audit_inserts_group_value_as_is() {
    // Group is passed through verbatim — caller is responsible for
    // pre-shaping it. Pin so a future "wrap it like host" doesn't
    // silently change the wire shape.
    let group = json!({"name": "compute", "members": ["x1", "x2"]});
    let msg =
      build_audit_message(&jwt_with("a", "a"), "m", None, Some(group.clone()));
    assert_eq!(msg["group"], group);
  }

  #[test]
  fn audit_falls_back_to_empty_strings_on_malformed_jwt() {
    // "nodots" can't be parsed; both jwt_ops calls return Err. The
    // audit fallback turns those into empty strings rather than
    // dropping the audit event entirely or panicking.
    let msg = build_audit_message("nodots", "m", None, None);
    assert_eq!(msg["user"]["name"], "");
    assert_eq!(msg["user"]["id"], "");
    assert_eq!(msg["message"], "m");
  }

  #[test]
  fn audit_does_not_leak_the_token_into_the_payload() {
    // Belt-and-braces: a refactor that accidentally embedded the
    // whole token in the audit payload would be a security incident.
    // Search the serialized JSON for the JWT body marker.
    let token = jwt_with("Alice", "alice");
    let msg = build_audit_message(&token, "deleted node", None, None);
    let json = serde_json::to_string(&msg).unwrap();
    assert!(
      !json.contains(&token),
      "audit payload must not contain the raw JWT"
    );
    // The base64-encoded JWT body contains "alic" (from "alice") — a
    // bare substring check would false-positive on the username, so
    // we check the full token string instead.
  }

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
