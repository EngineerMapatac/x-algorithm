use std::fs;
use std::path::{Path, PathBuf};

use arrow::record_batch::RecordBatch;
use log::{info, warn};
use parquet::arrow::ArrowWriter;
use parquet::file::properties::WriterProperties;

pub fn atomic_write_parquet(
    output_dir: &Path,
    window_name: &str,
    epoch_secs: i64,
    batch: &RecordBatch,
    keep_versions: usize,
) -> anyhow::Result<usize> {
    let tmp_path = output_dir.join(format!("{window_name}.parquet.tmp"));
    let versioned_path = output_dir.join(format!("{window_name}_{epoch_secs}.parquet"));
    let symlink_path = output_dir.join(format!("{window_name}.parquet"));

    let file = fs::File::create(&tmp_path)?;
    let props = WriterProperties::builder()
        .set_compression(parquet::basic::Compression::SNAPPY)
        .build();
    let mut writer = ArrowWriter::try_new(file, batch.schema(), Some(props))?;
    writer.write(batch)?;
    writer.close()?;

    fs::rename(&tmp_path, &versioned_path)?;

    let tmp_link = output_dir.join(format!("{window_name}.parquet.link.tmp"));
    let _ = fs::remove_file(&tmp_link);

    #[cfg(unix)]
    std::os::unix::fs::symlink(versioned_path.file_name().unwrap(), &tmp_link)?;
    #[cfg(not(unix))]
    fs::copy(&versioned_path, &tmp_link)?;

    fs::rename(&tmp_link, &symlink_path)?;

    cleanup_old_versions(output_dir, window_name, keep_versions);

    let row_count = batch.num_rows();
    info!(
        "wrote {row_count} records to {}, symlink updated",
        versioned_path.file_name().unwrap().to_string_lossy()
    );

    Ok(row_count)
}

fn cleanup_old_versions(output_dir: &Path, window_name: &str, keep: usize) {
    let prefix = format!("{window_name}_");
    let suffix = ".parquet";

    let mut versioned: Vec<PathBuf> = fs::read_dir(output_dir)
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            let name = p.file_name().unwrap_or_default().to_string_lossy();
            name.starts_with(&prefix) && name.ends_with(suffix) && !name.contains(".tmp")
        })
        .collect();

    versioned.sort_unstable_by(|a, b| b.cmp(a));

    for old_file in versioned.into_iter().skip(keep) {
        if let Err(e) = fs::remove_file(&old_file) {
            warn!(
                "failed to cleanup old version {}: {e}",
                old_file.file_name().unwrap().to_string_lossy()
            );
        } else {
            info!(
                "cleaned up old version: {}",
                old_file.file_name().unwrap().to_string_lossy()
            );
        }
    }
}

pub fn find_latest_versioned_file(output_dir: &Path, window_name: &str) -> Option<PathBuf> {
    let prefix = format!("{window_name}_");
    let suffix = ".parquet";

    let mut versioned: Vec<PathBuf> = fs::read_dir(output_dir)
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            let name = p.file_name().unwrap_or_default().to_string_lossy();
            name.starts_with(&prefix) && name.ends_with(suffix) && !name.contains(".tmp")
        })
        .collect();

    versioned.sort_unstable_by(|a, b| b.cmp(a));
    versioned.into_iter().next()
}

pub fn read_parquet_file(path: &Path) -> anyhow::Result<Vec<RecordBatch>> {
    use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;

    let file = fs::File::open(path)?;
    let reader = ParquetRecordBatchReaderBuilder::try_new(file)?.build()?;
    let batches: Result<Vec<_>, _> = reader.collect();
    Ok(batches?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrow::array::{ArrayRef, Int64Array};
    use arrow::datatypes::{DataType, Field, Schema};
    use std::sync::Arc;
    use tempfile::TempDir;

    fn make_test_batch(post_ids: &[i64], author_ids: &[i64]) -> RecordBatch {
        let schema = Arc::new(Schema::new(vec![
            Field::new("post_id", DataType::Int64, true),
            Field::new("author_id", DataType::Int64, true),
        ]));
        RecordBatch::try_new(
            schema,
            vec![
                Arc::new(Int64Array::from(post_ids.to_vec())) as ArrayRef,
                Arc::new(Int64Array::from(author_ids.to_vec())) as ArrayRef,
            ],
        )
        .unwrap()
    }

    #[test]
    fn write_and_read_back() {
        let dir = TempDir::new().unwrap();
        let batch = make_test_batch(&[100, 200, 300], &[10, 20, 30]);

        let count = atomic_write_parquet(dir.path(), "1fav_1day", 1700000000, &batch, 3).unwrap();
        assert_eq!(count, 3);

        assert!(dir.path().join("1fav_1day_1700000000.parquet").exists());

        let symlink = dir.path().join("1fav_1day.parquet");
        assert!(symlink.exists());

        let read_batches = read_parquet_file(&symlink).unwrap();
        assert_eq!(read_batches.len(), 1);
        assert_eq!(read_batches[0].num_rows(), 3);

        let post_ids = read_batches[0]
            .column(0)
            .as_any()
            .downcast_ref::<Int64Array>()
            .unwrap();
        assert_eq!(post_ids.values(), &[100, 200, 300]);
    }

    #[test]
    fn symlink_swap_updates_to_latest() {
        let dir = TempDir::new().unwrap();
        let batch1 = make_test_batch(&[100], &[10]);
        let batch2 = make_test_batch(&[200, 300], &[20, 30]);

        atomic_write_parquet(dir.path(), "test", 1000, &batch1, 3).unwrap();
        atomic_write_parquet(dir.path(), "test", 2000, &batch2, 3).unwrap();

        let symlink = dir.path().join("test.parquet");
        let target = fs::read_link(&symlink).unwrap();
        assert_eq!(target.to_string_lossy(), "test_2000.parquet");

        let read_batches = read_parquet_file(&symlink).unwrap();
        assert_eq!(read_batches[0].num_rows(), 2);
    }

    #[test]
    fn cleanup_keeps_only_n_versions() {
        let dir = TempDir::new().unwrap();
        let batch = make_test_batch(&[1], &[1]);

        for ts in [1000, 2000, 3000, 4000, 5000] {
            atomic_write_parquet(dir.path(), "win", ts, &batch, 3).unwrap();
        }

        let versioned: Vec<_> = fs::read_dir(dir.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| {
                let name = e.file_name().to_string_lossy().to_string();
                name.starts_with("win_") && name.ends_with(".parquet")
            })
            .collect();
        assert_eq!(versioned.len(), 3, "should keep exactly 3 versions");

        assert!(!dir.path().join("win_1000.parquet").exists());
        assert!(!dir.path().join("win_2000.parquet").exists());
        assert!(dir.path().join("win_5000.parquet").exists());
    }

    #[test]
    fn find_latest_versioned() {
        let dir = TempDir::new().unwrap();
        let batch = make_test_batch(&[1], &[1]);

        atomic_write_parquet(dir.path(), "idx", 1000, &batch, 10).unwrap();
        atomic_write_parquet(dir.path(), "idx", 3000, &batch, 10).unwrap();
        atomic_write_parquet(dir.path(), "idx", 2000, &batch, 10).unwrap();

        let latest = find_latest_versioned_file(dir.path(), "idx").unwrap();
        assert_eq!(
            latest.file_name().unwrap().to_string_lossy(),
            "idx_3000.parquet"
        );
    }

    #[test]
    fn find_latest_versioned_returns_none_when_empty() {
        let dir = TempDir::new().unwrap();
        assert!(find_latest_versioned_file(dir.path(), "nonexistent").is_none());
    }
}
