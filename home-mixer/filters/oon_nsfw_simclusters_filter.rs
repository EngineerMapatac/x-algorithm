use crate::models::candidate::PostCandidate;
use crate::models::query::ScoredPostsQuery;
use crate::params::EnableSimclustersSource;
use xai_candidate_pipeline::filter::{Filter, FilterResult};
use xai_home_mixer_proto::ServedType;

pub struct OONNsfwSimclustersFilter;

impl Filter<ScoredPostsQuery, PostCandidate> for OONNsfwSimclustersFilter {
    fn enable(&self, query: &ScoredPostsQuery) -> bool {
        query.params.get(EnableSimclustersSource)
    }

    fn filter(
        &self,
        _query: &ScoredPostsQuery,
        candidates: Vec<PostCandidate>,
    ) -> FilterResult<PostCandidate> {
        let (removed, kept): (Vec<_>, Vec<_>) = candidates.into_iter().partition(|c| {
            c.served_type == Some(ServedType::ForYouSimclusters)
                && c.in_network == Some(false)
                && c.nsfw_author == Some(true)
        });

        FilterResult { kept, removed }
    }
}
