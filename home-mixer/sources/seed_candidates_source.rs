use crate::models::candidate::PostCandidate;
use crate::models::query::ScoredPostsQuery;
use tonic::async_trait;
use xai_candidate_pipeline::source::Source;

pub struct SeedCandidatesSource;

#[async_trait]
impl Source<ScoredPostsQuery, PostCandidate> for SeedCandidatesSource {
    fn enable(&self, query: &ScoredPostsQuery) -> bool {
        !query.seed_candidate_post_ids.is_empty()
    }

    async fn source(&self, query: &ScoredPostsQuery) -> Result<Vec<PostCandidate>, String> {
        Ok(query
            .seed_candidate_post_ids
            .iter()
            .map(|&tweet_id| PostCandidate {
                tweet_id,
                ..Default::default()
            })
            .collect())
    }
}
