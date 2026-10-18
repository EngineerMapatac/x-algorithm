use crate::clients::impressed_posts_client::ImpressedPostsClient;
use crate::models::query::ScoredPostsQuery;
use std::sync::Arc;
use tonic::async_trait;
use xai_candidate_pipeline::query_hydrator::QueryHydrator;

pub struct ImpressedPostsQueryHydrator {
    pub client: Arc<dyn ImpressedPostsClient>,
}

#[async_trait]
impl QueryHydrator<ScoredPostsQuery> for ImpressedPostsQueryHydrator {
    async fn hydrate(&self, query: &ScoredPostsQuery) -> Result<ScoredPostsQuery, String> {
        let impressed_post_ids = self.client.get(query.user_id).await?;

        Ok(ScoredPostsQuery {
            impressed_post_ids,
            ..Default::default()
        })
    }

    fn update(&self, query: &mut ScoredPostsQuery, hydrated: ScoredPostsQuery) {
        query.impressed_post_ids = hydrated.impressed_post_ids;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clients::impressed_posts_client::MockImpressedPostsClient;

    #[tokio::test]
    async fn test_hydrate_populates_impressed_post_ids() {
        let mock = MockImpressedPostsClient {
            post_ids: vec![100, 200, 300],
        };

        let hydrator = ImpressedPostsQueryHydrator {
            client: Arc::new(mock),
        };

        let query = ScoredPostsQuery {
            user_id: 12345,
            ..Default::default()
        };

        let result = hydrator.hydrate(&query).await.unwrap();
        assert_eq!(result.impressed_post_ids, vec![100, 200, 300]);
    }

    #[tokio::test]
    async fn test_hydrate_cache_miss_returns_empty() {
        let mock = MockImpressedPostsClient::default();
        let hydrator = ImpressedPostsQueryHydrator {
            client: Arc::new(mock),
        };

        let query = ScoredPostsQuery {
            user_id: 99999,
            ..Default::default()
        };

        let result = hydrator.hydrate(&query).await.unwrap();
        assert!(result.impressed_post_ids.is_empty());
    }

    #[tokio::test]
    async fn test_update_overwrites_field() {
        let mock = MockImpressedPostsClient::default();
        let hydrator = ImpressedPostsQueryHydrator {
            client: Arc::new(mock),
        };

        let mut query = ScoredPostsQuery {
            user_id: 1,
            impressed_post_ids: vec![999],
            ..Default::default()
        };

        let hydrated = ScoredPostsQuery {
            impressed_post_ids: vec![1, 2, 3],
            ..Default::default()
        };

        hydrator.update(&mut query, hydrated);
        assert_eq!(query.impressed_post_ids, vec![1, 2, 3]);
    }
}
