use crate::models::query::ScoredPostsQuery;
use crate::params::{EnableFeedSurvey, FEED_SURVEY_POSITION};
use tonic::async_trait;
use xai_candidate_pipeline::source::Source;
use xai_home_mixer_proto::{feed_item, FeedItem, FeedSurvey};

pub struct FeedSurveySource;

#[async_trait]
impl Source<ScoredPostsQuery, FeedItem> for FeedSurveySource {
    fn enable(&self, query: &ScoredPostsQuery) -> bool {
        query.params.get(EnableFeedSurvey) && query.feed_survey_eligible
    }

    async fn source(&self, _query: &ScoredPostsQuery) -> Result<Vec<FeedItem>, String> {
        Ok(vec![FeedItem {
            position: FEED_SURVEY_POSITION as i32,
            item: Some(feed_item::Item::FeedSurvey(FeedSurvey {})),
        }])
    }
}
