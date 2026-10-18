use crate::models::query::ScoredPostsQuery;
use crate::params::EnableContextFeatures;
use std::sync::Arc;
use std::time::Duration;
use tonic::async_trait;
use xai_candidate_pipeline::component_library::clients::followed_grok_topics_store_client::FollowedGrokTopicsStoreClient;

use xai_candidate_pipeline::query_hydrator::QueryHydrator;
use xai_recsys_proto::grok_topics::{ids_to_bool_array, PARENT_TOPIC_IDS};

const MH_GET_TIMEOUT: Duration = Duration::from_millis(300);
pub struct FollowedGrokTopicsQueryHydrator {
    client: Arc<dyn FollowedGrokTopicsStoreClient>,
}

impl FollowedGrokTopicsQueryHydrator {
    pub fn new(client: Arc<dyn FollowedGrokTopicsStoreClient>) -> Self {
        Self { client }
    }
}

#[async_trait]
impl QueryHydrator<ScoredPostsQuery> for FollowedGrokTopicsQueryHydrator {
    fn enable(&self, query: &ScoredPostsQuery) -> bool {
        query.params.get(EnableContextFeatures) || query.is_shadow_traffic
    }

    async fn hydrate(&self, query: &ScoredPostsQuery) -> Result<ScoredPostsQuery, String> {
        let ids = tokio::time::timeout(MH_GET_TIMEOUT, self.client.fetch(query.user_id))
            .await
            .map_err(|_| "FollowedGrokTopics MH get timed out".to_string())??;

        let topics = ids.map(|ids| ids_to_bool_array(&ids, &PARENT_TOPIC_IDS));

        Ok(ScoredPostsQuery {
            followed_grok_topics: topics,
            ..Default::default()
        })
    }

    fn update(&self, query: &mut ScoredPostsQuery, hydrated: ScoredPostsQuery) {
        query.followed_grok_topics = hydrated.followed_grok_topics;
    }
}
