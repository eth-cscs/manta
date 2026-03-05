use std::time::Duration;

use rdkafka::{
  ClientConfig,
  producer::{FutureProducer, FutureRecord},
};
use serde::{Deserialize, Serialize};

use super::audit::Audit;

use anyhow::{Context, Result};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Kafka {
  pub brokers: Vec<String>,
  pub topic: String,
}

impl Audit for Kafka {
  async fn produce_message(&self, data: &[u8]) -> Result<()> {
    let brokers: &str = &self.brokers.join(",");
    let topic_name: &str = &self.topic;

    let producer: &FutureProducer = &ClientConfig::new()
      .set("bootstrap.servers", brokers)
      .set("message.timeout.ms", "5000")
      .create()
      .context("Failed to create Kafka producer")?;

    let delivery_status = producer
      .send::<Vec<u8>, _, _>(
        FutureRecord::to(topic_name).payload(data),
        Duration::from_secs(0),
      )
      .await;

    // This will be executed when the result is received.
    match delivery_status {
      Ok(_) => {
        log::info!("Delivery status for message received");
      }
      Err(e) => {
        return Err(anyhow::anyhow!(
          "Delivery status for message failed: {:?}",
          e.0
        ));
      }
    }

    Ok(())
  }
}
