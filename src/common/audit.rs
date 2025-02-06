use anyhow::Result;

pub trait Audit {
    async fn produce_message(&self, data: &[u8]) -> Result<()>;
}
