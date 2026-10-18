use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use arrow::array::{ArrayRef, Float64Array, Int64Array, StringArray};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use async_trait::async_trait;
use chrono::{Duration, NaiveDate, Utc};
use log::{info, warn};
use parquet::arrow::ArrowWriter;
use parquet::file::properties::WriterProperties;
use tokio::sync::Mutex;

use super::{SnapshotStore, StoreError};
use crate::processor::record::IndexRecord;

const RETENTION_DAYS: i64 = 7;
const DEFAULT_DUMP_EVERY: usize = 100_000;
const FILE_PREFIX: &str = "home_mixer_phoenix_scored_candidates";

pub struct AnalysisDumpStore {
    output_dir: PathBuf,
    dump_every: usize,
    buffer: Mutex<Vec<IndexRecord>>,
    dump_completed: Arc<AtomicBool>,
}

impl AnalysisDumpStore {
    pub fn new(output_dir: PathBuf, dump_every: Option<usize>) -> Self {
        std::fs::create_dir_all(&output_dir).ok();
        Self {
            output_dir,
            dump_every: dump_every.unwrap_or(DEFAULT_DUMP_EVERY),
            buffer: Mutex::new(Vec::new()),
            dump_completed: Arc::new(AtomicBool::new(false)),
        }
    }

    fn cleanup_old_dirs(&self) {
        let cutoff = Utc::now().date_naive() - Duration::days(RETENTION_DAYS - 1);

        let entries = match std::fs::read_dir(&self.output_dir) {
            Ok(e) => e,
            Err(_) => return,
        };

        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.len() != 8 || !name.chars().all(|c| c.is_ascii_digit()) {
                continue;
            }
            let dir_date = match NaiveDate::parse_from_str(&name, "%Y%m%d") {
                Ok(d) => d,
                Err(_) => continue,
            };
            if dir_date < cutoff {
                if let Err(e) = std::fs::remove_dir_all(entry.path()) {
                    warn!("failed to remove old dump dir {name}: {e}");
                } else {
                    info!("removed old dump directory {name}");
                }
            }
        }
    }

    fn dump_records(output_dir: &Path, records: &[IndexRecord]) -> anyhow::Result<()> {
        if records.is_empty() {
            return Ok(());
        }

        let now = Utc::now();
        let date_dir = now.format("%Y%m%d").to_string();
        let timestamp = now.format("%Y%m%d_%H%M%S").to_string();
        let filename = format!("{FILE_PREFIX}_{timestamp}.parquet");
        let dir = output_dir.join(&date_dir);
        std::fs::create_dir_all(&dir)?;
        let path = dir.join(&filename);

        let mut all_score_names: Vec<String> = Vec::new();
        for r in records {
            if let IndexRecord::Analysis { scores, .. } = r {
                for key in scores.keys() {
                    if !all_score_names.contains(key) {
                        all_score_names.push(key.clone());
                    }
                }
            }
        }
        all_score_names.sort();

        let mut fields = vec![
            Field::new("post_id", DataType::Int64, true),
            Field::new("author_id", DataType::Int64, true),
            Field::new("viewer_id", DataType::Int64, true),
            Field::new("served_type", DataType::Utf8, true),
        ];
        for name in &all_score_names {
            fields.push(Field::new(name, DataType::Float64, true));
        }
        let schema = Arc::new(Schema::new(fields));

        let mut post_ids = Vec::with_capacity(records.len());
        let mut author_ids = Vec::with_capacity(records.len());
        let mut viewer_ids = Vec::with_capacity(records.len());
        let mut served_types: Vec<Option<String>> = Vec::with_capacity(records.len());
        let mut score_columns: Vec<Vec<Option<f64>>> =
            vec![Vec::with_capacity(records.len()); all_score_names.len()];

        for r in records {
            match r {
                IndexRecord::Analysis {
                    post_id,
                    author_id,
                    viewer_id,
                    served_type,
                    scores,
                } => {
                    post_ids.push(*post_id);
                    author_ids.push(*author_id);
                    viewer_ids.push(*viewer_id);
                    served_types.push(served_type.clone());

                    for (i, name) in all_score_names.iter().enumerate() {
                        score_columns[i].push(scores.get(name).copied());
                    }
                }
                _ => continue,
            }
        }

        let mut columns: Vec<ArrayRef> = vec![
            Arc::new(Int64Array::from(post_ids)),
            Arc::new(Int64Array::from(author_ids)),
            Arc::new(Int64Array::from(viewer_ids)),
            Arc::new(StringArray::from(served_types)),
        ];
        for col in score_columns {
            columns.push(Arc::new(Float64Array::from(col)));
        }

        let batch = RecordBatch::try_new(schema, columns)?;

        let file = std::fs::File::create(&path)?;
        let props = WriterProperties::builder()
            .set_compression(parquet::basic::Compression::SNAPPY)
            .build();
        let mut writer = ArrowWriter::try_new(file, batch.schema(), Some(props))?;
        writer.write(&batch)?;
        writer.close()?;

        info!("wrote {} records to {}", batch.num_rows(), filename);
        Ok(())
    }
}

#[async_trait]
impl SnapshotStore for AnalysisDumpStore {
    async fn add_batch(&self, records: Vec<IndexRecord>) -> Result<(), StoreError> {
        let mut buffer = self.buffer.lock().await;
        buffer.extend(records);

        let mut dumped = false;
        while buffer.len() >= self.dump_every {
            let chunk: Vec<IndexRecord> = buffer.drain(..self.dump_every).collect();
            let output_dir = self.output_dir.clone();
            tokio::task::spawn_blocking(move || Self::dump_records(&output_dir, &chunk))
                .await
                .map_err(|e| StoreError::ParquetWrite(e.into()))?
                .map_err(StoreError::ParquetWrite)?;
            dumped = true;
        }

        if dumped {
            self.dump_completed.store(true, Ordering::Release);
        }

        Ok(())
    }

    async fn start(&self) -> Result<(), StoreError> {
        self.cleanup_old_dirs();
        info!("AnalysisDumpStore writing to {}", self.output_dir.display());
        Ok(())
    }

    async fn stop(&self) -> Result<(), StoreError> {
        let remaining: Vec<IndexRecord> = {
            let mut buffer = self.buffer.lock().await;
            buffer.drain(..).collect()
        };

        if !remaining.is_empty() {
            let output_dir = self.output_dir.clone();
            tokio::task::spawn_blocking(move || Self::dump_records(&output_dir, &remaining))
                .await
                .map_err(|e| StoreError::ParquetWrite(e.into()))?
                .map_err(StoreError::ParquetWrite)?;
        }

        info!("AnalysisDumpStore stopped");
        Ok(())
    }

    fn dump_completed_flag(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.dump_completed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn make_analysis_record(post_id: i64, scores: Vec<(&str, f64)>) -> IndexRecord {
        IndexRecord::Analysis {
            post_id,
            author_id: 100,
            viewer_id: 200,
            served_type: Some("RecTweet".to_string()),
            scores: scores
                .into_iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect(),
        }
    }

    #[tokio::test]
    async fn dump_when_buffer_full() {
        let dir = TempDir::new().unwrap();
        let store = AnalysisDumpStore::new(dir.path().to_path_buf(), Some(3));

        let records: Vec<IndexRecord> = (0..5)
            .map(|i| make_analysis_record(i + 100, vec![("score_a", 0.5 + i as f64)]))
            .collect();

        store.add_batch(records).await.unwrap();

        let date_dir = Utc::now().format("%Y%m%d").to_string();
        let date_path = dir.path().join(&date_dir);
        assert!(date_path.exists(), "date dir should exist");

        let parquet_files: Vec<_> = std::fs::read_dir(&date_path)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "parquet"))
            .collect();
        assert_eq!(parquet_files.len(), 1, "one parquet file from the dump");
    }

    #[tokio::test]
    async fn stop_flushes_remaining() {
        let dir = TempDir::new().unwrap();
        let store = AnalysisDumpStore::new(dir.path().to_path_buf(), Some(100));

        let records = vec![
            make_analysis_record(100, vec![("x", 0.1)]),
            make_analysis_record(200, vec![("x", 0.2)]),
        ];

        store.add_batch(records).await.unwrap();

        let date_dir = Utc::now().format("%Y%m%d").to_string();
        let date_path = dir.path().join(&date_dir);
        assert!(!date_path.exists(), "no dump yet");

        store.stop().await.unwrap();
        assert!(date_path.exists(), "date dir should exist after stop");

        let parquet_files: Vec<_> = std::fs::read_dir(&date_path)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "parquet"))
            .collect();
        assert_eq!(parquet_files.len(), 1, "flush on stop");
    }

    #[tokio::test]
    async fn dynamic_score_columns() {
        let dir = TempDir::new().unwrap();
        let store = AnalysisDumpStore::new(dir.path().to_path_buf(), Some(100));

        let records = vec![
            make_analysis_record(100, vec![("engagement", 0.9), ("quality", 0.8)]),
            make_analysis_record(200, vec![("engagement", 0.7)]),
        ];

        store.add_batch(records).await.unwrap();
        store.stop().await.unwrap();

        let date_dir = Utc::now().format("%Y%m%d").to_string();
        let date_path = dir.path().join(&date_dir);
        let parquet_files: Vec<_> = std::fs::read_dir(&date_path)
            .unwrap()
            .filter_map(|e| e.ok())
            .collect();

        let batches = crate::store::writer::read_parquet_file(&parquet_files[0].path()).unwrap();
        let batch = &batches[0];

        assert_eq!(batch.num_rows(), 2);
        assert_eq!(batch.num_columns(), 6);

        let schema = batch.schema();
        let col_names: Vec<&str> = schema.fields().iter().map(|f| f.name().as_str()).collect();
        assert!(col_names.contains(&"engagement"));
        assert!(col_names.contains(&"quality"));
    }
}
