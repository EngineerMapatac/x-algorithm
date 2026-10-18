use super::client_event::{post_client_event_info, served_type_component};
use xai_home_mixer_proto::ScoredPost;
use xai_urt_thrift::entry::{TimelineEntry, TimelineEntryContent};
use xai_urt_thrift::item::{TimelineItem, TimelineItemContent};
use xai_urt_thrift::metadata::{
    ClientEventInfo, ContextType, FeedbackInfo, TopicFeedbackContext, TweetContext,
    TweetContextDetails, Url, UrlType,
};
use xai_urt_thrift::tweet::{Tweet, TweetDisplayType, TweetFacepile};

pub(super) const ENTRY_NAMESPACE_TWEET: &str = "tweet";

pub(super) fn make_tweet(id: u64) -> Tweet {
    make_tweet_helper(id, None, None)
}

pub(super) fn make_tweet_helper(
    id: u64,
    tweet_facepile: Option<TweetFacepile>,
    tweet_context: Option<TweetContext>,
) -> Tweet {
    Tweet {
        id: id as i64,
        display_type: TweetDisplayType::TWEET,
        promoted_metadata: None,
        social_context: None,
        highlights: None,
        campaign_metadata: None,
        display_size: None,
        inner_tombstone_info: None,
        follow_prompt: None,
        tweet_composer: None,
        timelines_score_info: None,
        preroll_metadata: None,
        quote_pivot: None,
        preview_metadata: None,
        min_spacing: None,
        reply_badge: None,
        has_moderated_replies: None,
        tweet_context,
        tweet_social_proof: None,
        forward_pivot: None,
        education_prompt: None,
        rux_context: None,
        inner_forward_pivot: None,
        reactive_triggers: None,
        topic_follow_prompt: None,
        conversation_annotation: None,
        contextual_tweet_ref: None,
        reactive_triggers_v2: None,
        destination: None,
        ssp_metadata: None,
        tweet_facepile,
    }
}

pub(super) fn make_tweet_item(
    tweet: Tweet,
    client_event_info: Option<ClientEventInfo>,
    feedback_info: Option<FeedbackInfo>,
) -> TimelineItem {
    TimelineItem {
        content: TimelineItemContent::Tweet(tweet),
        client_event_info,
        feedback_info,
        prompt: None,
        reactive_triggers: None,
    }
}

pub(super) fn tweet_context_from_post(post: &ScoredPost) -> Option<TweetContext> {
    if !is_standalone_original_post(post) {
        return None;
    }
    if let Some(topic) = post.topic_feedback_topic.as_ref().filter(|t| !t.is_empty()) {
        return Some(TweetContext {
            context_type: ContextType::TOPIC,
            text: topic.clone(),
            context_image_urls: None,
            landing_url: None,
            context: Some(TweetContextDetails::TopicFeedbackContext(
                TopicFeedbackContext {
                    topic: Some(topic.clone()),
                    url: None,
                    topic_id: post.topic_feedback_topic_id.clone(),
                },
            )),
            icon: None,
        });
    }
    let trend = post.ai_trend_name.as_ref().filter(|t| !t.is_empty())?;
    let trend_id = post.ai_trend_id.as_ref().filter(|id| !id.is_empty())?;
    let landing_url = Url {
        url_type: UrlType::EXTERNAL_URL,
        url: format!("https://x.com/i/trending/{trend_id}"),
        urt_endpoint_options: None,
    };
    Some(TweetContext {
        context_type: ContextType::TOPIC,
        text: trend.clone(),
        context_image_urls: None,
        landing_url: Some(landing_url.clone()),
        context: Some(TweetContextDetails::TopicFeedbackContext(
            TopicFeedbackContext {
                topic: Some(trend.clone()),
                url: Some(landing_url),
                topic_id: None,
            },
        )),
        icon: None,
    })
}

fn is_standalone_original_post(post: &ScoredPost) -> bool {
    post.in_reply_to_tweet_id == 0
        && post.retweeted_tweet_id == 0
        && post.ancestors.is_empty()
        && post.following_replied_user_ids.is_empty()
}

pub(super) fn marshal_post(
    post: &ScoredPost,
    sort_index: i64,
    feedback_info: Option<FeedbackInfo>,
) -> TimelineEntry {
    let component = served_type_component(post.served_type);
    let cei = post_client_event_info(&component, post, sort_index as i32);
    let tweet = make_tweet_helper(post.tweet_id, None, tweet_context_from_post(post));
    TimelineEntry {
        entry_id: format!("{}-{}", ENTRY_NAMESPACE_TWEET, post.tweet_id),
        sort_index,
        content: TimelineEntryContent::Item(make_tweet_item(tweet, Some(cei), feedback_info)),
        expiry_time: None,
    }
}
