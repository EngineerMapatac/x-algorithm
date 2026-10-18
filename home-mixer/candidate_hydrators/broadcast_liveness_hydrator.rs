use crate::clients::tweet_entity_service_client::TESClient;
use crate::models::candidate::{CandidateHelpers, PostCandidate};
use crate::models::query::ScoredPostsQuery;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;
use tonic::async_trait;
use xai_candidate_pipeline::component_library::clients::StratoClient;
use xai_candidate_pipeline::component_library::utils::{
    build_moka_cache, default_quick_cache, MokaCache, MokaCacheConfig, QuickCache,
};
use xai_candidate_pipeline::hydrator::{CacheStore, Hydrator};
use xai_core_entities::entities::UrlEntities;
use xai_stats_receiver::global_stats_receiver;

const LIVENESS_CACHE_TTL: Duration = Duration::from_secs(5 * 60);
const LIVENESS_CACHE_SIZE: u64 = 100_000;

pub struct BroadcastLivenessHydrator {
    pub tes_client: Arc<dyn TESClient + Send + Sync>,
    pub strato_client: Arc<dyn StratoClient + Send + Sync>,
    pub broadcast_id_cache: QuickCache<u64, Option<String>>,
    pub liveness_cache: MokaCache<String, bool>,
}

impl BroadcastLivenessHydrator {
    pub fn new(
        tes_client: Arc<dyn TESClient + Send + Sync>,
        strato_client: Arc<dyn StratoClient + Send + Sync>,
    ) -> Self {
        Self {
            tes_client,
            strato_client,
            broadcast_id_cache: default_quick_cache(),
            liveness_cache: build_moka_cache(MokaCacheConfig {
                size: LIVENESS_CACHE_SIZE,
                ttl: LIVENESS_CACHE_TTL,
            }),
        }
    }

    fn emit_cache_stats(&self, cache: &str, cache_hits: usize, cache_misses: usize) {
        if let Some(receiver) = global_stats_receiver() {
            let metric_name = format!("{}.{}", self.name(), cache);
            if cache_hits > 0 {
                receiver.incr(
                    metric_name.as_str(),
                    &[("requests", "cache_hit")],
                    cache_hits as u64,
                );
            }
            if cache_misses > 0 {
                receiver.incr(
                    metric_name.as_str(),
                    &[("requests", "cache_miss")],
                    cache_misses as u64,
                );
            }
        }
    }
}

fn extract_broadcast_id(url: &str) -> Option<String> {
    const MARKER: &str = "/i/broadcasts/";
    let idx = url.find(MARKER)?;
    let id: String = url[idx + MARKER.len()..]
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric())
        .collect();
    (!id.is_empty()).then_some(id)
}

fn broadcast_id_from_urls(urls: Option<&UrlEntities>) -> Option<String> {
    urls?
        .iter()
        .find_map(|u| u.expanded.as_deref().and_then(extract_broadcast_id))
}

#[async_trait]
impl Hydrator<ScoredPostsQuery, PostCandidate> for BroadcastLivenessHydrator {
    fn enable(&self, query: &ScoredPostsQuery) -> bool {
        false
    }

    async fn hydrate(
        &self,
        query: &ScoredPostsQuery,
        candidates: &[PostCandidate],
    ) -> Vec<Result<PostCandidate, String>> {
        let in_network_only = true;
        let followed: HashSet<u64> = if in_network_only {
            query
                .user_features
                .followed_user_ids
                .iter()
                .map(|&id| id as u64)
                .collect()
        } else {
            HashSet::new()
        };

        let mut broadcast_id_by_index: Vec<Option<String>> = vec![None; candidates.len()];
        let mut cache_hits = 0usize;
        let mut cache_misses = 0usize;
        let mut miss_indices: Vec<usize> = Vec::new();
        let mut miss_tweet_ids: Vec<u64> = Vec::new();

        for (i, candidate) in candidates.iter().enumerate() {
            if in_network_only
                && candidate.author_id != query.user_id
                && !followed.contains(&candidate.author_id)
            {
                continue;
            }
            let key = candidate.get_original_tweet_id();
            match self.broadcast_id_cache.get(&key).await {
                Some(broadcast_id) => {
                    broadcast_id_by_index[i] = broadcast_id;
                    cache_hits += 1;
                }
                None => {
                    miss_indices.push(i);
                    miss_tweet_ids.push(key);
                    cache_misses += 1;
                }
            }
        }
        self.emit_cache_stats("cache", cache_hits, cache_misses);

        if !miss_tweet_ids.is_empty() {
            let urls_map = self.tes_client.get_urls(miss_tweet_ids.clone()).await;
            for (&index, tweet_id) in miss_indices.iter().zip(miss_tweet_ids.iter()) {
                if let Some(Ok(urls)) = urls_map.get(tweet_id) {
                    let broadcast_id = broadcast_id_from_urls(urls.as_ref());
                    self.broadcast_id_cache
                        .insert(*tweet_id, broadcast_id.clone())
                        .await;
                    broadcast_id_by_index[index] = broadcast_id;
                }
            }
        }

        let unique_ids: Vec<String> = broadcast_id_by_index
            .iter()
            .flatten()
            .cloned()
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();

        let mut live_by_id: HashMap<String, bool> = HashMap::new();
        let mut liveness_misses: Vec<String> = Vec::new();
        for id in unique_ids {
            match self.liveness_cache.get(&id).await {
                Some(is_live) => {
                    live_by_id.insert(id, is_live);
                }
                None => liveness_misses.push(id),
            }
        }
        self.emit_cache_stats("liveness_cache", live_by_id.len(), liveness_misses.len());

        if !liveness_misses.is_empty() {
            let fetched = self
                .strato_client
                .batch_get_broadcast_is_live(liveness_misses)
                .await;
            for (id, is_live) in fetched {
                self.liveness_cache.insert(id.clone(), is_live).await;
                live_by_id.insert(id, is_live);
            }
        }

        broadcast_id_by_index
            .iter()
            .map(|broadcast_id| {
                let broadcast_is_live = broadcast_id
                    .as_ref()
                    .map(|id| live_by_id.get(id).copied().unwrap_or(false));
                Ok(PostCandidate {
                    broadcast_is_live,
                    ..Default::default()
                })
            })
            .collect()
    }

    fn update(&self, candidate: &mut PostCandidate, hydrated: PostCandidate) {
        candidate.broadcast_is_live = hydrated.broadcast_is_live;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clients::tweet_entity_service_client::MockTESClient;
    use std::collections::HashMap;
    use xai_candidate_pipeline::component_library::clients::MockStratoClient;
    use xai_core_entities::entities::UrlEntity;

    fn broadcast_url(id: &str) -> UrlEntity {
        UrlEntity {
            expanded: Some(format!("https://x.com/i/broadcasts/{id}")),
        }
    }

    fn query() -> ScoredPostsQuery {
        ScoredPostsQuery {
            user_id: 1,
            user_features: crate::models::user_features::UserFeatures {
                followed_user_ids: vec![42],
                ..Default::default()
            },
            ..Default::default()
        }
    }

    fn candidate(tweet_id: u64) -> PostCandidate {
        PostCandidate {
            tweet_id,
            author_id: 42,
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn in_network_live_broadcast_is_live() {
        let tes = Arc::new(MockTESClient {
            urls: HashMap::from([(100u64, Some(vec![broadcast_url("BID1")]))]),
            ..Default::default()
        });
        let strato = MockStratoClient {
            broadcast_is_live: HashMap::from([("BID1".to_string(), true)]),
            ..Default::default()
        };
        let h = BroadcastLivenessHydrator::new(tes.clone(), Arc::new(strato));
        let c = [candidate(100)];

        let out = h.hydrate(&query(), &c).await;
        assert_eq!(out[0].as_ref().unwrap().broadcast_is_live, Some(true));

        h.hydrate(&query(), &c).await;
        assert_eq!(tes.call_count(), 1);
    }
}
