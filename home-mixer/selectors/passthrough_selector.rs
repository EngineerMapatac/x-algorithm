use crate::models::candidate::PostCandidate;
use crate::models::query::ScoredPostsQuery;
use xai_candidate_pipeline::selector::Selector;

pub struct PassthroughSelector;

impl Selector<ScoredPostsQuery, PostCandidate> for PassthroughSelector {
    fn score(&self, candidate: &PostCandidate) -> f64 {
        candidate.score.unwrap_or(f64::NEG_INFINITY)
    }
}
