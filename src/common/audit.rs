use anyhow::Result;
use serde::{Deserialize, Serialize};

use super::kafka::Kafka;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Auditor {
  pub kafka: Kafka,
  // pub syslog: Option<Syslog>,
}

pub trait Audit {
  async fn produce_message(&self, data: &[u8]) -> Result<()>;
}
