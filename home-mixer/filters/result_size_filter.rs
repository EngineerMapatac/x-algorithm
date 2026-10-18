use crate::models::candidate::PostCandidate;
use crate::models::query::ScoredPostsQuery;
use crate::params::PhoenixScoresResultSize;
use xai_candidate_pipeline::filter::{Filter, FilterResult};

pub struct ResultSizeFilter;

impl Filter<ScoredPostsQuery, PostCandidate> for ResultSizeFilter {
    fn filter(
        &self,
        query: &ScoredPostsQuery,
        mut candidates: Vec<PostCandidate>,
    ) -> FilterResult<PostCandidate> {
        let limit = query.params.get(PhoenixScoresResultSize) as usize;
        let removed = candidates.split_off(limit.min(candidates.len()));
        FilterResult {
            kept: candidates,
            removed,
        }
    }
}
