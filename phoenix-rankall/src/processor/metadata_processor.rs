use log::warn;

use super::record::{EngagementCounts, IndexRecord};
use super::thrift_types::{deserialize_binary, PhoenixRankAllCandidateFlat};
use super::{ProcessorStats, RecordProcessor};

pub struct MetadataProcessor {
    stats: ProcessorStats,
    pipeline_type: String,
    warn_limit: u64,
}

impl MetadataProcessor {
    pub fn new(pipeline_type: &str) -> Self {
        Self {
            stats: ProcessorStats::default(),
            pipeline_type: pipeline_type.to_string(),
            warn_limit: 5,
        }
    }
}

impl RecordProcessor for MetadataProcessor {
    fn process_batch(&mut self, raw: &[Vec<u8>]) -> Vec<IndexRecord> {
        let mut results = Vec::with_capacity(raw.len());

        for payload in raw {
            self.stats.total_processed += 1;

            let obj: PhoenixRankAllCandidateFlat = match deserialize_binary(payload) {
                Ok(obj) => obj,
                Err(e) => {
                    self.stats.total_deser_error += 1;
                    if self.stats.total_deser_error <= self.warn_limit {
                        warn!(
                            "failed to deserialize PhoenixRankAllCandidateFlat: {e}, first 50 bytes: {:?}",
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
            if index_name != self.pipeline_type {
                self.stats.total_filtered += 1;
                continue;
            }

            let ec = obj.engagement_count.as_ref();
            let engagement = EngagementCounts {
                retweet_count: ec.and_then(|e| e.retweet_count).unwrap_or(0),
                reply_count: ec.and_then(|e| e.reply_count).unwrap_or(0),
                fav_count: ec.and_then(|e| e.favorite_count).unwrap_or(0),
                quote_count: ec.and_then(|e| e.quote_count).unwrap_or(0),
                bookmark_count: ec.and_then(|e| e.bookmark_count).unwrap_or(0),
                view_count: ec.and_then(|e| e.view_count).unwrap_or(0),
                not_interested_in_count: ec.and_then(|e| e.not_interested_in_count).unwrap_or(0),
                report_count: ec.and_then(|e| e.report_count).unwrap_or(0),
                block_count: ec.and_then(|e| e.block_count).unwrap_or(0),
            };

            self.stats.total_success += 1;
            results.push(IndexRecord::Metadata {
                post_id,
                author_id,
                has_video: obj.has_video.unwrap_or(false),
                has_image: obj.has_image.unwrap_or(false),
                video_duration_ms: obj.video_duration_ms.unwrap_or(0),
                author_followers_count: obj.author_followers_count.unwrap_or(0),
                engagement,
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

    fn make_metadata_bytes(
        post_id: i64,
        author_id: i64,
        index_name: &str,
        has_video: bool,
        fav_count: i64,
    ) -> Vec<u8> {
        let ec = ThriftEngagementCount::new(
            None::<i64>,
            None::<i64>,
            Some(fav_count),
            None::<i64>,
            None::<i64>,
            None::<i64>,
            None::<i64>,
            None::<i64>,
            None::<i64>,
        );
        serialize_binary(&PhoenixRankAllCandidateFlat::new(
            Some(post_id),
            Some(author_id),
            Some(has_video),
            Some(false),
            Some(1000_i64),
            Some(ec),
            if has_video { Some(5000_i64) } else { None },
            None::<MultiModalEmbeddings>,
            Some(index_name.to_string()),
        ))
        .unwrap()
    }

    #[test]
    fn process_valid_metadata() {
        let mut proc = MetadataProcessor::new("metadata");
        let raw = vec![make_metadata_bytes(100, 10, "metadata", true, 50)];

        let results = proc.process_batch(&raw);
        assert_eq!(results.len(), 1);

        match &results[0] {
            IndexRecord::Metadata {
                post_id,
                author_id,
                has_video,
                has_image,
                video_duration_ms,
                author_followers_count,
                engagement,
            } => {
                assert_eq!(*post_id, 100);
                assert_eq!(*author_id, 10);
                assert!(*has_video);
                assert!(!*has_image);
                assert_eq!(*video_duration_ms, 5000);
                assert_eq!(*author_followers_count, 1000);
                assert_eq!(engagement.fav_count, 50);
            }
            _ => panic!("expected Metadata variant"),
        }
    }

    #[test]
    fn filters_wrong_index_name() {
        let mut proc = MetadataProcessor::new("metadata");
        let raw = vec![
            make_metadata_bytes(100, 10, "metadata", false, 10),
            make_metadata_bytes(200, 20, "other", false, 20),
        ];

        let results = proc.process_batch(&raw);
        assert_eq!(results.len(), 1, "only metadata records pass");
        assert_eq!(results[0].post_id(), 100);
        assert_eq!(proc.stats().total_filtered, 1);
    }

    #[test]
    fn skips_zero_ids() {
        let mut proc = MetadataProcessor::new("metadata");
        let raw = vec![
            make_metadata_bytes(0, 10, "metadata", false, 0),
            make_metadata_bytes(100, 10, "metadata", false, 0),
        ];

        let results = proc.process_batch(&raw);
        assert_eq!(results.len(), 1);
        assert_eq!(proc.stats().total_invalid, 1);
    }

    #[test]
    fn defaults_missing_engagement() {
        let mut proc = MetadataProcessor::new("metadata");
        let bytes = serialize_binary(&PhoenixRankAllCandidateFlat::new(
            Some(100_i64),
            Some(10_i64),
            None::<bool>,
            None::<bool>,
            None::<i64>,
            None::<ThriftEngagementCount>,
            None::<i64>,
            None::<MultiModalEmbeddings>,
            Some("metadata".to_string()),
        ))
        .unwrap();

        let results = proc.process_batch(&[bytes]);
        assert_eq!(results.len(), 1);

        match &results[0] {
            IndexRecord::Metadata { engagement, .. } => {
                assert_eq!(engagement.fav_count, 0);
                assert_eq!(engagement.view_count, 0);
            }
            _ => panic!("expected Metadata"),
        }
    }
}
