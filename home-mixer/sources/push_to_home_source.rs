use crate::clients::tweet_entity_service_client::TESClient;
use crate::models::query::ScoredPostsQuery;
use std::sync::Arc;
use tonic::async_trait;
use tracing::warn;
use xai_candidate_pipeline::component_library::clients::ReplyMixerClient;
use xai_candidate_pipeline::source::Source;
use xai_home_mixer_proto::{feed_item, FeedItem, PushToHomePost, ServedType};

const MAX_REPLIERS: usize = 3;

pub struct PushToHomeSource {
    pub tes_client: Arc<dyn TESClient + Send + Sync>,
    pub reply_mixer_client: Arc<dyn ReplyMixerClient>,
}

#[async_trait]
impl Source<ScoredPostsQuery, FeedItem> for PushToHomeSource {
    fn enable(&self, query: &ScoredPostsQuery) -> bool {
        query.push_to_home_post_id.is_some()
    }

    async fn source(&self, query: &ScoredPostsQuery) -> Result<Vec<FeedItem>, String> {
        let focal_tweet_id = match query.push_to_home_post_id {
            Some(id) => id,
            None => return Ok(vec![]),
        };

        let core_data = self
            .tes_client
            .get_tweet_core_datas(vec![focal_tweet_id])
            .await;

        let core_data = match core_data.get(&focal_tweet_id) {
            Some(Ok(Some(cd))) => cd,
            Some(Ok(None)) => return Ok(vec![]),
            Some(Err(e)) => {
                return Err(format!(
                    "PushToHomeSource: TES error for tweet {focal_tweet_id}: {e}"
                ));
            }
            None => return Ok(vec![]),
        };

        let is_root = core_data.in_reply_to_tweet_id.is_none();

        let facepile_ids = if is_root {
            self.fetch_top_repliers(query.user_id, focal_tweet_id, core_data.author_id)
                .await
        } else {
            vec![]
        };

        let pth_post = PushToHomePost {
            tweet_id: focal_tweet_id,
            author_id: core_data.author_id,
            in_reply_to_tweet_id: core_data.in_reply_to_tweet_id.unwrap_or(0),
            conversation_id: core_data.conversation_id.unwrap_or(0),
            facepile_user_ids: facepile_ids,
            served_type: ServedType::ForYouPushToHome as i32,
        };

        Ok(vec![FeedItem {
            position: 0,
            item: Some(feed_item::Item::PushToHome(pth_post)),
        }])
    }
}

impl PushToHomeSource {
    async fn fetch_top_repliers(
        &self,
        viewer_id: u64,
        focal_tweet_id: u64,
        author_id: u64,
    ) -> Vec<u64> {
        match self
            .reply_mixer_client
            .get_scored_replies(viewer_id, focal_tweet_id, focal_tweet_id, author_id)
            .await
        {
            Ok(replies) => replies
                .into_iter()
                .filter_map(|r| {
                    r.author_id
                        .map(|id| id as u64)
                        .filter(|&id| id != author_id)
                })
                .take(MAX_REPLIERS)
                .collect(),
            Err(e) => {
                warn!(error = %e, "PushToHomeSource: reply-mixer failed, skipping repliers");
                vec![]
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clients::tweet_entity_service_client::MockTESClient;
    use xai_candidate_pipeline::component_library::clients::MockReplyMixerClient;
    use xai_candidate_pipeline::source::Source;
    use xai_core_entities::entities::PureCoreData;

    fn make_source(mock: MockTESClient) -> PushToHomeSource {
        PushToHomeSource {
            tes_client: Arc::new(mock),
            reply_mixer_client: Arc::new(MockReplyMixerClient),
        }
    }

    #[test]
    fn enable_returns_false_when_no_push_to_home() {
        let source = make_source(MockTESClient::default());
        let query = ScoredPostsQuery::default();
        assert!(!source.enable(&query));
    }

    #[test]
    fn enable_returns_true_when_push_to_home_set() {
        let source = make_source(MockTESClient::default());
        let query = ScoredPostsQuery {
            push_to_home_post_id: Some(123),
            ..Default::default()
        };
        assert!(source.enable(&query));
    }

    #[tokio::test]
    async fn returns_empty_when_tweet_not_found() {
        let source = make_source(MockTESClient::default());
        let query = ScoredPostsQuery {
            push_to_home_post_id: Some(999),
            ..Default::default()
        };
        let result = source.source(&query).await.unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn returns_focal_tweet_when_found() {
        let mut mock = MockTESClient::default();
        mock.core_data.insert(
            42,
            Some(PureCoreData {
                author_id: 100,
                text: "hello".into(),
                in_reply_to_tweet_id: None,
                ..Default::default()
            }),
        );
        let source = make_source(mock);
        let query = ScoredPostsQuery {
            push_to_home_post_id: Some(42),
            ..Default::default()
        };
        let result = source.source(&query).await.unwrap();
        assert_eq!(result.len(), 1);
        match &result[0].item {
            Some(feed_item::Item::PushToHome(post)) => {
                assert_eq!(post.tweet_id, 42);
                assert_eq!(post.author_id, 100);
            }
            _ => panic!("expected PushToHome item"),
        }
    }

    #[tokio::test]
    async fn sets_conversation_fields_for_reply() {
        let mut mock = MockTESClient::default();
        mock.core_data.insert(
            42,
            Some(PureCoreData {
                author_id: 100,
                text: "reply".into(),
                in_reply_to_tweet_id: Some(20),
                conversation_id: Some(10),
                ..Default::default()
            }),
        );
        let source = make_source(mock);
        let query = ScoredPostsQuery {
            push_to_home_post_id: Some(42),
            ..Default::default()
        };
        let result = source.source(&query).await.unwrap();
        match &result[0].item {
            Some(feed_item::Item::PushToHome(post)) => {
                assert_eq!(post.in_reply_to_tweet_id, 20);
                assert_eq!(post.conversation_id, 10);
            }
            _ => panic!("expected PushToHome item"),
        }
    }
}
