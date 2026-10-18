use crate::models::query::{RequestType, ScoredPostsQuery};
use tracing::info;
use xai_candidate_pipeline::candidate_pipeline::PipelineResult;
use xai_home_mixer_proto::{feed_item, FeedItem};

pub fn feed_name(request_type: RequestType) -> &'static str {
    match request_type {
        RequestType::RankedFollowing => "RankedFollowingFeed",
        RequestType::Following => "FollowingFeed",
        _ => "ForYouFeed",
    }
}

pub fn count_posts(items: &[FeedItem]) -> usize {
    items
        .iter()
        .filter(|i| matches!(i.item, Some(feed_item::Item::Post(_))))
        .count()
}

pub fn count_ads(items: &[FeedItem]) -> usize {
    items
        .iter()
        .filter(|i| matches!(i.item, Some(feed_item::Item::Ad(_))))
        .count()
}

pub fn log_feed_request(query: &ScoredPostsQuery) {
    let feed = feed_name(query.request_type);
    info!(
        request_id = query.request_id,
        user_id = query.user_id,
        app_id = query.client_app_id,
        request_context = %query.request_context,
        cursor = ?query.cursor,
        seen_ids = query.seen_ids.len(),
        "{feed} request -"
    );
}

pub fn log_feed_response(result: &PipelineResult<ScoredPostsQuery, FeedItem>) {
    let query = &result.query;
    let feed = feed_name(query.request_type);
    let post_count = count_posts(&result.selected_candidates);
    let ad_count = count_ads(&result.selected_candidates);
    let fetched_posts = count_posts(&result.retrieved_candidates)
        .saturating_sub(count_posts(&result.filtered_candidates));
    let fetched_ads = count_ads(&result.retrieved_candidates)
        .saturating_sub(count_ads(&result.filtered_candidates));
    let country_code = &query.country_code;
    let sub_status = query
        .subscription_level
        .map(|s| s.as_str())
        .unwrap_or("none");
    info!(
        "{feed} response - {post_count} posts, {ad_count} ads \
         (fetched {fetched_posts} posts, {fetched_ads} ads), \
         country {country_code}, subscription {sub_status}.",
    );
}
