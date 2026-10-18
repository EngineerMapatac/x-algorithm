use crate::clients::post_engagers_client::PostEngagersClient;
use crate::models::candidate::{CandidateHelpers, PostCandidate};
use crate::models::query::ScoredPostsQuery;
use crate::params::EnablePostEngagers;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tonic::async_trait;
use xai_candidate_pipeline::component_library::utils::{
    build_moka_cache, MokaCache, MokaCacheConfig,
};
use xai_candidate_pipeline::hydrator::{CacheStore, CachedHydrator};
use xai_proto::engagement_counter::PostEngagers;

const CACHE_SIZE: u64 = 500_000;
const CACHE_TTL: Duration = Duration::from_secs(60);
const LOOKUP_FAILED: &str = "post engagers lookup failed";

pub struct PostEngagersHydrator {
    client: Arc<dyn PostEngagersClient>,
    cache: MokaCache<u64, Option<Arc<PostEngagers>>>,
    no_engagers: Arc<PostEngagers>,
}

impl PostEngagersHydrator {
    pub fn new(client: Arc<dyn PostEngagersClient>) -> Self {
        Self {
            client,
            cache: build_moka_cache(MokaCacheConfig {
                size: CACHE_SIZE,
                ttl: CACHE_TTL,
            }),
            no_engagers: Arc::new(PostEngagers::default()),
        }
    }
}

#[async_trait]
impl CachedHydrator<ScoredPostsQuery, PostCandidate> for PostEngagersHydrator {
    type CacheKey = u64;
    type CacheValue = Option<Arc<PostEngagers>>;

    fn enable(&self, query: &ScoredPostsQuery) -> bool {
        (query.params.get(EnablePostEngagers) || query.is_shadow_traffic) && !query.has_cached_posts
    }

    fn cache_store(&self) -> &dyn CacheStore<Self::CacheKey, Self::CacheValue> {
        &self.cache
    }

    fn cache_key(&self, candidate: &PostCandidate) -> Self::CacheKey {
        candidate.get_original_tweet_id()
    }

    fn cache_value(&self, hydrated: &PostCandidate) -> Self::CacheValue {
        hydrated.post_engagers.clone()
    }

    fn hydrate_from_cache(&self, value: Self::CacheValue) -> PostCandidate {
        PostCandidate {
            post_engagers: value,
            ..Default::default()
        }
    }

    async fn hydrate_from_client(
        &self,
        _query: &ScoredPostsQuery,
        candidates: &[PostCandidate],
    ) -> Vec<Result<PostCandidate, String>> {
        let mut post_ids: Vec<u64> = candidates
            .iter()
            .map(|c| c.get_original_tweet_id())
            .collect();
        post_ids.sort_unstable();
        post_ids.dedup();

        let records: HashMap<u64, Arc<PostEngagers>> = self
            .client
            .get_post_engagers(&post_ids)
            .await
            .into_iter()
            .map(|(post_id, record)| {
                let record = record.map_or_else(|| Arc::clone(&self.no_engagers), Arc::new);
                (post_id, record)
            })
            .collect();

        candidates
            .iter()
            .map(|c| match records.get(&c.get_original_tweet_id()) {
                Some(record) => Ok(PostCandidate {
                    post_engagers: Some(Arc::clone(record)),
                    ..Default::default()
                }),
                None => Err(LOOKUP_FAILED.to_string()),
            })
            .collect()
    }

    fn update(&self, candidate: &mut PostCandidate, hydrated: PostCandidate) {
        candidate.post_engagers = hydrated.post_engagers;
    }
}
