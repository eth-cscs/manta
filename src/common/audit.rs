use anyhow::Result;

pub trait Audit {
    fn produce_message(&self, data: &[u8]) -> Result<()>;
}
