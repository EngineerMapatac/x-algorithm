use crate::models::candidate::PostCandidate;
use crate::models::query::ScoredPostsQuery;
use xai_candidate_pipeline::filter::{Filter, FilterResult};

pub struct AncestorVideoFilter;

impl Filter<ScoredPostsQuery, PostCandidate> for AncestorVideoFilter {
    fn enable(&self, query: &ScoredPostsQuery) -> bool {
        query.exclude_videos
    }

    fn filter(
        &self,
        _query: &ScoredPostsQuery,
        candidates: Vec<PostCandidate>,
    ) -> FilterResult<PostCandidate> {
        let (removed, kept): (Vec<_>, Vec<_>) = candidates
            .into_iter()
            .partition(|c| c.ancestor_has_video == Some(true));

        FilterResult { kept, removed }
    }
}
