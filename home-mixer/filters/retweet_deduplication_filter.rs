use crate::models::candidate::PostCandidate;
use crate::models::query::ScoredPostsQuery;
use rustc_hash::FxHashMap;
use xai_candidate_pipeline::filter::{Filter, FilterResult};

pub struct RetweetDeduplicationFilter;

impl Filter<ScoredPostsQuery, PostCandidate> for RetweetDeduplicationFilter {
    fn filter(
        &self,
        _query: &ScoredPostsQuery,
        candidates: Vec<PostCandidate>,
    ) -> FilterResult<PostCandidate> {
        let mut kept_index: FxHashMap<u64, usize> =
            FxHashMap::with_capacity_and_hasher(candidates.len(), Default::default());
        let mut kept: Vec<PostCandidate> = Vec::with_capacity(candidates.len());
        let mut removed = Vec::new();

        for mut candidate in candidates {
            let dedup_id = candidate.retweeted_tweet_id.unwrap_or(candidate.tweet_id);
            if let Some(&idx) = kept_index.get(&dedup_id) {
                kept[idx]
                    .retrieval_sources
                    .append(&mut candidate.retrieval_sources);
                removed.push(candidate);
            } else {
                kept_index.insert(dedup_id, kept.len());
                kept.push(candidate);
            }
        }

        FilterResult { kept, removed }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_candidate(tweet_id: u64, retweeted_tweet_id: Option<u64>) -> PostCandidate {
        PostCandidate {
            tweet_id,
            retweeted_tweet_id,
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn test_multiple_retweets_keeps_first() {
        let filter = RetweetDeduplicationFilter;
        let query = ScoredPostsQuery::default();

        let candidates = vec![
            make_candidate(1, Some(100)),
            make_candidate(2, Some(100)),
            make_candidate(3, Some(100)),
        ];

        let result = filter.filter(&query, candidates);

        assert_eq!(result.kept.len(), 1);
        assert_eq!(result.kept[0].tweet_id, 1);
        assert_eq!(result.removed.len(), 2);
    }

    #[tokio::test]
    async fn test_original_before_retweets_removes_retweets() {
        let filter = RetweetDeduplicationFilter;
        let query = ScoredPostsQuery::default();

        let candidates = vec![
            make_candidate(100, None),
            make_candidate(1, Some(100)),
            make_candidate(2, Some(100)),
        ];

        let result = filter.filter(&query, candidates);

        assert_eq!(result.kept.len(), 1);
        assert_eq!(result.kept[0].tweet_id, 100);
        assert_eq!(result.removed.len(), 2);
    }

    #[tokio::test]
    async fn test_retweet_before_original_removes_original() {
        let filter = RetweetDeduplicationFilter;
        let query = ScoredPostsQuery::default();

        let candidates = vec![
            make_candidate(1, Some(100)),
            make_candidate(100, None),
            make_candidate(2, Some(100)),
        ];

        let result = filter.filter(&query, candidates);

        assert_eq!(result.kept.len(), 1);
        assert_eq!(result.kept[0].tweet_id, 1);
        assert_eq!(result.removed.len(), 2);
    }

    #[tokio::test]
    async fn test_non_retweets_always_kept() {
        let filter = RetweetDeduplicationFilter;
        let query = ScoredPostsQuery::default();

        let candidates = vec![
            make_candidate(1, None),
            make_candidate(2, None),
            make_candidate(3, None),
        ];

        let result = filter.filter(&query, candidates);

        assert_eq!(result.kept.len(), 3);
        assert_eq!(result.removed.len(), 0);
    }

    #[tokio::test]
    async fn test_mixed_retweets_of_different_tweets() {
        let filter = RetweetDeduplicationFilter;
        let query = ScoredPostsQuery::default();

        let candidates = vec![
            make_candidate(1, Some(100)),
            make_candidate(2, Some(200)),
            make_candidate(3, Some(100)),
            make_candidate(4, Some(200)),
            make_candidate(5, Some(300)),
        ];

        let result = filter.filter(&query, candidates);

        assert_eq!(result.kept.len(), 3);
        assert_eq!(result.kept[0].tweet_id, 1);
        assert_eq!(result.kept[1].tweet_id, 2);
        assert_eq!(result.kept[2].tweet_id, 5);
        assert_eq!(result.removed.len(), 2);
    }

    #[tokio::test]
    async fn test_retweet_before_original_deduplicates() {
        let filter = RetweetDeduplicationFilter;
        let query = ScoredPostsQuery::default();

        let candidates = vec![make_candidate(1, Some(100)), make_candidate(100, None)];

        let result = filter.filter(&query, candidates);

        assert_eq!(
            result.kept.len(),
            1,
            "expected only one candidate to survive dedup"
        );
        assert_eq!(result.removed.len(), 1);
    }
}
