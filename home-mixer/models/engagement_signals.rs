use serde::Serialize;
use std::collections::{BTreeMap, HashSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum EngagementSignalType {
    Favorite,
    Retweet,
    Reply,
    Bookmark,
    Share,
    OriginalTweet,
    PhotoExpand,
    VideoQualityView,
    ImmersiveVideoQualityView,
}

impl EngagementSignalType {
    pub fn as_str(&self) -> &'static str {
        match self {
            EngagementSignalType::Favorite => "favorite",
            EngagementSignalType::Retweet => "retweet",
            EngagementSignalType::Reply => "reply",
            EngagementSignalType::Bookmark => "bookmark",
            EngagementSignalType::Share => "share",
            EngagementSignalType::OriginalTweet => "original_tweet",
            EngagementSignalType::PhotoExpand => "photo_expand",
            EngagementSignalType::VideoQualityView => "video_quality_view",
            EngagementSignalType::ImmersiveVideoQualityView => "immersive_video_quality_view",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct EngagementSignal {
    pub tweet_id: i64,
    pub author_id: Option<i64>,
    pub engaged_at_ms: i64,
}

pub type EngagementSignalsByType = BTreeMap<EngagementSignalType, Vec<EngagementSignal>>;

pub fn sort_dedup_truncate(
    mut signals: Vec<EngagementSignal>,
    max_per_type: usize,
) -> Vec<EngagementSignal> {
    signals.sort_by_key(|b| std::cmp::Reverse(b.engaged_at_ms));
    let mut seen_tweet_ids = HashSet::new();
    signals.retain(|signal| seen_tweet_ids.insert(signal.tweet_id));
    signals.truncate(max_per_type);
    signals
}
