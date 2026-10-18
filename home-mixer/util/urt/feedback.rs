use crate::util::url::percent_encode;
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use std::collections::{BTreeMap, HashMap};
use std::hash::{Hash, Hasher};
use xai_home_mixer_proto::ScoredPost;
use xai_urt_thrift::feedback::{FeedbackEngagementType, FeedbackEntity, FeedbackMetadata};
use xai_urt_thrift::icon::HorizonIcon;
use xai_urt_thrift::metadata::{ClientEventInfo, FeedbackAction, FeedbackInfo, FeedbackType};
use xai_urt_thrift::serialize_compact;

pub(super) const FEEDBACK_TTL_MS: i64 = 30 * 24 * 60 * 60 * 1000;

mod keys {
    pub const DONT_LIKE: &str = "Feedback.dontLike";
    pub const DONT_LIKE_CONFIRMATION: &str = "Feedback.dontLikeConfirmation";
    pub const NOT_RELEVANT: &str = "Feedback.notRelevant";
    pub const NOT_RELEVANT_CONFIRMATION: &str = "Feedback.notRelevantConfirmation";
    pub const SHOW_FEWER_TWEETS: &str = "Feedback.showFewerTweets";
    pub const SHOW_FEWER_TWEETS_CONFIRMATION: &str = "Feedback.showFewerTweetsConfirmation";
    pub const SHOW_FEWER_RETWEETS: &str = "Feedback.showFewerRetweets";
    pub const SHOW_FEWER_RETWEETS_CONFIRMATION: &str = "Feedback.showFewerRetweetsConfirmation";
    pub const NOT_INTERESTED_IN_TOPIC: &str = "Feedback.notInterestedInTopic";
    pub const NOT_INTERESTED_IN_TOPIC_CONFIRMATION: &str =
        "Feedback.notInterestedInTopicConfirmation";
}

pub(super) struct PostFeedback {
    pub feedback_info: FeedbackInfo,
    pub actions: BTreeMap<String, FeedbackAction>,
}

pub(super) fn build_dont_like_feedback(
    post: &ScoredPost,
    viewer_id: u64,
    language: &str,
    country: Option<&str>,
) -> Option<PostFeedback> {
    let is_retweet = post.retweeted_tweet_id != 0;
    let original_author_id = if is_retweet {
        post.retweeted_user_id
    } else {
        post.author_id
    };

    if original_author_id == 0 || original_author_id == viewer_id {
        return None;
    }

    let mut child_keys: Vec<String> = Vec::new();
    let mut actions: BTreeMap<String, FeedbackAction> = BTreeMap::new();

    let dont_like_prompt = lookup(keys::DONT_LIKE, language, country)?;
    let dont_like_confirmation = lookup(keys::DONT_LIKE_CONFIRMATION, language, country)?;

    if let Some(author_name) = post.screen_names.get(&original_author_id)
        && let Some(prompt) =
            lookup_with_user(keys::SHOW_FEWER_TWEETS, language, country, author_name)
        && let Some(confirmation) = lookup_with_user(
            keys::SHOW_FEWER_TWEETS_CONFIRMATION,
            language,
            country,
            author_name,
        )
    {
        let metadata = FeedbackMetadata {
            injection_type: None,
            engagement_type: Some(FeedbackEngagementType::TWEET),
            entity_ids: Some(vec![FeedbackEntity::UserId(original_author_id as i64)]),
            ttl_ms: Some(FEEDBACK_TTL_MS),
        };
        let action = FeedbackAction {
            feedback_type: FeedbackType::SEE_FEWER,
            prompt: Some(prompt),
            confirmation: Some(confirmation),
            child_keys: None,
            feedback_url: Some(feedback_url("SeeFewer", &metadata)),
            has_undo_action: Some(true),
            confirmation_display_type: None,
            client_event_info: None,
            icon: None,
            rich_behavior: None,
            subprompt: None,
            encoded_feedback_request: None,
        };
        let key = generate_key(&action);
        child_keys.push(key.clone());
        actions.insert(key, action);
    }

    if is_retweet
        && let Some(retweeter_name) = post.screen_names.get(&post.author_id)
        && let Some(prompt) =
            lookup_with_user(keys::SHOW_FEWER_RETWEETS, language, country, retweeter_name)
        && let Some(confirmation) = lookup_with_user(
            keys::SHOW_FEWER_RETWEETS_CONFIRMATION,
            language,
            country,
            retweeter_name,
        )
    {
        let metadata = FeedbackMetadata {
            injection_type: None,
            engagement_type: Some(FeedbackEngagementType::RETWEET),
            entity_ids: Some(vec![FeedbackEntity::UserId(post.author_id as i64)]),
            ttl_ms: Some(FEEDBACK_TTL_MS),
        };
        let action = FeedbackAction {
            feedback_type: FeedbackType::SEE_FEWER,
            prompt: Some(prompt),
            confirmation: Some(confirmation),
            child_keys: None,
            feedback_url: Some(feedback_url("SeeFewer", &metadata)),
            has_undo_action: Some(true),
            confirmation_display_type: None,
            client_event_info: None,
            icon: None,
            rich_behavior: None,
            subprompt: None,
            encoded_feedback_request: None,
        };
        let key = generate_key(&action);
        child_keys.push(key.clone());
        actions.insert(key, action);
    }

    if let Some(prompt) = lookup(keys::NOT_RELEVANT, language, country)
        && let Some(confirmation) = lookup(keys::NOT_RELEVANT_CONFIRMATION, language, country)
    {
        let not_relevant_metadata = FeedbackMetadata {
            injection_type: None,
            engagement_type: None,
            entity_ids: Some(vec![FeedbackEntity::TweetId(post.tweet_id as i64)]),
            ttl_ms: Some(FEEDBACK_TTL_MS),
        };
        let not_relevant = FeedbackAction {
            feedback_type: FeedbackType::NOT_RELEVANT,
            prompt: Some(prompt),
            confirmation: Some(confirmation),
            child_keys: None,
            feedback_url: Some(feedback_url("NotRelevant", &not_relevant_metadata)),
            has_undo_action: Some(true),
            confirmation_display_type: None,
            client_event_info: Some(ClientEventInfo {
                component: None,
                element: Some("feedback_notrelevant".to_string()),
                details: None,
                action: Some("click".to_string()),
                entity_token: None,
            }),
            icon: None,
            rich_behavior: None,
            subprompt: None,
            encoded_feedback_request: None,
        };
        let key = generate_key(&not_relevant);
        child_keys.push(key.clone());
        actions.insert(key, not_relevant);
    }

    if let Some(topic_id) = post
        .topic_feedback_topic_id
        .as_ref()
        .filter(|id| !id.is_empty())
        .and_then(|id| id.parse::<i64>().ok())
        && let Some(topic_name) = post
            .topic_feedback_topic
            .as_ref()
            .filter(|name| !name.is_empty())
        && let Some(prompt) =
            lookup_with_topic(keys::NOT_INTERESTED_IN_TOPIC, language, country, topic_name)
        && let Some(confirmation) = lookup(
            keys::NOT_INTERESTED_IN_TOPIC_CONFIRMATION,
            language,
            country,
        )
    {
        let metadata = FeedbackMetadata {
            injection_type: None,
            engagement_type: None,
            entity_ids: Some(vec![FeedbackEntity::TopicId(topic_id)]),
            ttl_ms: Some(FEEDBACK_TTL_MS),
        };
        let action = FeedbackAction {
            feedback_type: FeedbackType::NOT_RELEVANT,
            prompt: Some(prompt),
            confirmation: Some(confirmation),
            child_keys: None,
            feedback_url: Some(feedback_url("NotRelevant", &metadata)),
            has_undo_action: Some(true),
            confirmation_display_type: None,
            client_event_info: None,
            icon: None,
            rich_behavior: None,
            subprompt: None,
            encoded_feedback_request: None,
        };
        let key = generate_key(&action);
        child_keys.push(key.clone());
        actions.insert(key, action);
    }

    let dont_like_metadata = FeedbackMetadata {
        injection_type: None,
        engagement_type: None,
        entity_ids: Some(vec![
            FeedbackEntity::TweetId(post.tweet_id as i64),
            FeedbackEntity::UserId(original_author_id as i64),
        ]),
        ttl_ms: Some(FEEDBACK_TTL_MS),
    };
    let dont_like = FeedbackAction {
        feedback_type: FeedbackType::DONT_LIKE,
        prompt: Some(dont_like_prompt),
        confirmation: Some(dont_like_confirmation),
        child_keys: if child_keys.is_empty() {
            None
        } else {
            Some(child_keys)
        },
        feedback_url: Some(feedback_url("DontLike", &dont_like_metadata)),
        has_undo_action: Some(true),
        confirmation_display_type: None,
        client_event_info: Some(ClientEventInfo {
            component: None,
            element: Some("feedback_dontlike".to_string()),
            details: None,
            action: Some("click".to_string()),
            entity_token: None,
        }),
        icon: Some(HorizonIcon::FROWN),
        rich_behavior: None,
        subprompt: None,
        encoded_feedback_request: None,
    };
    let dont_like_key = generate_key(&dont_like);
    actions.insert(dont_like_key.clone(), dont_like);

    Some(PostFeedback {
        feedback_info: FeedbackInfo {
            feedback_keys: vec![dont_like_key],
            feedback_metadata: None,
            display_context: None,
            client_event_info: None,
        },
        actions,
    })
}

fn generate_key(action: &FeedbackAction) -> String {
    let mut hasher = std::hash::DefaultHasher::new();
    action.hash(&mut hasher);
    hasher.finish().to_string()
}

pub(super) fn feedback_url(feedback_type: &str, metadata: &FeedbackMetadata) -> String {
    let bytes = serialize_compact(metadata).expect("compact thrift serialization is infallible");
    let b64 = STANDARD.encode(&bytes);
    let encoded = percent_encode(&b64);
    format!(
        "/2/timeline/feedback.json?feedback_type={}&action_metadata={}",
        feedback_type, encoded
    )
}

fn lookup(key: &str, language: &str, country: Option<&str>) -> Option<String> {
    xai_stringcenter::global()?.get(key, language, country)
}

fn lookup_with_user(
    key: &str,
    language: &str,
    country: Option<&str>,
    user: &str,
) -> Option<String> {
    let placeholders = HashMap::from([("user", user)]);
    xai_stringcenter::global()?.get_with(key, language, country, &placeholders)
}

fn lookup_with_topic(
    key: &str,
    language: &str,
    country: Option<&str>,
    topic_name: &str,
) -> Option<String> {
    let placeholders = HashMap::from([("topicName", topic_name)]);
    xai_stringcenter::global()?.get_with(key, language, country, &placeholders)
}
