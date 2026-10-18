use crate::candidate_pipeline::following_candidate_pipeline::FollowingCandidatePipeline;
use crate::models::query::{FollowingPaginationMeta, ScoredPostsQuery};
use crate::params;
use crate::server::QueryBuilder;
use crate::util::feed_log;
use crate::util::urt::reverse_chron_following::make_urt_timeline;
use tonic::Status;
use xai_candidate_pipeline::candidate_pipeline::CandidatePipeline;
use xai_home_mixer_proto::FeedItem;

pub(crate) struct FollowingFeedOutput {
    pub items: Vec<FeedItem>,
    pub pagination_meta: FollowingPaginationMeta,
}

pub struct FollowingFeedServer {
    pub(crate) query_builder: QueryBuilder,
    pipeline: FollowingCandidatePipeline,
}

impl FollowingFeedServer {
    pub fn new(query_builder: QueryBuilder, pipeline: FollowingCandidatePipeline) -> Self {
        Self {
            query_builder,
            pipeline,
        }
    }

    pub(crate) async fn get_following_feed(
        &self,
        query: ScoredPostsQuery,
    ) -> Result<FollowingFeedOutput, Status> {
        if params::TEST_USER_IDS.contains(&query.user_id) {
            return Ok(FollowingFeedOutput {
                items: vec![],
                pagination_meta: FollowingPaginationMeta::default(),
            });
        }

        let result = self.pipeline.execute(query).await;
        feed_log::log_feed_response(&result);

        let pagination_meta = result
            .query
            .following_pagination_meta
            .get()
            .cloned()
            .unwrap_or_default();

        Ok(FollowingFeedOutput {
            items: result.selected_candidates,
            pagination_meta,
        })
    }

    pub(crate) async fn get_following_feed_urt(
        &self,
        query: ScoredPostsQuery,
    ) -> Result<Vec<u8>, Status> {
        feed_log::log_feed_request(&query);

        let cursor = query.cursor.clone();
        let request_context = query.request_context.clone();
        let client_app_id = query.client_app_id;
        let viewer_id = query.user_id;
        let language_code = query.language_code.clone();
        let country_code = query.country_code.clone();

        let output = self.get_following_feed(query).await?;

        let timeline_response = make_urt_timeline(
            &output.items,
            cursor.as_ref(),
            &request_context,
            client_app_id,
            viewer_id,
            &language_code,
            if country_code.is_empty() {
                None
            } else {
                Some(&country_code)
            },
            &output.pagination_meta,
        );

        xai_urt_thrift::serialize_binary(&timeline_response)
            .map_err(|e| Status::internal(format!("failed to serialize URT: {e}")))
    }
}
