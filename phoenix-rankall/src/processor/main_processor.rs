use log::warn;

use super::record::IndexRecord;
use super::thrift_types::{deserialize_binary, PhoenixRankAllObject};
use super::{ProcessorStats, RecordProcessor};

pub struct MainProcessor {
    stats: ProcessorStats,
    warn_limit: u64,
}

impl Default for MainProcessor {
    fn default() -> Self {
        Self {
            stats: ProcessorStats::default(),
            warn_limit: 5,
        }
    }
}

impl MainProcessor {
    pub fn new() -> Self {
        Self::default()
    }
}

impl RecordProcessor for MainProcessor {
    fn process_batch(&mut self, raw: &[Vec<u8>]) -> Vec<IndexRecord> {
        let mut results = Vec::with_capacity(raw.len());

        for payload in raw {
            self.stats.total_processed += 1;

            let obj: PhoenixRankAllObject = match deserialize_binary(payload) {
                Ok(obj) => obj,
                Err(e) => {
                    self.stats.total_deser_error += 1;
                    if self.stats.total_deser_error <= self.warn_limit {
                        warn!(
                            "failed to deserialize PhoenixRankAllObject: {e}, first 50 bytes: {:?}",
                            &payload[..payload.len().min(50)]
                        );
                    }
                    continue;
                }
            };

            let post_id = obj.post_id.unwrap_or(0);
            let author_id = obj.author_id.unwrap_or(0);
            let index_name = obj.index_name.unwrap_or_default();

            if post_id == 0 || author_id == 0 || index_name.is_empty() {
                self.stats.total_invalid += 1;
                continue;
            }

            self.stats.total_success += 1;
            results.push(IndexRecord::Core {
                post_id,
                author_id,
                index_name,
            });
        }

        results
    }

    fn stats(&self) -> &ProcessorStats {
        &self.stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::processor::thrift_types::serialize_binary;

    fn make_thrift_bytes(post_id: i64, author_id: i64, index_name: &str) -> Vec<u8> {
        serialize_binary(&PhoenixRankAllObject::new(
            Some(post_id),
            Some(author_id),
            Some(index_name.to_string()),
            None::<Vec<i64>>,
        ))
        .unwrap()
    }

    #[test]
    fn process_valid_records() {
        let mut proc = MainProcessor::new();
        let raw = vec![
            make_thrift_bytes(100, 10, "1fav"),
            make_thrift_bytes(200, 20, "video"),
            make_thrift_bytes(300, 30, "post_creation"),
        ];

        let results = proc.process_batch(&raw);
        assert_eq!(results.len(), 3);

        match &results[0] {
            IndexRecord::Core {
                post_id,
                author_id,
                index_name,
            } => {
                assert_eq!(*post_id, 100);
                assert_eq!(*author_id, 10);
                assert_eq!(index_name, "1fav");
            }
            _ => panic!("expected Core variant"),
        }

        assert_eq!(proc.stats().total_processed, 3);
        assert_eq!(proc.stats().total_success, 3);
    }

    #[test]
    fn skip_records_with_zero_ids() {
        let mut proc = MainProcessor::new();
        let raw = vec![
            make_thrift_bytes(0, 10, "1fav"),
            make_thrift_bytes(100, 0, "1fav"),
            make_thrift_bytes(100, 10, ""),
            make_thrift_bytes(100, 10, "1fav"),
        ];

        let results = proc.process_batch(&raw);
        assert_eq!(results.len(), 1, "only the valid record should pass");
        assert_eq!(proc.stats().total_invalid, 3);
        assert_eq!(proc.stats().total_success, 1);
    }

    #[test]
    fn handle_corrupt_payload() {
        let mut proc = MainProcessor::new();
        let raw = vec![vec![0xFF, 0xFE, 0xFD], make_thrift_bytes(100, 10, "1fav")];

        let results = proc.process_batch(&raw);
        assert_eq!(results.len(), 1);
        assert_eq!(proc.stats().total_deser_error, 1);
        assert_eq!(proc.stats().total_success, 1);
    }

    #[test]
    fn empty_batch_returns_empty() {
        let mut proc = MainProcessor::new();
        let results = proc.process_batch(&[]);
        assert!(results.is_empty());
        assert_eq!(proc.stats().total_processed, 0);
    }

    #[test]
    fn stats_accumulate_across_batches() {
        let mut proc = MainProcessor::new();
        let batch1 = vec![make_thrift_bytes(100, 10, "1fav")];
        let batch2 = vec![make_thrift_bytes(200, 20, "video")];

        proc.process_batch(&batch1);
        proc.process_batch(&batch2);

        assert_eq!(proc.stats().total_processed, 2);
        assert_eq!(proc.stats().total_success, 2);
    }
}
