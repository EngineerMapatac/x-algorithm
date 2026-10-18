use crate::models::candidate::{CandidateHelpers, PostCandidate};
use crate::models::query::ScoredPostsQuery;
use crate::params::VideoCarouselImmersiveClusterId;
use crate::util::egress::PredictionDispatch;
use crate::util::phoenix_request::build_immersive_prediction_request;
use crate::util::video_carousel;
use tonic::async_trait;
use xai_candidate_pipeline::component_library::clients::phoenix_prediction_client::PhoenixCluster;
use xai_candidate_pipeline::scorer::Scorer;

pub struct ImmersiveVideoScorer {
    pub dispatch: PredictionDispatch,
}

#[async_trait]
impl Scorer<ScoredPostsQuery, PostCandidate> for ImmersiveVideoScorer {
    fn enable(&self, query: &ScoredPostsQuery) -> bool {
        !query.has_cached_posts
            && video_carousel::is_enabled(query)
            && video_carousel::ranks_by_immersive_scores(query)
            && query.immersive_columnar_scoring_sequence.is_some()
    }

    async fn score(
        &self,
        query: &ScoredPostsQuery,
        candidates: &[PostCandidate],
    ) -> Vec<Result<PostCandidate, String>> {
        let videos: Vec<&PostCandidate> = candidates
            .iter()
            .filter(|candidate| video_carousel::is_carousel_video(candidate))
            .collect();
        if videos.is_empty() {
            return vec![Ok(PostCandidate::default()); candidates.len()];
        }

        let cluster = PhoenixCluster::parse(&query.params.get(VideoCarouselImmersiveClusterId));
        let request = build_immersive_prediction_request(query, &videos);
        let predictions = match self
            .dispatch
            .predict_with_fallback(query, cluster, request)
            .await
        {
            Ok(predictions) => predictions,
            Err(err) => {
                return vec![Err(format!("Immersive prediction failed: {err}")); candidates.len()];
            }
        };

        candidates
            .iter()
            .map(|candidate| {
                Ok(PostCandidate {
                    immersive_scores: video_carousel::is_carousel_video(candidate)
                        .then(|| predictions.candidate_scores(&candidate.get_original_tweet_id())),
                    ..Default::default()
                })
            })
            .collect()
    }

    fn update(&self, candidate: &mut PostCandidate, scored: PostCandidate) {
        candidate.immersive_scores = scored.immersive_scores;
    }
}
