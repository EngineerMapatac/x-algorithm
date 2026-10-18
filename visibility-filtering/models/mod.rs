pub mod author;
pub mod conversation_control;
pub mod region;
pub mod safety_labels;
pub mod tweet;
pub mod verdict;
pub mod viewer;

pub use author::{AuthorFeatures, AuthorLabel, AuthorLabelSet};
pub use conversation_control::ConversationControlFeatures;
pub use safety_labels::{SafetyLabelMap, SafetyLabelType};
pub use tweet::{ArticleLifecycle, CommunityModeration, MediaFeature, NsfwFeature, TweetFeatures};
pub use verdict::{
    Decided, DropReason, Evaluation, FosnrReason, LimitedEngagement, LimitedEngagementReason,
    MediaInterstitial, MediaRestriction, Notice, NsfwViewerDropReason, TombstoneReason, Verdict,
    Withholding,
};
pub use viewer::{
    ClientCapability, VerifyBlurSupport, Viewer, ViewerAge, ViewerFeatures, ViewerProfile,
};

use crate::hydration::{Hydrator, Hydrators};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TweetId(pub u64);

pub fn tweet_timestamp_ms(tweet_id: u64) -> u64 {
    const SNOWFLAKE_EPOCH_MS: u64 = 1288834974657;
    const SNOWFLAKE_TIMESTAMP_SHIFT: u32 = 22;
    (tweet_id >> SNOWFLAKE_TIMESTAMP_SHIFT) + SNOWFLAKE_EPOCH_MS
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct AuthorId(pub u64);

impl AuthorId {
    pub fn get(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Copy, Debug)]
pub struct PureCore {
    pub author_id: AuthorId,
    pub source_tweet_id: Option<TweetId>,
    pub source_author_id: Option<AuthorId>,
    pub reply_to: Option<ReplyTo>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReplyTo {
    Root {
        conversation_id: TweetId,
        root_author_id: AuthorId,
    },
    Nested {
        conversation_id: TweetId,
        parent_id: TweetId,
    },
}

#[derive(Clone, Copy, Debug)]
pub struct RawCandidate {
    pub tweet_id: TweetId,
    pub request_author_id: Option<u64>,
}

#[derive(Clone, Debug, Default)]
pub struct PostCandidate {
    pub author: UserEntity,
    pub viewer_relationship: ViewerRelationship,
    pub post: PostEntity,
    pub failed: Hydrators,
}

#[derive(Clone, Debug, Default)]
pub struct UserEntity {
    pub id: u64,
    pub is_suspended: bool,
    pub is_deactivated: bool,
    pub is_protected: bool,
    pub is_nsfw_user: bool,
    pub is_nsfw_admin: bool,
    pub is_erased: bool,
    pub is_offboarded: bool,
    pub labels: AuthorLabelSet,
}

impl UserEntity {
    pub(crate) fn new(id: u64, features: AuthorFeatures, labels: AuthorLabelSet) -> Self {
        let AuthorFeatures {
            is_suspended,
            is_deactivated,
            is_protected,
            is_nsfw_user,
            is_nsfw_admin,
            is_erased,
            is_offboarded,
        } = features;
        Self {
            id,
            is_suspended,
            is_deactivated,
            is_protected,
            is_nsfw_user,
            is_nsfw_admin,
            is_erased,
            is_offboarded,
            labels,
        }
    }

    #[inline]
    pub(crate) fn features(&self) -> AuthorFeatures {
        AuthorFeatures {
            is_suspended: self.is_suspended,
            is_deactivated: self.is_deactivated,
            is_protected: self.is_protected,
            is_nsfw_user: self.is_nsfw_user,
            is_nsfw_admin: self.is_nsfw_admin,
            is_erased: self.is_erased,
            is_offboarded: self.is_offboarded,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ViewerRelationship {
    pub follows: bool,
    pub blocks: bool,
    pub mutes: bool,
    pub blocked_by: bool,
}

impl ViewerRelationship {
    #[inline]
    pub(crate) fn slot(&mut self, node: Hydrator) -> Option<&mut bool> {
        match node {
            Hydrator::Follows => Some(&mut self.follows),
            Hydrator::Blocks => Some(&mut self.blocks),
            Hydrator::Mutes => Some(&mut self.mutes),
            Hydrator::BlockedByAuthor => Some(&mut self.blocked_by),
            Hydrator::PureCore
            | Hydrator::Tweet
            | Hydrator::ConversationControl
            | Hydrator::TweetSafetyLabels
            | Hydrator::ViewerProfile
            | Hydrator::ViewerLabels
            | Hydrator::AuthorSafety
            | Hydrator::AuthorLabels
            | Hydrator::MuteRetweets
            | Hydrator::BlockedByReplyRoot
            | Hydrator::SuperFollowsExclusive
            | Hydrator::RootFollowsViewer
            | Hydrator::RootFollowsViewerSecondDegree
            | Hydrator::SuperFollowsRoot
            | Hydrator::BlockedByConversationRoot
            | Hydrator::ConversationRootPost
            | Hydrator::BlockedByConversationRootPost
            | Hydrator::ViewerCountry
            | Hydrator::CommunityModeration
            | Hydrator::CommunityModerator
            | Hydrator::CommunityViewerRemoved
            | Hydrator::ArticleLifecycle
            | Hydrator::ArticleMediaEntities
            | Hydrator::ArticleCoverMedia
            | Hydrator::TrustedFriends
            | Hydrator::OutsideNarrowcastPlace => None,
        }
    }

    #[inline]
    pub(crate) fn get(mut self, node: Hydrator) -> Option<bool> {
        self.slot(node).copied()
    }
}

#[derive(Clone, Debug, Default)]
pub struct PostEntity {
    pub tweet_id: u64,
    pub source_tweet_id: Option<u64>,
    pub is_reply: bool,
    pub tweet_features: TweetFeatures,
    pub safety_labels: SafetyLabelMap,
    pub edges: Hydrators,
    pub conversation_control: Option<ConversationControlFeatures>,
    pub community_moderation: CommunityModeration,
    pub viewer_is_community_moderator: Option<bool>,
    pub viewer_is_removed_from_community: bool,
    pub article_lifecycle: Option<ArticleLifecycle>,
    pub article_has_media_entities: bool,
    pub article_has_cover_media: bool,
}
