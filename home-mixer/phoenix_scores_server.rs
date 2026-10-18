use crate::candidate_pipeline::phoenix_scores_pipeline::PhoenixScoresPipeline;
use crate::models::candidate::PostCandidate;
use crate::models::query::ScoredPostsQuery;
use crate::params;
use crate::server::QueryBuilder;
use std::sync::Arc;
use std::time::Instant;
use tonic::Status;
use tracing::info;
use xai_candidate_pipeline::candidate_pipeline::{CandidatePipeline, PipelineResult};
use xai_home_mixer_proto::PhoenixScore;

pub(crate) struct PhoenixScoresOutput {
    pub scores: Vec<PhoenixScore>,
    pub pipeline_result: PipelineResult<ScoredPostsQuery, PostCandidate>,
}

pub struct PhoenixScoresServer {
    pub(crate) phoenix_scores_pipeline: Arc<PhoenixScoresPipeline>,
    pub(crate) query_builder: QueryBuilder,
}

impl PhoenixScoresServer {
    pub fn new(
        phoenix_scores_pipeline: Arc<PhoenixScoresPipeline>,
        query_builder: QueryBuilder,
    ) -> Self {
        Self {
            phoenix_scores_pipeline,
            query_builder,
        }
    }

    pub(crate) async fn run_pipeline(
        &self,
        query: ScoredPostsQuery,
    ) -> Result<PhoenixScoresOutput, Status> {
        if params::TEST_USER_IDS.contains(&query.user_id) {
            return Ok(PhoenixScoresOutput {
                scores: vec![],
                pipeline_result: PipelineResult::empty(),
            });
        }

        let start = Instant::now();

        let pipeline_result = self.phoenix_scores_pipeline.execute(query).await;

        info!(
            "Phoenix Scores response - request_id {} - {} posts ({} ms)",
            pipeline_result.query.request_id,
            pipeline_result.selected_candidates.len(),
            start.elapsed().as_millis()
        );

        let scores = candidates_to_phoenix_scores(&pipeline_result.selected_candidates);

        Ok(PhoenixScoresOutput {
            scores,
            pipeline_result,
        })
    }
}

pub(crate) fn build_query_builder_input(
    proto_query: xai_home_mixer_proto::PhoenixScoresQuery,
) -> xai_home_mixer_proto::ScoredPostsQuery {
    proto_query.query.unwrap_or_default()
}

fn candidates_to_phoenix_scores(candidates: &[PostCandidate]) -> Vec<PhoenixScore> {
    candidates
        .iter()
        .map(|candidate| PhoenixScore {
            tweet_id: candidate.tweet_id,
            score: candidate.score,
            author_id: candidate.author_id,
        })
        .collect()
}
