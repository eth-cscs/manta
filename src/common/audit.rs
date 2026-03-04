use anyhow::Result;
use serde::{Deserialize, Serialize};

use super::kafka::Kafka;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Auditor {
  pub kafka: Kafka,
}

pub trait Audit {
  async fn produce_message(&self, data: &[u8]) -> Result<()>;
}

/// Serialize a JSON audit message and send it to Kafka.
///
/// Logs a warning on failure instead of propagating the
/// error, since audit failures should not abort the
/// operation.
pub async fn send_audit_message(kafka: &Kafka, msg_json: serde_json::Value) {
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
