use crate::models::candidate::{CandidateHelpers, PostCandidate};
use crate::models::query::ScoredPostsQuery;
use crate::params::{EnableSemanticIdHydration, SidServerEndpoint};
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::time::Duration;
use tonic::async_trait;
use xai_candidate_pipeline::component_library::clients::SidClient;
use xai_candidate_pipeline::component_library::utils::{
    build_moka_cache, MokaCache, MokaCacheConfig,
};
use xai_candidate_pipeline::hydrator::{CacheStore, CachedHydrator};

const CACHE_SIZE: u64 = 1_000_000;
const CACHE_TTL: Duration = Duration::from_secs(7200);

fn endpoint_hash(endpoint: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    endpoint.hash(&mut hasher);
    hasher.finish()
}

pub struct SemanticIdHydrator {
    client: Arc<dyn SidClient>,
    cache: MokaCache<(u64, u64), Vec<i32>>,
}

impl SemanticIdHydrator {
    pub fn new(client: Arc<dyn SidClient>) -> Self {
        let cache = build_moka_cache(MokaCacheConfig {
            size: CACHE_SIZE,
            ttl: CACHE_TTL,
        });
        Self { client, cache }
    }
}

#[async_trait]
impl CachedHydrator<ScoredPostsQuery, PostCandidate> for SemanticIdHydrator {
    type CacheKey = (u64, u64);
    type CacheValue = Vec<i32>;

    fn enable(&self, query: &ScoredPostsQuery) -> bool {
        !query.has_cached_posts
            && query.params.get(EnableSemanticIdHydration)
            && !query.params.get(SidServerEndpoint).is_empty()
    }

    fn cache_store(&self) -> &dyn CacheStore<Self::CacheKey, Self::CacheValue> {
        &self.cache
    }

    fn cache_key(&self, candidate: &PostCandidate) -> Self::CacheKey {
        (candidate.get_original_tweet_id(), 0)
    }

    fn cache_key_for(&self, query: &ScoredPostsQuery, candidate: &PostCandidate) -> Self::CacheKey {
        (
            candidate.get_original_tweet_id(),
            endpoint_hash(&query.params.get(SidServerEndpoint)),
        )
    }

    fn cache_value(&self, hydrated: &PostCandidate) -> Self::CacheValue {
        hydrated.semantic_ids.clone().unwrap_or_default()
    }

    fn hydrate_from_cache(&self, value: Self::CacheValue) -> PostCandidate {
        PostCandidate {
            semantic_ids: Some(value),
            ..Default::default()
        }
    }

    async fn hydrate_from_client(
        &self,
        query: &ScoredPostsQuery,
        candidates: &[PostCandidate],
    ) -> Vec<Result<PostCandidate, String>> {
        let endpoint = query.params.get(SidServerEndpoint);

        let mut unique_ids: Vec<i64> = candidates
            .iter()
            .map(|c| c.get_original_tweet_id() as i64)
            .collect();
        unique_ids.sort_unstable();
        unique_ids.dedup();

        let Some(looked_up) = self.client.lookup(&endpoint, &unique_ids).await else {
            return vec![Err("SID lookup failed".to_string()); candidates.len()];
        };

        let codes_by_id: HashMap<i64, Option<Vec<i32>>> =
            unique_ids.into_iter().zip(looked_up).collect();

        candidates
            .iter()
            .map(|candidate| {
                match codes_by_id
                    .get(&(candidate.get_original_tweet_id() as i64))
                    .cloned()
                    .flatten()
                {
                    Some(codes) => Ok(PostCandidate {
                        semantic_ids: Some(codes),
                        ..Default::default()
                    }),
                    None => Err("no SID codes".to_string()),
                }
            })
            .collect()
    }

    fn update(&self, candidate: &mut PostCandidate, hydrated: PostCandidate) {
        candidate.semantic_ids = hydrated.semantic_ids;
    }
}
