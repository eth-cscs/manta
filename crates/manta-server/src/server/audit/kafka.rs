//! Kafka destination for audit events.
//!
//! [`Kafka`] is the `[auditor.kafka]` block of `server.toml`;
//! [`Kafka::build_sink`] turns it into a [`KafkaSink`] once, at
//! startup, so a malformed librdkafka setting fails the boot instead
//! of the first audited request.
//!
//! Emission never blocks a request: [`KafkaSink::emit`] only enqueues
//! the message into librdkafka's in-memory queue and hands the
//! delivery report to a spawned task, which logs a warning if the
//! broker never acknowledges it within `message_timeout_ms`. A down
//! broker therefore costs log lines, not latency. Messages still
//! queued at shutdown get [`KafkaSink::flush`]ed within a bounded
//! window.

use std::{collections::BTreeMap, fmt, time::Duration};

use manta_shared::common::error::MantaError;
use rdkafka::{
  ClientConfig,
  producer::{FutureProducer, FutureRecord, Producer},
};
use serde::{Deserialize, Serialize};

use super::{AuditSink, event::AuditEvent};

/// Default librdkafka `message.timeout.ms`, used when
/// `[auditor.kafka].message_timeout_ms` is absent.
const DEFAULT_KAFKA_MESSAGE_TIMEOUT_MS: u32 = 5000;

fn default_kafka_message_timeout_ms() -> u32 {
  DEFAULT_KAFKA_MESSAGE_TIMEOUT_MS
}

/// librdkafka keys owned by the typed fields; setting them again in
/// [`Kafka::properties`] is rejected rather than silently resolved.
const RESERVED_PROPERTIES: [&str; 2] =
  ["bootstrap.servers", "message.timeout.ms"];

/// `[auditor.kafka]`: where audit events are produced to.
#[derive(Serialize, Deserialize, Clone)]
pub struct Kafka {
  /// Bootstrap broker list, e.g. `["kafka.example.com:9092"]`.
  pub brokers: Vec<String>,
  /// Topic every audit event is produced to.
  pub topic: String,
  /// librdkafka `message.timeout.ms`: how long a queued event keeps
  /// retrying delivery before it is dropped (and a warning logged).
  #[serde(default = "default_kafka_message_timeout_ms")]
  pub message_timeout_ms: u32,
  /// Extra librdkafka client properties, passed through verbatim —
  /// typically transport security: `security.protocol`,
  /// `ssl.ca.location`, `sasl.mechanism`, `sasl.username`,
  /// `sasl.password`, … Values are never printed (they may hold
  /// credentials); only the keys are.
  #[serde(default)]
  pub properties: BTreeMap<String, String>,
}

impl fmt::Debug for Kafka {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("Kafka")
      .field("brokers", &self.brokers)
      .field("topic", &self.topic)
      .field("message_timeout_ms", &self.message_timeout_ms)
      .field("properties", &self.properties.keys().collect::<Vec<_>>())
      .finish()
  }
}

impl Kafka {
  /// Assemble the librdkafka client configuration.
  ///
  /// # Errors
  ///
  /// [`MantaError::KafkaError`] when `brokers` is empty or
  /// `properties` sets a key owned by a typed field.
  fn client_config(&self) -> Result<ClientConfig, MantaError> {
    if self.brokers.is_empty() {
      return Err(MantaError::KafkaError(
        "[auditor.kafka].brokers must not be empty".to_string(),
      ));
    }
    if let Some(key) = RESERVED_PROPERTIES
      .iter()
      .find(|k| self.properties.contains_key(**k))
    {
      return Err(MantaError::KafkaError(format!(
        "[auditor.kafka].properties must not set `{key}`; use the \
         `brokers` / `message_timeout_ms` fields instead"
      )));
    }
    let mut config = ClientConfig::new();
    for (key, value) in &self.properties {
      config.set(key, value);
    }
    config
      .set("bootstrap.servers", self.brokers.join(","))
      .set("message.timeout.ms", self.message_timeout_ms.to_string());
    Ok(config)
  }

  /// Build the producer. librdkafka connects lazily, so this does not
  /// need a reachable broker — but it does validate every property.
  ///
  /// # Errors
  ///
  /// [`MantaError::KafkaError`] on an invalid configuration (see
  /// `client_config`) or a property librdkafka rejects.
  pub fn build_sink(&self) -> Result<KafkaSink, MantaError> {
    let producer: FutureProducer =
      self.client_config()?.create().map_err(|e| {
        MantaError::KafkaError(format!("Failed to create Kafka producer: {e}"))
      })?;
    Ok(KafkaSink {
      producer,
      topic: self.topic.clone(),
    })
  }
}

/// A live Kafka producer for one topic.
pub struct KafkaSink {
  producer: FutureProducer,
  topic: String,
}

impl AuditSink for KafkaSink {
  fn emit(&self, event: &AuditEvent) {
    let payload = match serde_json::to_vec(event) {
      Ok(p) => p,
      Err(e) => {
        tracing::warn!(event_id = %event.event_id, "audit event not serialisable: {e}");
        return;
      }
    };
    let record = FutureRecord::to(&self.topic)
      .key(event.event_id.as_str())
      .payload(&payload);
    match self.producer.send_result(record) {
      Ok(delivery) => {
        let event_id = event.event_id.clone();
        tokio::spawn(async move {
          match delivery.await {
            Ok(Ok(_)) => {}
            Ok(Err((e, _))) => tracing::warn!(
              %event_id, "audit event not delivered to Kafka: {e}"
            ),
            Err(_) => tracing::warn!(
              %event_id, "audit event delivery report lost (producer dropped)"
            ),
          }
        });
      }
      Err((e, _)) => tracing::warn!(
        event_id = %event.event_id, "audit event not queued for Kafka: {e}"
      ),
    }
  }

  fn flush(&self, timeout: Duration) {
    if let Err(e) = self.producer.flush(timeout) {
      tracing::warn!("audit events still queued for Kafka at shutdown: {e}");
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn kafka(properties: &[(&str, &str)]) -> Kafka {
    Kafka {
      brokers: vec!["b1:9092".into(), "b2:9092".into()],
      topic: "events.manta".into(),
      message_timeout_ms: DEFAULT_KAFKA_MESSAGE_TIMEOUT_MS,
      properties: properties
        .iter()
        .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
        .collect(),
    }
  }

  #[test]
  fn deserialises_minimal_block_with_defaults() {
    let k: Kafka =
      toml::from_str("brokers = [\"b:9092\"]\ntopic = \"t\"").unwrap();
    assert_eq!(k.message_timeout_ms, DEFAULT_KAFKA_MESSAGE_TIMEOUT_MS);
    assert!(k.properties.is_empty());
  }

  #[test]
  fn deserialises_properties_table() {
    let k: Kafka = toml::from_str(
      "brokers = [\"b:9092\"]\ntopic = \"t\"\n[properties]\n\
       \"security.protocol\" = \"SASL_SSL\"",
    )
    .unwrap();
    assert_eq!(k.properties["security.protocol"], "SASL_SSL");
  }

  #[test]
  fn client_config_joins_brokers_and_passes_properties_through() {
    let c = kafka(&[("security.protocol", "SSL")])
      .client_config()
      .unwrap();
    assert_eq!(c.get("bootstrap.servers"), Some("b1:9092,b2:9092"));
    assert_eq!(c.get("message.timeout.ms"), Some("5000"));
    assert_eq!(c.get("security.protocol"), Some("SSL"));
  }

  #[test]
  fn client_config_rejects_reserved_property() {
    let err = kafka(&[("bootstrap.servers", "x:1")])
      .client_config()
      .unwrap_err();
    assert!(err.to_string().contains("bootstrap.servers"), "{err}");
  }

  #[test]
  fn client_config_rejects_empty_brokers() {
    let mut k = kafka(&[]);
    k.brokers.clear();
    assert!(k.client_config().is_err());
  }

  #[test]
  fn build_sink_rejects_unknown_librdkafka_property() {
    // librdkafka validates property names at create(); no broker needed.
    assert!(kafka(&[("no.such.property", "1")]).build_sink().is_err());
  }

  #[test]
  fn build_sink_succeeds_without_a_reachable_broker() {
    assert!(kafka(&[]).build_sink().is_ok());
  }

  #[test]
  fn debug_hides_property_values() {
    let s = format!("{:?}", kafka(&[("sasl.password", "hunter2")]));
    assert!(s.contains("sasl.password"), "{s}");
    assert!(!s.contains("hunter2"), "{s}");
  }
}
