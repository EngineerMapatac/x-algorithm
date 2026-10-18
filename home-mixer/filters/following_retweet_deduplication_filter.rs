use crate::models::candidate::PostCandidate;
use crate::models::query::ScoredPostsQuery;
use std::collections::HashSet;
use xai_candidate_pipeline::filter::{Filter, FilterResult};

pub struct FollowingRetweetDeduplicationFilter;

impl Filter<ScoredPostsQuery, PostCandidate> for FollowingRetweetDeduplicationFilter {
    fn filter(
        &self,
        _query: &ScoredPostsQuery,
        candidates: Vec<PostCandidate>,
    ) -> FilterResult<PostCandidate> {
        let (retweets, native_tweets): (Vec<_>, Vec<_>) = candidates
            .iter()
            .cloned()
            .partition(|c| c.retweeted_tweet_id.is_some());

        let mut seen_tweet_ids: HashSet<u64> = HashSet::with_capacity(candidates.len());
        let mut kept_ids: HashSet<u64> = HashSet::with_capacity(candidates.len());
        for native in &native_tweets {
            seen_tweet_ids.insert(native.tweet_id);
            kept_ids.insert(native.tweet_id);
        }

        for retweet in &retweets {
            let source_id = retweet.retweeted_tweet_id.expect("partitioned as retweet");
            let ids = [retweet.tweet_id, source_id];
            if ids.iter().any(|id| seen_tweet_ids.contains(id)) {
                continue;
            }
            seen_tweet_ids.extend(ids);
            kept_ids.insert(retweet.tweet_id);
        }

        let mut kept = Vec::with_capacity(kept_ids.len());
        let mut removed = Vec::new();
        for candidate in candidates {
            if kept_ids.contains(&candidate.tweet_id) {
                kept.push(candidate);
            } else {
                removed.push(candidate);
            }
        }

        FilterResult { kept, removed }
    }
}
