use log::warn;

use super::blacklist::Blacklist;
use super::record::IndexRecord;
use super::thrift_types::{deserialize_binary, PhoenixRankAllObject};
use super::{ProcessorStats, RecordProcessor};

pub struct TopicProcessor {
    stats: ProcessorStats,
    blacklist: Blacklist,
    warn_limit: u64,
    blacklist_log_count: u64,
}

impl TopicProcessor {
    pub fn new(blacklist: Blacklist) -> Self {
        Self {
            stats: ProcessorStats::default(),
            blacklist,
            warn_limit: 5,
            blacklist_log_count: 0,
        }
    }
}

impl RecordProcessor for TopicProcessor {
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

            if self.blacklist.is_blocked(post_id, author_id) {
                self.stats.total_filtered += 1;
                self.blacklist_log_count += 1;
                if self.blacklist_log_count <= 10 {
                    warn!(
                        "blacklist filtered: post_id={post_id}, author_id={author_id}, index_name={index_name}",
                    );
                }
                continue;
            }

            let topic_entity_ids = obj.topic_entity_ids.unwrap_or_default();

            self.stats.total_success += 1;
            results.push(IndexRecord::Topic {
                post_id,
                author_id,
                index_name,
                topic_entity_ids,
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
    use crate::processor::thrift_types::{serialize_binary, PhoenixRankAllObject};

    fn make_thrift_bytes(
        post_id: i64,
        author_id: i64,
        index_name: &str,
        topics: Option<Vec<i64>>,
    ) -> Vec<u8> {
        serialize_binary(&PhoenixRankAllObject::new(
            Some(post_id),
            Some(author_id),
            Some(index_name.to_string()),
            topics,
        ))
        .unwrap()
    }

    #[test]
    fn process_valid_with_topics() {
        let mut proc = TopicProcessor::new(Blacklist::default());
        let raw = vec![make_thrift_bytes(
            100,
            10,
            "1fav_topic",
            Some(vec![1925949634972626944, 1925949452197478400]),
        )];

        let results = proc.process_batch(&raw);
        assert_eq!(results.len(), 1);

        match &results[0] {
            IndexRecord::Topic {
                post_id,
                author_id,
                index_name,
                topic_entity_ids,
            } => {
                assert_eq!(*post_id, 100);
                assert_eq!(*author_id, 10);
                assert_eq!(index_name, "1fav_topic");
                assert_eq!(topic_entity_ids.len(), 2);
                assert_eq!(topic_entity_ids[0], 1925949634972626944);
            }
            _ => panic!("expected Topic variant"),
        }
    }

    #[test]
    fn process_without_topics_defaults_to_empty() {
        let mut proc = TopicProcessor::new(Blacklist::default());
        let raw = vec![make_thrift_bytes(100, 10, "1fav", None)];

        let results = proc.process_batch(&raw);
        assert_eq!(results.len(), 1);

        match &results[0] {
            IndexRecord::Topic {
                topic_entity_ids, ..
            } => {
                assert!(topic_entity_ids.is_empty());
            }
            _ => panic!("expected Topic variant"),
        }
    }

    #[test]
    fn blacklist_filters_blocked_posts() {
        let mut bl = Blacklist::default();
        bl.blocked_post_ids.insert(100);

        let mut proc = TopicProcessor::new(bl);
        let raw = vec![
            make_thrift_bytes(100, 10, "1fav", None),
            make_thrift_bytes(200, 20, "1fav", None),
        ];

        let results = proc.process_batch(&raw);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].post_id(), 200);
        assert_eq!(proc.stats().total_filtered, 1);
    }

    #[test]
    fn blacklist_filters_blocked_authors() {
        let mut bl = Blacklist::default();
        bl.blocked_author_ids.insert(10);

        let mut proc = TopicProcessor::new(bl);
        let raw = vec![
            make_thrift_bytes(100, 10, "1fav", None),
            make_thrift_bytes(200, 20, "1fav", None),
        ];

        let results = proc.process_batch(&raw);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].post_id(), 200);
    }
}
