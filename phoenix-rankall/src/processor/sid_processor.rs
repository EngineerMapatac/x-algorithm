use std::collections::HashSet;
use std::sync::Arc;

use log::warn;

use super::record::IndexRecord;
use super::thrift_types::{deserialize_binary, PhoenixRankAllObject};
use super::{ProcessorStats, RecordProcessor};
use crate::sid::client::SidClient;

pub struct SidProcessor {
    stats: ProcessorStats,
    warn_limit: u64,
    index_filters: HashSet<String>,
    sid_client: Arc<SidClient>,
    batch_post_ids: Vec<i64>,
    runtime: tokio::runtime::Handle,
}

impl SidProcessor {
    pub fn new(
        index_filters: impl IntoIterator<Item = impl Into<String>>,
        sid_client: Arc<SidClient>,
        runtime: tokio::runtime::Handle,
    ) -> Self {
        Self {
            stats: ProcessorStats::default(),
            warn_limit: 5,
            index_filters: index_filters.into_iter().map(Into::into).collect(),
            sid_client,
            batch_post_ids: Vec::new(),
            runtime,
        }
    }
}

impl RecordProcessor for SidProcessor {
    fn process_batch(&mut self, raw: &[Vec<u8>]) -> Vec<IndexRecord> {
        let mut staged: Vec<(i64, i64, String)> = Vec::with_capacity(raw.len());
        self.batch_post_ids.clear();

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

            if !self.index_filters.contains(&index_name) {
                self.stats.total_filtered += 1;
                continue;
            }

            self.batch_post_ids.push(post_id);
            staged.push((post_id, author_id, index_name));
        }

        if staged.is_empty() {
            return Vec::new();
        }

        let sid_map = {
            let client = Arc::clone(&self.sid_client);
            let ids = self.batch_post_ids.clone();
            self.runtime
                .block_on(async move { client.lookup_sids(&ids, None).await })
        };

        let mut results = Vec::with_capacity(staged.len());
        for (post_id, author_id, index_name) in staged {
            let post_sid = sid_map.get(&post_id).cloned().unwrap_or_default();
            self.stats.total_success += 1;
            results.push(IndexRecord::Sid {
                post_id,
                author_id,
                index_name,
                post_sid,
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
    use crate::sid::metrics::SidMetrics;
    use std::time::Duration;

    fn make_thrift_bytes(post_id: i64, author_id: i64, index_name: &str) -> Vec<u8> {
        serialize_binary(&PhoenixRankAllObject::new(
            Some(post_id),
            Some(author_id),
            Some(index_name.to_string()),
            None::<Vec<i64>>,
        ))
        .unwrap()
    }

    const TEST_FILTERS: [&str; 4] = ["1fav", "video", "nsfw_video", "evergreen_video"];

    fn make_processor() -> SidProcessor {
        let client = Arc::new(
            SidClient::new(
                "127.0.0.1:1",
                0.0,
                Duration::from_millis(50),
                SidMetrics::for_tests(),
            )
            .unwrap(),
        );
        SidProcessor::new(TEST_FILTERS, client, tokio::runtime::Handle::current())
    }

    async fn process_in_blocking<F>(f: F) -> Vec<IndexRecord>
    where
        F: FnOnce() -> Vec<IndexRecord> + Send + 'static,
    {
        tokio::task::spawn_blocking(f).await.unwrap()
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn filters_records_by_index_name() {
        let mut proc = make_processor();
        let raw = vec![
            make_thrift_bytes(100, 10, "1fav"),
            make_thrift_bytes(200, 20, "video"),
            make_thrift_bytes(300, 30, "nsfw_video"),
            make_thrift_bytes(400, 40, "evergreen_video"),
            make_thrift_bytes(500, 50, "32fav"),
            make_thrift_bytes(600, 60, "topic"),
        ];
        let results = process_in_blocking(move || proc.process_batch(&raw)).await;
        let names: Vec<String> = results
            .iter()
            .filter_map(|r| match r {
                IndexRecord::Sid { index_name, .. } => Some(index_name.clone()),
                _ => None,
            })
            .collect();
        assert_eq!(names.len(), 4, "configured names kept, others dropped");
        for n in &TEST_FILTERS {
            assert!(names.contains(&n.to_string()), "missing {n}");
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn invalid_records_dropped_before_filter() {
        let mut proc = make_processor();
        let raw = vec![
            make_thrift_bytes(0, 10, "1fav"),
            make_thrift_bytes(100, 0, "1fav"),
            make_thrift_bytes(100, 10, ""),
        ];
        let results = process_in_blocking(move || proc.process_batch(&raw)).await;
        assert!(results.is_empty());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn rpc_failure_yields_empty_post_sid_not_drop() {
        let mut proc = make_processor();
        let raw = vec![make_thrift_bytes(100, 10, "1fav")];
        let results = process_in_blocking(move || proc.process_batch(&raw)).await;
        assert_eq!(results.len(), 1);
        match &results[0] {
            IndexRecord::Sid {
                post_id,
                author_id,
                post_sid,
                ..
            } => {
                assert_eq!(*post_id, 100);
                assert_eq!(*author_id, 10);
                assert!(
                    post_sid.is_empty(),
                    "RPC fails -> empty post_sid (backfill will retry)"
                );
            }
            other => panic!("expected Sid variant, got {other:?}"),
        }
    }
}
