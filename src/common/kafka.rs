use std::time::Duration;

// use kafka::producer::{Producer, Record, RequiredAcks};
use rdkafka::{
  producer::{FutureProducer, FutureRecord},
  ClientConfig,
};
use serde::{Deserialize, Serialize};

use super::audit::Audit;

use anyhow::Result;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Kafka {
  pub brokers: Vec<String>,
  pub topic: String,
}

impl Audit for Kafka {
  /* fn produce_message(&self, data: &[u8]) -> Result<()> {
      let brokers = self.brokers.clone();
      let topic = self.topic.clone();

      println!("About to publish a message at {:?} to: {}", brokers, topic);

      // ~ create a producer. this is a relatively costly operation, so
      // you'll do this typically once in your application and re-use
      // the instance many times.
      let mut producer = Producer::from_hosts(brokers)
          // ~ give the brokers one second time to ack the message
          .with_ack_timeout(Duration::from_secs(1))
          // ~ require only one broker to ack the message
          .with_required_acks(RequiredAcks::One)
          // ~ build the producer with the above settings
          .create()?;

      // ~ now send a single message.  this is a synchronous/blocking
      // operation.

      // ~ we're sending 'data' as a 'value'. there will be no key
      // associated with the sent message.

      // ~ we leave the partition "unspecified" - this is a negative
      // partition - which causes the producer to find out one on its
      // own using its underlying partitioner.
      producer.send(&Record {
          topic: &topic,
          partition: -1,
          key: (),
          value: data,
      })?;

      // ~ we can achieve exactly the same as above in a shorter way with
      // the following call
      producer.send(&Record::from_value(&topic, data))?;

      Ok(())
  } */

  async fn produce_message(&self, data: &[u8]) -> Result<()> {
    let brokers: &str = &self.brokers.join(",");
    let topic_name: &str = &self.topic;

    let producer: &FutureProducer = &ClientConfig::new()
      .set("bootstrap.servers", brokers)
      .set("message.timeout.ms", "5000")
      .create()
      .expect("Producer creation error");

    let delivery_status = producer
      .send::<Vec<u8>, _, _>(
        FutureRecord::to(topic_name).payload(data),
        /* FutureRecord::to(topic_name)
        .payload(&format!("Message {}", i))
        .key(&format!("Key {}", i))
        .headers(OwnedHeaders::new().insert(Header {
            key: "header_key",
            value: Some("header_value"),
        })), */
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

    /* // This loop is non blocking: all messages will be sent one after the other, without waiting
    // for the results.
    let futures = (0..5)
        .map(|i| async move {
            // The send operation on the topic returns a future, which will be
            // completed once the result or failure from Kafka is received.
            let delivery_status = producer
                .send::<Vec<u8>, _, _>(
                    FutureRecord::to(topic_name).payload(data),
                    /* FutureRecord::to(topic_name)
                    .payload(&format!("Message {}", i))
                    .key(&format!("Key {}", i))
                    .headers(OwnedHeaders::new().insert(Header {
                        key: "header_key",
                        value: Some("header_value"),
                    })), */
                    Duration::from_secs(0),
                )
                .await;

            // This will be executed when the result is received.
            log::info!("Delivery status for message {} received", i);
            delivery_status
        })
        .collect::<Vec<_>>();

    // This loop will wait until all delivery statuses have been received.
    for future in futures {
        log::info!("Future completed. Result: {:?}", future.await);
    } */

    Ok(())
  }
}
