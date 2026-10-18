use crate::models::query::ScoredPostsQuery;
use xai_candidate_pipeline::filter::{Filter, FilterResult};
use xai_home_mixer_proto::{feed_item, FeedItem};

pub struct PushToHomeDedupFilter;

impl Filter<ScoredPostsQuery, FeedItem> for PushToHomeDedupFilter {
    fn enable(&self, query: &ScoredPostsQuery) -> bool {
        query.push_to_home_post_id.is_some()
    }

    fn filter(
        &self,
        query: &ScoredPostsQuery,
        candidates: Vec<FeedItem>,
    ) -> FilterResult<FeedItem> {
        let push_to_home_id = query.push_to_home_post_id.unwrap_or(u64::MAX);
        let (kept, removed) = candidates.into_iter().partition(|item| match &item.item {
            Some(feed_item::Item::Post(post)) => {
                post.tweet_id != push_to_home_id && post.retweeted_tweet_id != push_to_home_id
            }
            _ => true,
        });

        FilterResult { kept, removed }
    }
}
