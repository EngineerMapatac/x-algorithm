pub mod analysis;
pub mod base;
pub mod snowflake;
pub mod writer;

use crate::config::WindowConfig;
use crate::processor::record::IndexRecord;
use async_trait::async_trait;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("parquet write error: {0}")]
    ParquetWrite(#[from] anyhow::Error),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

pub type WindowRouter = Box<dyn Fn(&IndexRecord, &[WindowConfig]) -> Vec<String> + Send + Sync>;

#[derive(Debug, Clone)]
pub struct StoreConfig {
        pub output_dir: PathBuf,
        pub windows: Vec<WindowConfig>,
        pub compaction_interval_secs: u64,
        pub versions_to_keep: usize,
            pub pipeline: String,
                pub sid_num_levels: usize,
}

#[async_trait]
pub trait SnapshotStore: Send + Sync {
        async fn add_batch(&self, records: Vec<IndexRecord>) -> Result<(), StoreError>;

        async fn start(&self) -> Result<(), StoreError>;

        async fn stop(&self) -> Result<(), StoreError>;

                fn dump_completed_flag(&self) -> Arc<AtomicBool>;
}

#[async_trait]
impl<T: SnapshotStore + ?Sized> SnapshotStore for Arc<T> {
    async fn add_batch(&self, records: Vec<IndexRecord>) -> Result<(), StoreError> {
        T::add_batch(self.as_ref(), records).await
    }
    async fn start(&self) -> Result<(), StoreError> {
        T::start(self.as_ref()).await
    }
    async fn stop(&self) -> Result<(), StoreError> {
        T::stop(self.as_ref()).await
    }
    fn dump_completed_flag(&self) -> Arc<AtomicBool> {
        T::dump_completed_flag(self.as_ref())
    }
}
