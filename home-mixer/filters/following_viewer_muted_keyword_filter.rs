use crate::models::candidate::PostCandidate;
use crate::models::query::ScoredPostsQuery;
use std::sync::Arc;
use xai_candidate_pipeline::filter::{Filter, FilterResult};
use xai_post_text::{MatchTweetGroup, TokenSequence, TweetTokenizer, UserMutes};

pub struct FollowingViewerMutedKeywordFilter {
    pub tokenizer: Arc<TweetTokenizer>,
}

impl FollowingViewerMutedKeywordFilter {
    pub fn new() -> Self {
        Self {
            tokenizer: Arc::new(TweetTokenizer::new()),
        }
    }
}

impl Filter<ScoredPostsQuery, PostCandidate> for FollowingViewerMutedKeywordFilter {
    fn filter(
        &self,
        query: &ScoredPostsQuery,
        candidates: Vec<PostCandidate>,
    ) -> FilterResult<PostCandidate> {
        let muted_keywords = &query.user_features.muted_keywords;
        if muted_keywords.is_empty() {
            return FilterResult {
                kept: candidates,
                removed: vec![],
            };
        }

        let tokenizer = Arc::clone(&self.tokenizer);
        tokio::task::block_in_place(|| {
            let token_sequences: Vec<TokenSequence> = muted_keywords
                .iter()
                .map(|k| tokenizer.tokenize(k))
                .collect();
            let matcher = MatchTweetGroup::new(UserMutes::new(token_sequences));

            let mut kept = Vec::new();
            let mut removed = Vec::new();
            for candidate in candidates {
                if candidate_matches(&candidate, &tokenizer, &matcher) {
                    removed.push(candidate);
                } else {
                    kept.push(candidate);
                }
            }
            FilterResult { kept, removed }
        })
    }
}

fn candidate_matches(
    candidate: &PostCandidate,
    tokenizer: &TweetTokenizer,
    matcher: &MatchTweetGroup,
) -> bool {
    std::iter::once(candidate.tweet_text.as_str())
        .chain(candidate.quoted_tweet_text.as_deref())
        .chain(candidate.ancestor_texts.values().map(String::as_str))
        .filter(|text| !text.is_empty())
        .any(|text| matcher.matches(&tokenizer.tokenize(text)))
}
