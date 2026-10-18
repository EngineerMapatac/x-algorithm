use crate::models::query::ImpressionBloomFilterEntry;
use crate::models::query::ScoredPostsQuery;
use std::sync::Arc;
use tonic::async_trait;
use xai_candidate_pipeline::component_library::clients::ImpressionBloomFilterClient;
use xai_candidate_pipeline::query_hydrator::QueryHydrator;
use xai_x_thrift::impression_bloom_filter::SurfaceArea;

pub struct ImpressionBloomFilterQueryHydrator {
    pub client: Arc<dyn ImpressionBloomFilterClient>,
}

#[async_trait]
impl QueryHydrator<ScoredPostsQuery> for ImpressionBloomFilterQueryHydrator {
    fn enable(&self, query: &ScoredPostsQuery) -> bool {
        query.bloom_filter_entries.is_empty()
    }

    async fn hydrate(&self, query: &ScoredPostsQuery) -> Result<ScoredPostsQuery, String> {
        let user_id = query.user_id as i64;

        let bloom_filter_thrift = self
            .client
            .get(user_id, SurfaceArea::HOME_TIMELINE)
            .await
            .map_err(|e| e.to_string())?;

        let entries: Vec<ImpressionBloomFilterEntry> = bloom_filter_thrift
            .and_then(|s| s.entries)
            .unwrap_or_default()
            .iter()
            .map(|e| thrift_entry_to_proto(e))
            .collect();

        Ok(ScoredPostsQuery {
            bloom_filter_entries: entries,
            ..Default::default()
        })
    }

    fn update(&self, query: &mut ScoredPostsQuery, hydrated: ScoredPostsQuery) {
        query.bloom_filter_entries = hydrated.bloom_filter_entries;
    }
}

fn thrift_entry_to_proto(
    entry: &xai_x_thrift::impression_bloom_filter::ImpressionBloomFilterEntry,
) -> ImpressionBloomFilterEntry {
    ImpressionBloomFilterEntry {
        bloom_filter: entry.bloom_filter.iter().map(|&v| v as u64).collect(),
        size_cap: entry.size_cap,
        false_positive_rate: entry.false_positive_rate.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use xai_candidate_pipeline::component_library::clients::MockImpressionBloomFilterClient;
    use xai_x_thrift::impression_bloom_filter::{
        ImpressionBloomFilterEntry as ThriftEntry, ImpressionBloomFilterSeq,
    };

    #[test]
    fn test_thrift_entry_to_proto_conversion() {
        use thrift::OrderedFloat;

        let thrift_entry = ThriftEntry::new(
            vec![100i64, 200, 300],
            1000,
            2000,
            5,
            50,
            SurfaceArea::HOME_TIMELINE,
            OrderedFloat::from(0.01),
        );

        let proto_entry = thrift_entry_to_proto(&thrift_entry);
        assert_eq!(proto_entry.bloom_filter, vec![100u64, 200, 300]);
        assert_eq!(proto_entry.size_cap, 50);
        assert!((proto_entry.false_positive_rate - 0.01).abs() < f64::EPSILON);
    }

    #[tokio::test]
    async fn test_hydrate_populates_bloom_filter_entries() {
        use thrift::OrderedFloat;

        let entry = ThriftEntry::new(
            vec![1i64, 2, 3],
            1000,
            2000,
            5,
            100,
            SurfaceArea::HOME_TIMELINE,
            OrderedFloat::from(0.01),
        );
        let seq = ImpressionBloomFilterSeq::new(Some(vec![Box::new(entry)]));

        let mut mock = MockImpressionBloomFilterClient::default();
        mock.responses
            .insert((12345, i32::from(SurfaceArea::HOME_TIMELINE)), seq);

        let hydrator = ImpressionBloomFilterQueryHydrator {
            client: Arc::new(mock),
        };

        let query = ScoredPostsQuery {
            user_id: 12345,
            ..Default::default()
        };

        let result = hydrator.hydrate(&query).await.unwrap();
        assert_eq!(result.bloom_filter_entries.len(), 1);
        assert_eq!(result.bloom_filter_entries[0].bloom_filter, vec![1, 2, 3]);
        assert_eq!(result.bloom_filter_entries[0].size_cap, 100);
    }

    #[tokio::test]
    async fn test_hydrate_cache_miss_returns_empty() {
        let mock = MockImpressionBloomFilterClient::default();
        let hydrator = ImpressionBloomFilterQueryHydrator {
            client: Arc::new(mock),
        };

        let query = ScoredPostsQuery {
            user_id: 99999,
            ..Default::default()
        };

        let result = hydrator.hydrate(&query).await.unwrap();
        assert!(result.bloom_filter_entries.is_empty());
    }

    #[tokio::test]
    async fn test_update_overwrites_field() {
        let mock = MockImpressionBloomFilterClient::default();
        let hydrator = ImpressionBloomFilterQueryHydrator {
            client: Arc::new(mock),
        };

        let mut query = ScoredPostsQuery {
            user_id: 1,
            bloom_filter_entries: vec![ImpressionBloomFilterEntry {
                bloom_filter: vec![999],
                size_cap: 10,
                false_positive_rate: 0.5,
            }],
            ..Default::default()
        };

        let hydrated = ScoredPostsQuery {
            bloom_filter_entries: vec![ImpressionBloomFilterEntry {
                bloom_filter: vec![1, 2, 3],
                size_cap: 100,
                false_positive_rate: 0.01,
            }],
            ..Default::default()
        };

        hydrator.update(&mut query, hydrated);
        assert_eq!(query.bloom_filter_entries.len(), 1);
        assert_eq!(query.bloom_filter_entries[0].bloom_filter, vec![1, 2, 3]);
    }

    #[test]
    fn test_enable_dedup_guard() {
        let hydrator = ImpressionBloomFilterQueryHydrator {
            client: Arc::new(MockImpressionBloomFilterClient::default()),
        };

        assert!(hydrator.enable(&ScoredPostsQuery::default()));

        let populated = ScoredPostsQuery {
            bloom_filter_entries: vec![ImpressionBloomFilterEntry {
                bloom_filter: vec![1],
                size_cap: 10,
                false_positive_rate: 0.1,
            }],
            ..Default::default()
        };
        assert!(!hydrator.enable(&populated));
    }
}
