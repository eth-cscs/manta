use std::time::Duration;

use kafka::error::Error as KafkaError;
use kafka::producer::{Producer, Record, RequiredAcks};

use super::audit::Audit;

use anyhow::Result;

#[derive(Debug)]
pub struct Kafka {
    pub brokers: Vec<String>,
    pub topic: String,
}

impl Audit for Kafka {
    fn produce_message(&self, data: &[u8]) -> Result<()> {
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
    }
}
