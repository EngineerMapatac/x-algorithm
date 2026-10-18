use super::post_marshaller::{
    make_tweet, make_tweet_helper, make_tweet_item, ENTRY_NAMESPACE_TWEET,
};
use xai_home_mixer_proto::PushToHomePost;
use xai_urt_thrift::entry::{TimelineEntry, TimelineEntryContent};
use xai_urt_thrift::metadata::{ClientEventInfo, TimelinesDetails};
use xai_urt_thrift::timeline_module::{
    ModuleConversationMetadata, ModuleDisplayType, ModuleItem, ModuleMetadata, TimelineModule,
};
use xai_urt_thrift::tweet::{TweetFacepile, TweetFacepileDisplayType};

const ENTRY_NAMESPACE: &str = "push-to-home";

pub(super) fn marshal_push_to_home(post: &PushToHomePost, sort_index: i64) -> TimelineEntry {
    let focal_id = post.tweet_id;
    let component = super::client_event::served_type_component(post.served_type);

    let cei = build_client_event_info(&component);

    let is_root = post.in_reply_to_tweet_id == 0;
    let root = if post.conversation_id != 0 && post.conversation_id != focal_id {
        Some(post.conversation_id)
    } else {
        None
    };

    let focal_tweet = if is_root && !post.facepile_user_ids.is_empty() {
        let facepile = TweetFacepile::new(
            post.facepile_user_ids.iter().map(|&id| id as i64).collect(),
            None::<TweetFacepileDisplayType>,
        );
        make_tweet_helper(focal_id, Some(facepile), None)
    } else {
        make_tweet(focal_id)
    };

    let mut module_items: Vec<ModuleItem> = Vec::with_capacity(2);
    if let Some(root_id) = root {
        module_items.push(ModuleItem {
            entry_id: format!("{ENTRY_NAMESPACE_TWEET}-{root_id}"),
            item: make_tweet_item(make_tweet(root_id), Some(cei.clone()), None),
            dispensable: None,
            tree_display: None,
            pill_group: None,
        });
    }
    module_items.push(ModuleItem {
        entry_id: format!("{ENTRY_NAMESPACE_TWEET}-{focal_id}"),
        item: make_tweet_item(focal_tweet, Some(cei.clone()), None),
        dispensable: None,
        tree_display: None,
        pill_group: None,
    });

    let all_tweet_ids = build_all_tweet_ids(post);

    TimelineEntry {
        entry_id: format!("{ENTRY_NAMESPACE}-{focal_id}"),
        sort_index,
        content: TimelineEntryContent::TimelineModule(TimelineModule {
            items: module_items,
            display_type: ModuleDisplayType::VERTICAL_CONVERSATION,
            header: None,
            footer: None,
            client_event_info: Some(ClientEventInfo {
                component: Some(component),
                element: None,
                details: None,
                action: None,
                entity_token: None,
            }),
            feedback_info: None,
            metadata: Some(ModuleMetadata {
                ads_metadata: None,
                conversation_metadata: Some(ModuleConversationMetadata {
                    all_tweet_ids: Some(all_tweet_ids),
                    social_context: None,
                    enable_deduplication: Some(true),
                }),
                grid_carousel_metadata: None,
                vertical_metadata: None,
                pill_group_metadata: None,
            }),
            show_more_behavior: None,
        }),
        expiry_time: None,
    }
}

fn build_client_event_info(component: &str) -> ClientEventInfo {
    let mut details = super::client_event::empty_details();
    details.timelines_details = Some(TimelinesDetails {
        injection_type: Some(component.to_string()),
        controller_data: None,
        source_data: None,
    });
    ClientEventInfo {
        component: Some(component.to_string()),
        element: Some(super::client_event::ELEMENT_TWEET.to_string()),
        details: Some(details),
        action: None,
        entity_token: None,
    }
}

fn build_all_tweet_ids(post: &PushToHomePost) -> Vec<i64> {
    let mut ids: Vec<i64> = Vec::with_capacity(3);
    if post.conversation_id != 0 {
        ids.push(post.conversation_id as i64);
    }
    if post.in_reply_to_tweet_id != 0 && !ids.contains(&(post.in_reply_to_tweet_id as i64)) {
        ids.push(post.in_reply_to_tweet_id as i64);
    }
    if !ids.contains(&(post.tweet_id as i64)) {
        ids.push(post.tweet_id as i64);
    }
    ids
}
