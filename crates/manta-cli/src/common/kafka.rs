use std::{fmt, sync::OnceLock, time::Duration};

use manta_backend_dispatcher::error::Error;
use rdkafka::{
  ClientConfig,
  producer::{FutureProducer, FutureRecord},
};
use serde::{Deserialize, Serialize};

use super::audit::Audit;

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
  fn get_or_init_producer(&self) -> Result<&FutureProducer, Error> {
    if let Some(p) = self.producer.get() {
      return Ok(p);
    }
    let brokers = self.brokers.join(",");
    let p: FutureProducer = ClientConfig::new()
      .set("bootstrap.servers", &brokers)
      .set("message.timeout.ms", KAFKA_MESSAGE_TIMEOUT_MS)
      .create()
      .map_err(|e| {
        Error::KafkaError(format!("Failed to create Kafka producer: {}", e))
      })?;
    // Another thread may have raced us; either value is
    // fine since they are configured identically.
    Ok(self.producer.get_or_init(|| p))
  }
}

impl Audit for Kafka {
  async fn produce_message(&self, data: &[u8]) -> Result<(), Error> {
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
        return Err(Error::KafkaError(format!(
          "Delivery status for message failed: {:?}",
          e.0
        )));
      }
    }

    Ok(())
  }
}
