use std::sync::Arc;

use log::warn;

use super::record::IndexRecord;
use super::thrift_types::{deserialize_binary, PhoenixRankAllCandidateFlat};
use super::{ProcessorStats, RecordProcessor};
use crate::sid::client::SidClient;

pub const TAIL_INDEX_NAME: &str = "tail";

pub struct SidTailProcessor {
    stats: ProcessorStats,
    warn_limit: u64,
    max_author_followers: i64,
    min_fav_count: i64,
    sid_client: Arc<SidClient>,
    batch_post_ids: Vec<i64>,
    runtime: tokio::runtime::Handle,
}

impl SidTailProcessor {
    pub fn new(
        max_author_followers: i64,
        min_fav_count: i64,
        sid_client: Arc<SidClient>,
        runtime: tokio::runtime::Handle,
    ) -> Self {
        Self {
            stats: ProcessorStats::default(),
            warn_limit: 5,
            max_author_followers,
            min_fav_count,
            sid_client,
            batch_post_ids: Vec::new(),
            runtime,
        }
    }
}

impl RecordProcessor for SidTailProcessor {
    fn process_batch(&mut self, raw: &[Vec<u8>]) -> Vec<IndexRecord> {
        let mut staged: Vec<(i64, i64)> = Vec::with_capacity(raw.len());
        self.batch_post_ids.clear();

        for payload in raw {
            self.stats.total_processed += 1;

            let obj: PhoenixRankAllCandidateFlat = match deserialize_binary(payload) {
                Ok(obj) => obj,
                Err(e) => {
                    self.stats.total_deser_error += 1;
                    if self.stats.total_deser_error <= self.warn_limit {
                        warn!(
                            "sid-tail: failed to deserialize PhoenixRankAllCandidateFlat: {e}, \
                             first 50 bytes: {:?}",
                            &payload[..payload.len().min(50)]
                        );
                    }
                    continue;
                }
            };

            let post_id = obj.post_id.unwrap_or(0);
            let author_id = obj.author_id.unwrap_or(0);
            if post_id == 0 || author_id == 0 {
                self.stats.total_invalid += 1;
                continue;
            }

            let index_name = obj.index_name.as_deref().unwrap_or("metadata");
            if index_name != "metadata" {
                self.stats.total_filtered += 1;
                continue;
            }

            let followers = obj.author_followers_count.unwrap_or(0);
            if followers >= self.max_author_followers {
                self.stats.total_filtered += 1;
                continue;
            }

            if self.min_fav_count > 0 {
                let fav = obj
                    .engagement_count
                    .as_ref()
                    .and_then(|e| e.favorite_count)
                    .unwrap_or(0);
                if fav < self.min_fav_count {
                    self.stats.total_filtered += 1;
                    continue;
                }
            }

            self.batch_post_ids.push(post_id);
            staged.push((post_id, author_id));
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
        for (post_id, author_id) in staged {
            let post_sid = sid_map.get(&post_id).cloned().unwrap_or_default();
            self.stats.total_success += 1;
            results.push(IndexRecord::Sid {
                post_id,
                author_id,
                index_name: TAIL_INDEX_NAME.to_string(),
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
    use crate::processor::thrift_types::{
        serialize_binary, EngagementCount as ThriftEngagementCount, MultiModalEmbeddings,
        PhoenixRankAllCandidateFlat,
    };
    use crate::sid::metrics::SidMetrics;
    use std::time::Duration;

    fn make_bytes(
        post_id: i64,
        author_id: i64,
        index_name: Option<&str>,
        followers: i64,
        fav: Option<i64>,
    ) -> Vec<u8> {
        let ec = fav.map(|f| {
            ThriftEngagementCount::new(
                None::<i64>,
                None::<i64>,
                Some(f),
                None::<i64>,
                None::<i64>,
                None::<i64>,
                None::<i64>,
                None::<i64>,
                None::<i64>,
            )
        });
        serialize_binary(&PhoenixRankAllCandidateFlat::new(
            Some(post_id),
            Some(author_id),
            Some(false),
            Some(false),
            Some(followers),
            ec,
            None::<i64>,
            None::<MultiModalEmbeddings>,
            index_name.map(|s| s.to_string()),
        ))
        .unwrap()
    }

    fn make_processor(max_followers: i64, min_fav: i64) -> SidTailProcessor {
        let client = Arc::new(
            SidClient::new(
                "127.0.0.1:1",
                0.0,
                Duration::from_millis(50),
                SidMetrics::for_tests(),
            )
            .unwrap(),
        );
        SidTailProcessor::new(
            max_followers,
            min_fav,
            client,
            tokio::runtime::Handle::current(),
        )
    }

    async fn process_in_blocking<F>(f: F) -> Vec<IndexRecord>
    where
        F: FnOnce() -> Vec<IndexRecord> + Send + 'static,
    {
        tokio::task::spawn_blocking(f).await.unwrap()
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn keeps_tail_metadata_rows() {
        let mut proc = make_processor(1000, 0);
        let batch = vec![
            make_bytes(1, 10, None, 50, Some(3)),
            make_bytes(2, 20, Some("metadata"), 999, None),
            make_bytes(3, 30, Some("metadata"), 1000, None),
            make_bytes(4, 40, Some("other"), 10, Some(5)),
            make_bytes(5, 50, Some("metadata"), 5_000_000, Some(10)),
        ];
        let out = process_in_blocking(move || proc.process_batch(&batch)).await;
        let ids: Vec<i64> = out.iter().map(|r| r.post_id()).collect();
        assert_eq!(ids, vec![1, 2]);
        for r in &out {
            match r {
                IndexRecord::Sid { index_name, .. } => assert_eq!(index_name, TAIL_INDEX_NAME),
                _ => panic!("expected Sid record"),
            }
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn min_fav_filter() {
        let mut proc = make_processor(1000, 1);
        let batch = vec![
            make_bytes(1, 10, Some("metadata"), 10, Some(0)),
            make_bytes(2, 20, Some("metadata"), 10, None),
            make_bytes(3, 30, Some("metadata"), 10, Some(1)),
        ];
        let out = process_in_blocking(move || proc.process_batch(&batch)).await;
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].post_id(), 3);
    }
}
