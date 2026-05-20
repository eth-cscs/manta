use std::{fmt, sync::OnceLock, time::Duration};

use rdkafka::{
  ClientConfig,
  producer::{FutureProducer, FutureRecord},
};
use serde::{Deserialize, Serialize};

use super::{audit::Audit, error::MantaError};

/// Kafka message delivery timeout in milliseconds.
const KAFKA_MESSAGE_TIMEOUT_MS: &str = "5000";

/// How long to wait for Kafka delivery confirmation.
/// Zero means fire-and-forget.
const KAFKA_DELIVERY_WAIT: Duration = Duration::from_secs(0);

/// Kafka client configuration for audit message production.
///
/// The [`FutureProducer`] is lazily created on the first
/// call to [`Audit::produce_message`] and reused for all
/// subsequent calls via an internal [`OnceLock`].
#[derive(Serialize, Deserialize)]
pub struct Kafka {
  pub brokers: Vec<String>,
  pub topic: String,
  #[serde(skip)]
  producer: OnceLock<FutureProducer>,
}

impl fmt::Debug for Kafka {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("Kafka")
      .field("brokers", &self.brokers)
      .field("topic", &self.topic)
      .field(
        "producer",
        &if self.producer.get().is_some() {
          "Some(<FutureProducer>)"
        } else {
          "None"
        },
      )
      .finish()
  }
}

impl Clone for Kafka {
  /// Clone the configuration only; the cached producer is
  /// not cloned and will be lazily recreated.
  fn clone(&self) -> Self {
    Self {
      brokers: self.brokers.clone(),
      topic: self.topic.clone(),
      producer: OnceLock::new(),
    }
  }
}

impl Kafka {
  /// Create a new `Kafka` instance with the given broker
  /// list and topic name.
  pub fn new(brokers: Vec<String>, topic: String) -> Self {
    Self {
      brokers,
      topic,
      producer: OnceLock::new(),
    }
  }

  /// Return the cached [`FutureProducer`], creating it on
  /// first call.
  fn get_or_init_producer(&self) -> Result<&FutureProducer, MantaError> {
    if let Some(p) = self.producer.get() {
      return Ok(p);
    }
    let brokers = self.brokers.join(",");
    let p: FutureProducer = ClientConfig::new()
      .set("bootstrap.servers", &brokers)
      .set("message.timeout.ms", KAFKA_MESSAGE_TIMEOUT_MS)
      .create()
      .map_err(|e| {
        MantaError::KafkaError(format!(
          "Failed to create Kafka producer: {}",
          e
        ))
      })?;
    // Another thread may have raced us; either value is
    // fine since they are configured identically.
    Ok(self.producer.get_or_init(|| p))
  }
}

impl Audit for Kafka {
  async fn produce_message(&self, data: &[u8]) -> Result<(), MantaError> {
    let producer = self.get_or_init_producer()?;

    let delivery_status = producer
      .send::<Vec<u8>, _, _>(
        FutureRecord::to(&self.topic).payload(data),
        KAFKA_DELIVERY_WAIT,
      )
      .await;

    match delivery_status {
      Ok(_) => {
        tracing::info!("Delivery status for message received");
      }
      Err(e) => {
        return Err(MantaError::KafkaError(format!(
          "Delivery status for message failed: {:?}",
          e.0
        )));
      }
    }

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  //! Tests for the non-IO parts of [`Kafka`]: configuration plumbing,
  //! clone semantics, and the redacted Debug representation.
  //! `produce_message` and `get_or_init_producer` require a broker
  //! (or a librdkafka mock) and are exercised via integration tests.

  use super::*;

  #[test]
  fn new_round_trips_brokers_and_topic() {
    let k = Kafka::new(
      vec!["broker1:9092".into(), "broker2:9092".into()],
      "audit-events".into(),
    );
    assert_eq!(k.brokers, vec!["broker1:9092", "broker2:9092"]);
    assert_eq!(k.topic, "audit-events");
    assert!(
      k.producer.get().is_none(),
      "producer must be uninitialised on construction (lazy init)"
    );
  }

  #[test]
  fn clone_resets_the_producer_cache() {
    // The `Clone` impl deliberately drops the cached producer —
    // otherwise two `Kafka` values would share rdkafka state in a
    // way the rdkafka APIs don't sanction. A future "fix" that
    // shares the OnceLock would break the lazy-init contract; this
    // test makes that change deliberate.
    let original = Kafka::new(vec!["b:9092".into()], "t".into());
    let cloned = original.clone();
    assert_eq!(cloned.brokers, original.brokers);
    assert_eq!(cloned.topic, original.topic);
    assert!(
      cloned.producer.get().is_none(),
      "cloned producer cache must be empty regardless of source state"
    );
  }

  #[test]
  fn debug_masks_the_producer_and_shows_init_state() {
    // The Debug impl deliberately substitutes a placeholder string
    // for the FutureProducer — librdkafka internals would otherwise
    // appear in log lines if a Kafka value is debug-printed. Pin
    // both the placeholder string and that brokers/topic remain
    // visible (they're not secret).
    let uninit = Kafka::new(vec!["b:9092".into()], "audit".into());
    let s = format!("{:?}", uninit);
    assert!(s.contains("brokers"), "brokers field must be visible");
    assert!(s.contains("\"b:9092\""), "broker value must be visible");
    assert!(s.contains("audit"), "topic must be visible");
    assert!(
      s.contains("None"),
      "uninitialised producer must show as `None`, got: {s}"
    );
    // The literal placeholder string used for an initialised producer
    // is pinned indirectly: if `Some(<FutureProducer>)` ever leaks
    // through to Debug output of an uninit Kafka, this assertion
    // catches it.
    assert!(
      !s.contains("FutureProducer"),
      "uninitialised Kafka must not mention FutureProducer in Debug output"
    );
  }
}
