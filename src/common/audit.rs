use anyhow::Result;
use serde::{Deserialize, Serialize};

use super::{jwt_ops, kafka::Kafka};

#[derive(Serialize, Deserialize, Debug, Clone)]
/// Wraps a [`Kafka`] instance for sending audit messages.
pub struct Auditor {
  pub kafka: Kafka,
}

/// Trait for producing audit messages to a message broker.
pub trait Audit {
  async fn produce_message(&self, data: &[u8]) -> Result<()>;
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
      log::warn!("Failed serializing audit message: {}", e);
      return;
    }
  };

  if let Err(e) = kafka.produce_message(msg_data.as_bytes()).await {
    log::warn!("Failed producing audit message: {}", e);
  }
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
  let username = jwt_ops::get_name(token).unwrap_or_else(|e| {
    log::warn!("Failed to extract user name from JWT for audit: {}", e);
    String::new()
  });
  let user_id =
    jwt_ops::get_preferred_username(token).unwrap_or_else(|e| {
      log::warn!("Failed to extract user ID from JWT for audit: {}", e);
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

  send_audit_message(kafka, msg).await;
}
