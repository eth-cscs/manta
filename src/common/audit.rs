use anyhow::{Context, Result};
use manta_backend_dispatcher::interfaces::hsm::group::GroupTrait;
use serde::{Deserialize, Serialize};

use super::{jwt_ops, kafka::Kafka};
use crate::manta_backend_dispatcher::StaticBackendDispatcher;

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
      tracing::warn!("Failed serializing audit message: {}", e);
      return;
    }
  };

  if let Err(e) = kafka.produce_message(msg_data.as_bytes()).await {
    tracing::warn!("Failed producing audit message: {}", e);
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
    tracing::warn!("Failed to extract user name from JWT for audit: {}", e);
    String::new()
  });
  let user_id =
    jwt_ops::get_preferred_username(token).unwrap_or_else(|e| {
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

  send_audit_message(kafka, msg).await;
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

/// Send an audit message with group names resolved from a
/// backend lookup.
///
/// Like [`maybe_send_audit`], but first queries the backend to
/// discover which HSM groups the given `xnames` belong to, and
/// includes those group names in the audit message.
pub async fn maybe_send_audit_with_group_lookup(
  kafka_opt: Option<&Kafka>,
  backend: &StaticBackendDispatcher,
  token: &str,
  message: impl Into<String>,
  xnames: &[String],
) -> Result<()> {
  if let Some(kafka) = kafka_opt {
    let xname_refs: Vec<&str> =
      xnames.iter().map(String::as_str).collect();

    let group_map = backend
      .get_group_map_and_filter_by_member_vec(token, &xname_refs)
      .await
      .context("Failed to get group map for audit")?;

    send_audit(
      kafka,
      token,
      message,
      Some(serde_json::json!(xnames)),
      Some(serde_json::json!(group_map.keys().collect::<Vec<_>>())),
    )
    .await;
  }
  Ok(())
}
