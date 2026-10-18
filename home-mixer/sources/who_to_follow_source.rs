use crate::clients::who_to_follow_client::WhoToFollowClient;
use crate::models::query::{RequestType, ScoredPostsQuery};
use crate::params::EnableWhoToFollowModule;
use std::sync::Arc;
use tonic::async_trait;
use xai_account_recommendations_mixer_proto::{
    product_context, AccountRecommendationsMixerRequest, ClientContext,
    HomeReverseChronWhoToFollowProductContext, HomeWhoToFollowProductContext, Product,
    ProductContext, WhoToFollowReactiveContext,
};
use xai_candidate_pipeline::source::Source;
use xai_home_mixer_proto::{feed_item, FeedItem, WhoToFollowModule};
use xai_x_thrift::served_history::EntityIdType;

const EXCLUDED_USER_IDS_LIMIT: usize = 200;
const MAX_WHO_TO_FOLLOW_USERS: usize = 3;

pub struct WhoToFollowSource {
    pub who_to_follow_client: Arc<dyn WhoToFollowClient + Send + Sync>,
}

#[async_trait]
impl Source<ScoredPostsQuery, FeedItem> for WhoToFollowSource {
    fn enable(&self, query: &ScoredPostsQuery) -> bool {
        query.params.get(EnableWhoToFollowModule) && query.who_to_follow_eligible
    }

    async fn source(&self, query: &ScoredPostsQuery) -> Result<Vec<FeedItem>, String> {
        let request = build_wtf_request(query);

        let mut response = self
            .who_to_follow_client
            .get_wtf_recommendations(request)
            .await
            .map_err(|e| format!("WhoToFollowSource: {e}"))?;

        if response.user_recommendations.is_empty() {
            return Ok(vec![]);
        }

        response
            .user_recommendations
            .truncate(MAX_WHO_TO_FOLLOW_USERS);

        let module = WhoToFollowModule {
            who_to_follow_response: Some(response),
        };

        Ok(vec![FeedItem {
            position: 0,
            item: Some(feed_item::Item::WhoToFollow(module)),
        }])
    }
}

fn build_wtf_request(query: &ScoredPostsQuery) -> AccountRecommendationsMixerRequest {
    let excluded_user_ids = get_excluded_user_ids(query);
    let reactive = WhoToFollowReactiveContext {
        excluded_user_ids,
        followed_user_id: None,
        dismissed_user_id: None,
    };

    let (product, product_context) = match query.request_type {
        RequestType::Following => (
            Product::HomeReverseChronWhoToFollow,
            ProductContext {
                context: Some(
                    product_context::Context::HomeReverseChronWhoToFollowProductContext(
                        HomeReverseChronWhoToFollowProductContext {
                            wtf_reactive_context: Some(reactive),
                        },
                    ),
                ),
            },
        ),
        _ => (
            Product::HomeWhoToFollow,
            ProductContext {
                context: Some(product_context::Context::HomeWhoToFollowProductContext(
                    HomeWhoToFollowProductContext {
                        wtf_reactive_context: Some(reactive),
                    },
                )),
            },
        ),
    };

    AccountRecommendationsMixerRequest {
        client_context: Some(ClientContext {
            user_id: Some(query.user_id as i64),
            app_id: Some(query.client_app_id as i64),
            country_code: Some(query.country_code.clone()),
            language_code: Some(query.language_code.clone()),
            ip_address: Some(query.ip_address.clone()),
            user_agent: Some(query.user_agent.clone()),
            ..Default::default()
        }),
        product: product as i32,
        debug_params: None,
        cursor: None,
        product_context: Some(product_context),
    }
}

fn get_excluded_user_ids(query: &ScoredPostsQuery) -> Vec<i64> {
    query
        .served_history
        .iter()
        .flat_map(|sh| &sh.entries)
        .filter(|entry| entry.entity_type == EntityIdType::WHO_TO_FOLLOW)
        .flat_map(|entry| {
            entry
                .item_ids
                .iter()
                .flatten()
                .filter_map(|item| item.user_id)
        })
        .take(EXCLUDED_USER_IDS_LIMIT)
        .collect()
}
