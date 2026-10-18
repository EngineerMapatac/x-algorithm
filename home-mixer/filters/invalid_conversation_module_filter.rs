use crate::models::query::ScoredPostsQuery;
use crate::util::conversation_grouping::select_conversation_modules;
use xai_candidate_pipeline::filter::{Filter, FilterResult};
use xai_home_mixer_proto::FeedItem;

pub struct InvalidConversationModuleFilter;

impl Filter<ScoredPostsQuery, FeedItem> for InvalidConversationModuleFilter {
    fn filter(
        &self,
        _query: &ScoredPostsQuery,
        candidates: Vec<FeedItem>,
    ) -> FilterResult<FeedItem> {
        let (kept, removed) = select_conversation_modules(candidates);
        FilterResult { kept, removed }
    }
}
