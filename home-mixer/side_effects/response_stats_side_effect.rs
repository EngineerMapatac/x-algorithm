use crate::models::query::{RequestType, ScoredPostsQuery};
use crate::params::AdsBlenderType;
use crate::util::country_codes::bucket_country;
use crate::util::feed_log::{count_ads, count_posts, feed_name};
use std::sync::Arc;
use tonic::async_trait;
use xai_candidate_pipeline::component_library::utils::client_utils::{
    ClientPlatform, RequestContext,
};
use xai_candidate_pipeline::component_library::utils::client_version;
use xai_candidate_pipeline::side_effect::{SideEffect, SideEffectInput};
use xai_core_entities::entities::SubscriptionLevel;
use xai_home_mixer_proto::FeedItem;
use xai_stats_receiver::global_stats_receiver;

const TYPE_TOTAL: (&str, &str) = ("type", "total");
const TYPE_POSTS: (&str, &str) = ("type", "posts");
const TYPE_ADS: (&str, &str) = ("type", "ads");
const TYPE_EMPTY_ADS: (&str, &str) = ("type", "empty_ads");
const TYPE_EMPTY_POSTS: (&str, &str) = ("type", "empty_posts");
const TYPE_SUFFICIENT_ADS: (&str, &str) = ("type", "sufficient_ads");
const TYPE_SUFFICIENT_POSTS: (&str, &str) = ("type", "sufficient_posts");
const STAGE_RESPONSE: (&str, &str) = ("stage", "response");
const STAGE_FETCHED: (&str, &str) = ("stage", "fetched");

pub struct ResponseStatsSideEffect;

#[async_trait]
impl SideEffect<ScoredPostsQuery, FeedItem> for ResponseStatsSideEffect {
    async fn side_effect(
        &self,
        input: Arc<SideEffectInput<ScoredPostsQuery, FeedItem>>,
    ) -> Result<(), String> {
        let fetched_posts =
            count_posts(&input.selected_candidates) + count_posts(&input.non_selected_candidates);
        let fetched_ads =
            count_ads(&input.selected_candidates) + count_ads(&input.non_selected_candidates);

        let query = &input.query;
        let blender_type = if query.request_type == RequestType::Following {
            "safe_gap".to_string()
        } else {
            query.params.get(AdsBlenderType)
        };
        let feed = feed_name(query.request_type);

        stat_request_context(feed, query);
        stat_response(
            feed,
            &input.selected_candidates,
            fetched_posts,
            fetched_ads,
            &query.country_code,
            query.subscription_level,
            &blender_type,
        );

        Ok(())
    }
}

#[allow(clippy::too_many_arguments)]
fn stat_response(
    feed: &str,
    items: &[FeedItem],
    fetched_posts: usize,
    fetched_ads: usize,
    country_code: &str,
    subscription_level: Option<SubscriptionLevel>,
    blender_type: &str,
) {
    let post_count = count_posts(items);
    let ad_count = count_ads(items);

    let sub_status = subscription_level.map(|s| s.as_str()).unwrap_or("none");

    if let Some(receiver) = global_stats_receiver() {
        let metric = format!("{feed}.response");
        let metric = metric.as_str();
        let sub = ("subscription", sub_status);
        let blender = ("blender", blender_type);
        receiver.incr(metric, &[TYPE_TOTAL, sub], 1);
        receiver.incr(
            metric,
            &[TYPE_TOTAL, sub, ("country", bucket_country(country_code))],
            1,
        );
        receiver.incr(metric, &[TYPE_POSTS, sub, blender], post_count as u64);
        receiver.incr(metric, &[TYPE_ADS, sub, blender], ad_count as u64);

        if ad_count == 0 {
            receiver.incr(metric, &[TYPE_EMPTY_ADS, STAGE_RESPONSE, sub, blender], 1);
        }
        if post_count == 0 {
            receiver.incr(metric, &[TYPE_EMPTY_POSTS, STAGE_RESPONSE, sub, blender], 1);
        }
        if ad_count >= 5 {
            receiver.incr(
                metric,
                &[TYPE_SUFFICIENT_ADS, STAGE_RESPONSE, sub, blender],
                1,
            );
        }
        if post_count >= 20 {
            receiver.incr(
                metric,
                &[TYPE_SUFFICIENT_POSTS, STAGE_RESPONSE, sub, blender],
                1,
            );
        }

        if fetched_ads == 0 {
            receiver.incr(metric, &[TYPE_EMPTY_ADS, STAGE_FETCHED, sub], 1);
        }
        if fetched_posts == 0 {
            receiver.incr(metric, &[TYPE_EMPTY_POSTS, STAGE_FETCHED, sub], 1);
        }
        if fetched_ads >= 5 {
            receiver.incr(metric, &[TYPE_SUFFICIENT_ADS, STAGE_FETCHED, sub], 1);
        }
        if fetched_posts >= 20 {
            receiver.incr(metric, &[TYPE_SUFFICIENT_POSTS, STAGE_FETCHED, sub], 1);
        }
    }
}

fn stat_request_context(feed: &str, query: &ScoredPostsQuery) {
    let Some(receiver) = global_stats_receiver() else {
        return;
    };
    let platform = ClientPlatform::from_app_id(query.client_app_id);
    let client = ("client", platform.stats_client());
    let client_version = (
        "client_version",
        client_version::stats_label(client.1, &query.client_version),
    );
    let polling = if query.is_polling { "true" } else { "false" };
    let request_context = RequestContext::parse(&query.request_context);
    receiver.incr(
        &format!("{feed}.request.polling"),
        &[("polling", polling), client],
        1,
    );
    receiver.incr(
        &format!("{feed}.request.request_context"),
        &[("request_context", request_context.as_ref()), client],
        1,
    );
    receiver.incr(
        &format!("{feed}.request.client"),
        &[client, client_version],
        1,
    );
}
