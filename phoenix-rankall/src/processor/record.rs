use std::collections::HashMap;

pub type PostId = i64;
pub type AuthorId = i64;

#[derive(Debug, Clone)]
pub enum IndexRecord {
    Core {
        post_id: PostId,
        author_id: AuthorId,
        index_name: String,
    },

    Topic {
        post_id: PostId,
        author_id: AuthorId,
        index_name: String,
        topic_entity_ids: Vec<i64>,
    },

    Metadata {
        post_id: PostId,
        author_id: AuthorId,
        has_video: bool,
        has_image: bool,
        video_duration_ms: i64,
        author_followers_count: i64,
        engagement: EngagementCounts,
    },

    Ads {
        post_id: PostId,
        author_id: AuthorId,
    },

    Sid {
        post_id: PostId,
        author_id: AuthorId,
        index_name: String,
        post_sid: Vec<i32>,
    },

    Analysis {
        post_id: PostId,
        author_id: AuthorId,
        viewer_id: i64,
        served_type: Option<String>,
        scores: HashMap<String, f64>,
    },
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EngagementCounts {
    pub retweet_count: i64,
    pub reply_count: i64,
    pub fav_count: i64,
    pub quote_count: i64,
    pub bookmark_count: i64,
    pub view_count: i64,
    pub not_interested_in_count: i64,
    pub report_count: i64,
    pub block_count: i64,
}

impl EngagementCounts {
    pub fn merge(&self, incoming: &Self) -> Self {
        Self {
            retweet_count: self.retweet_count.max(incoming.retweet_count),
            reply_count: self.reply_count.max(incoming.reply_count),
            fav_count: self.fav_count.max(incoming.fav_count),
            quote_count: self.quote_count.max(incoming.quote_count),
            bookmark_count: self.bookmark_count.max(incoming.bookmark_count),
            view_count: self.view_count.max(incoming.view_count),
            not_interested_in_count: self
                .not_interested_in_count
                .max(incoming.not_interested_in_count),
            report_count: self.report_count.max(incoming.report_count),
            block_count: self.block_count.max(incoming.block_count),
        }
    }
}

impl IndexRecord {
    pub fn post_id(&self) -> PostId {
        match self {
            Self::Core { post_id, .. }
            | Self::Topic { post_id, .. }
            | Self::Metadata { post_id, .. }
            | Self::Ads { post_id, .. }
            | Self::Sid { post_id, .. }
            | Self::Analysis { post_id, .. } => *post_id,
        }
    }

    pub fn author_id(&self) -> AuthorId {
        match self {
            Self::Core { author_id, .. }
            | Self::Topic { author_id, .. }
            | Self::Metadata { author_id, .. }
            | Self::Ads { author_id, .. }
            | Self::Sid { author_id, .. }
            | Self::Analysis { author_id, .. } => *author_id,
        }
    }

    pub fn index_name(&self) -> &str {
        match self {
            Self::Core { index_name, .. }
            | Self::Topic { index_name, .. }
            | Self::Sid { index_name, .. } => index_name,
            Self::Metadata { .. } => "metadata",
            Self::Ads { .. } => "ads",
            Self::Analysis { .. } => "analysis",
        }
    }

    pub fn merge_with(self, incoming: Self) -> Self {
        match (self, incoming) {
            (
                Self::Metadata {
                    post_id,
                    author_id,
                    has_video,
                    has_image,
                    video_duration_ms,
                    author_followers_count,
                    engagement,
                },
                Self::Metadata {
                    author_id: in_author,
                    has_video: in_video,
                    has_image: in_image,
                    video_duration_ms: in_duration,
                    author_followers_count: in_followers,
                    engagement: in_engagement,
                    ..
                },
            ) => Self::Metadata {
                post_id,
                author_id: if in_author != 0 { in_author } else { author_id },
                has_video: has_video || in_video,
                has_image: has_image || in_image,
                video_duration_ms: video_duration_ms.max(in_duration),
                author_followers_count: author_followers_count.max(in_followers),
                engagement: engagement.merge(&in_engagement),
            },
            (_, incoming) => incoming,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merge_keeps_positive_fav_when_incoming_is_zero() {
        let seen = EngagementCounts {
            fav_count: 50,
            view_count: 200,
            ..Default::default()
        };
        let empty = EngagementCounts::default();
        let merged = seen.merge(&empty);
        assert_eq!(merged.fav_count, 50);
        assert_eq!(merged.view_count, 200);
    }

    #[test]
    fn merge_takes_higher_later_snapshot() {
        let seen = EngagementCounts {
            fav_count: 50,
            view_count: 200,
            ..Default::default()
        };
        let later = EngagementCounts {
            fav_count: 80,
            view_count: 10,
            ..Default::default()
        };
        let merged = seen.merge(&later);
        assert_eq!(merged.fav_count, 80);
        assert_eq!(merged.view_count, 200);
    }

    #[test]
    fn metadata_merge_does_not_reset_engagement() {
        let first = IndexRecord::Metadata {
            post_id: 1,
            author_id: 2,
            has_video: false,
            has_image: true,
            video_duration_ms: 0,
            author_followers_count: 10,
            engagement: EngagementCounts {
                fav_count: 50,
                ..Default::default()
            },
        };
        let later = IndexRecord::Metadata {
            post_id: 1,
            author_id: 2,
            has_video: false,
            has_image: false,
            video_duration_ms: 0,
            author_followers_count: 0,
            engagement: EngagementCounts::default(),
        };
        match first.merge_with(later) {
            IndexRecord::Metadata {
                engagement,
                author_followers_count,
                has_image,
                ..
            } => {
                assert_eq!(engagement.fav_count, 50);
                assert_eq!(author_followers_count, 10);
                assert!(has_image);
            }
            other => panic!("unexpected {other:?}"),
        }
    }
}
