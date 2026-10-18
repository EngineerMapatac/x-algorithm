use crate::models::candidate::PostCandidate;
use crate::models::query::ScoredPostsQuery;
use crate::params;
use std::sync::Arc;
use tonic::async_trait;
use tracing::info;
use xai_candidate_pipeline::side_effect::{SideEffect, SideEffectInput};
use xai_stats_receiver::global_stats_receiver;

const CANDIDATES_METRIC: &str = "DebugSideEffect.candidates";

pub struct DebugSideEffect;

#[async_trait]
impl SideEffect<ScoredPostsQuery, PostCandidate> for DebugSideEffect {
    async fn side_effect(
        &self,
        input: Arc<SideEffectInput<ScoredPostsQuery, PostCandidate>>,
    ) -> Result<(), String> {
        let candidates = &input.selected_candidates;

        if let Some(receiver) = global_stats_receiver() {
            receiver.incr(CANDIDATES_METRIC, &[], candidates.len() as u64);
        }

        if params::TRACE_USER_IDS.contains(&input.query.user_id) {
            let posts: Vec<String> = candidates
                .iter()
                .map(|c| format!("post={} author={}", c.tweet_id, c.author_id))
                .collect();
            info!(
                "debug_side_effect: viewer={} [{}]",
                input.query.user_id,
                posts.join(", ")
            );
        }

        Ok(())
    }
}
