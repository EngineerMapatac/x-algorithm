use crate::models::candidate::PostCandidate;
use crate::models::query::ScoredPostsQuery;
use rustc_hash::FxHashMap;
use xai_candidate_pipeline::filter::{Filter, FilterResult};

pub struct DropDuplicatesFilter;

impl Filter<ScoredPostsQuery, PostCandidate> for DropDuplicatesFilter {
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
            if let Some(&idx) = kept_index.get(&candidate.tweet_id) {
                kept[idx]
                    .retrieval_sources
                    .append(&mut candidate.retrieval_sources);
                removed.push(candidate);
            } else {
                kept_index.insert(candidate.tweet_id, kept.len());
                kept.push(candidate);
            }
        }

        FilterResult { kept, removed }
    }
}
