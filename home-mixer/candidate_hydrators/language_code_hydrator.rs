use crate::clients::tweet_entity_service_client::TESClient;
use crate::models::candidate::{CandidateHelpers, PostCandidate};
use crate::models::query::ScoredPostsQuery;
use std::sync::Arc;
use tonic::async_trait;
use xai_candidate_pipeline::component_library::utils::{default_quick_cache, QuickCache};
use xai_candidate_pipeline::hydrator::{CacheStore, CachedHydrator};

pub struct LanguageCodeHydrator {
    pub tes_client: Arc<dyn TESClient + Send + Sync>,
    pub cache: QuickCache<u64, Option<String>>,
}

impl LanguageCodeHydrator {
    pub async fn new(tes_client: Arc<dyn TESClient + Send + Sync>) -> Self {
        let cache = default_quick_cache();
        Self { tes_client, cache }
    }
}

#[async_trait]
impl CachedHydrator<ScoredPostsQuery, PostCandidate> for LanguageCodeHydrator {
    type CacheKey = u64;

    type CacheValue = Option<String>;

    fn enable(&self, query: &ScoredPostsQuery) -> bool {
        !query.has_cached_posts
    }

    fn cache_store(&self) -> &dyn CacheStore<Self::CacheKey, Self::CacheValue> {
        &self.cache
    }

    fn cache_key(&self, candidate: &PostCandidate) -> Self::CacheKey {
        candidate.get_original_tweet_id()
    }

    fn cache_value(&self, hydrated: &PostCandidate) -> Self::CacheValue {
        hydrated.language_code.clone()
    }

    fn hydrate_from_cache(&self, value: Self::CacheValue) -> PostCandidate {
        PostCandidate {
            language_code: value,
            ..Default::default()
        }
    }

    async fn hydrate_from_client(
        &self,
        _query: &ScoredPostsQuery,
        candidates: &[PostCandidate],
    ) -> Vec<Result<PostCandidate, String>> {
        let client = &self.tes_client;

        let tweet_ids: Vec<u64> = candidates
            .iter()
            .map(|c| c.get_original_tweet_id())
            .collect();

        let language_results = client.get_language_code(tweet_ids.clone()).await;

        let mut hydrated_candidates = Vec::with_capacity(candidates.len());
        for tweet_id in tweet_ids {
            let result = language_results.get(&tweet_id);
            let hydrated = match result {
                Some(Ok(value)) => Ok(PostCandidate {
                    language_code: value.clone(),
                    ..Default::default()
                }),
                None => Err(format!("Missing language_code for tweet_id={}", tweet_id)),
                Some(Err(err)) => Err(err.to_string()),
            };
            hydrated_candidates.push(hydrated);
        }

        hydrated_candidates
    }

    fn update(&self, candidate: &mut PostCandidate, hydrated: PostCandidate) {
        candidate.language_code = hydrated.language_code;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clients::tweet_entity_service_client::MockTESClient;
    use std::collections::HashMap;
    use xai_candidate_pipeline::hydrator::Hydrator;

    #[tokio::test]
    async fn test_language_code_found() {
        let mut language_code = HashMap::new();
        language_code.insert(1001u64, Some("en".to_string()));

        let client = Arc::new(MockTESClient {
            language_code,
            ..Default::default()
        });
        let hydrator = LanguageCodeHydrator::new(client as Arc<dyn TESClient + Send + Sync>).await;

        let candidates = vec![PostCandidate {
            tweet_id: 1001,
            ..Default::default()
        }];

        let result = hydrator
            .hydrate(&ScoredPostsQuery::default(), &candidates)
            .await;

        assert_eq!(result.len(), 1);
        assert!(result[0].is_ok());
        assert_eq!(
            result[0].as_ref().unwrap().language_code,
            Some("en".to_string())
        );
    }

    #[tokio::test]
    async fn test_language_code_none() {
        let mut language_code = HashMap::new();
        language_code.insert(1002u64, None);

        let client = Arc::new(MockTESClient {
            language_code,
            ..Default::default()
        });
        let hydrator = LanguageCodeHydrator::new(client as Arc<dyn TESClient + Send + Sync>).await;

        let candidates = vec![PostCandidate {
            tweet_id: 1002,
            ..Default::default()
        }];

        let result = hydrator
            .hydrate(&ScoredPostsQuery::default(), &candidates)
            .await;

        assert_eq!(result.len(), 1);
        assert!(result[0].is_ok());
        assert_eq!(result[0].as_ref().unwrap().language_code, None);
    }

    #[tokio::test]
    async fn test_language_code_caching() {
        let mut language_code = HashMap::new();
        language_code.insert(1003u64, Some("ja".to_string()));

        let client = Arc::new(MockTESClient {
            language_code,
            ..Default::default()
        });
        let hydrator = LanguageCodeHydrator::new(client as Arc<dyn TESClient + Send + Sync>).await;

        let candidates = vec![PostCandidate {
            tweet_id: 1003,
            ..Default::default()
        }];

        let first = hydrator
            .hydrate(&ScoredPostsQuery::default(), &candidates)
            .await;
        assert_eq!(
            first[0].as_ref().unwrap().language_code,
            Some("ja".to_string())
        );

        let second = hydrator
            .hydrate(&ScoredPostsQuery::default(), &candidates)
            .await;
        assert_eq!(
            second[0].as_ref().unwrap().language_code,
            Some("ja".to_string())
        );
    }

    #[tokio::test]
    async fn test_language_code_multiple_candidates() {
        let mut language_code = HashMap::new();
        language_code.insert(2001u64, Some("en".to_string()));
        language_code.insert(2002u64, Some("fr".to_string()));
        language_code.insert(2003u64, None);

        let client = Arc::new(MockTESClient {
            language_code,
            ..Default::default()
        });
        let hydrator = LanguageCodeHydrator::new(client as Arc<dyn TESClient + Send + Sync>).await;

        let candidates = vec![
            PostCandidate {
                tweet_id: 2001,
                ..Default::default()
            },
            PostCandidate {
                tweet_id: 2002,
                ..Default::default()
            },
            PostCandidate {
                tweet_id: 2003,
                ..Default::default()
            },
        ];

        let result = hydrator
            .hydrate(&ScoredPostsQuery::default(), &candidates)
            .await;

        assert_eq!(result.len(), 3);
        assert_eq!(
            result[0].as_ref().unwrap().language_code,
            Some("en".to_string())
        );
        assert_eq!(
            result[1].as_ref().unwrap().language_code,
            Some("fr".to_string())
        );
        assert_eq!(result[2].as_ref().unwrap().language_code, None);
    }
}
