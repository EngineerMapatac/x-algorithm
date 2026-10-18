use crate::clients::user_action_aggregation_client::{
    UserActionAggregationClient, UserActionsCluster,
};
use crate::models::query::ScoredPostsQuery;
use crate::params::{
    self as p, MaxSeqLengthScoring, UasSourceDataType, UseXdsForUas, UserActionsClusterId,
    VideoCarouselImmersiveAggregationType,
};
use crate::util::video_carousel;
use std::sync::Arc;
use tonic::async_trait;
use xai_candidate_pipeline::query_hydrator::QueryHydrator;
use xai_recsys_proto::{
    ResponseFormat, UaasModelType, UserActionAggregationType, UserActionSequenceSourceDataType,
};

pub struct ImmersiveScoringSequenceQueryHydrator {
    pub user_action_aggregation_client: Arc<dyn UserActionAggregationClient + Send + Sync>,
}

impl ImmersiveScoringSequenceQueryHydrator {
    pub fn new(
        user_action_aggregation_client: Arc<dyn UserActionAggregationClient + Send + Sync>,
    ) -> Self {
        Self {
            user_action_aggregation_client,
        }
    }
}

#[async_trait]
impl QueryHydrator<ScoredPostsQuery> for ImmersiveScoringSequenceQueryHydrator {
    fn enable(&self, query: &ScoredPostsQuery) -> bool {
        video_carousel::ranks_by_immersive_scores(query)
    }

    async fn hydrate(&self, query: &ScoredPostsQuery) -> Result<ScoredPostsQuery, String> {
        let aggregation_type = UserActionAggregationType::from_str_name(
            &query.params.get(VideoCarouselImmersiveAggregationType),
        )
        .ok_or("unknown immersive aggregation type")?;
        let result = self
            .user_action_aggregation_client
            .fetch_aggregated_sequence(
                UserActionsCluster::parse(&query.params.get(UserActionsClusterId)),
                query.user_id,
                p::UAS_WINDOW_TIME_MS as u32,
                query.params.get(MaxSeqLengthScoring),
                aggregation_type,
                UserActionSequenceSourceDataType::from_str_name(
                    &query.params.get(UasSourceDataType),
                )
                .unwrap_or(UserActionSequenceSourceDataType::Arrow),
                ResponseFormat::Arrow,
                None,
                query.params.get(UseXdsForUas),
                UaasModelType::Ranking,
            )
            .await
            .map_err(|e| format!("Aggregation service call failed: {}", e))?;

        Ok(ScoredPostsQuery {
            immersive_columnar_scoring_sequence: Some(
                result
                    .columnar_bytes
                    .ok_or("no columnar immersive sequence")?,
            ),
            ..Default::default()
        })
    }

    fn update(&self, query: &mut ScoredPostsQuery, hydrated: ScoredPostsQuery) {
        query.immersive_columnar_scoring_sequence = hydrated.immersive_columnar_scoring_sequence;
    }
}
