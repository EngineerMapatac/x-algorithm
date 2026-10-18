pub mod kafka;

use async_trait::async_trait;

pub use kafka::KafkaEventConsumer;

pub type RawBatch = Vec<Vec<u8>>;

#[derive(Debug, thiserror::Error)]
pub enum ConsumerError {
    #[error("kafka error: {0}")]
    Kafka(#[from] anyhow::Error),

    #[error("consumer closed")]
    Closed,
}

#[async_trait]
pub trait EventConsumer: Send {
    async fn poll_batch(&mut self) -> Result<RawBatch, ConsumerError>;

    async fn commit(&mut self) -> Result<(), ConsumerError>;

    async fn close(&mut self) -> Result<(), ConsumerError>;
}
