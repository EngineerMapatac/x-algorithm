use crate::clients::socialgraph_client::{EdgeDirection, Graph};
use crate::hydration::inventory::{MissPolicy, Source, Subject};
use crate::hydration::sources::Scope;
use std::fmt;
use strum::VariantArray;
use xai_core_entities::entities::ConversationControlArm;
use xai_core_entities::gizmoduck_client::QueryFields;

#[derive(Clone, Copy, Debug, PartialEq, Eq, strum::IntoStaticStr, strum::VariantArray)]
#[strum(serialize_all = "snake_case")]
#[repr(u8)]
pub enum Hydrator {
    PureCore,
    Tweet,
    ConversationControl,
    TweetSafetyLabels,
    ViewerProfile,
    ViewerLabels,
    AuthorSafety,
    AuthorLabels,
    Follows,
    Blocks,
    Mutes,
    MuteRetweets,
    BlockedByAuthor,
    BlockedByReplyRoot,
    SuperFollowsExclusive,
    RootFollowsViewer,
    RootFollowsViewerSecondDegree,
    SuperFollowsRoot,
    BlockedByConversationRoot,
    ConversationRootPost,
    BlockedByConversationRootPost,
    ViewerCountry,
    CommunityModeration,
    CommunityModerator,
    CommunityViewerRemoved,
    ArticleLifecycle,
    ArticleMediaEntities,
    ArticleCoverMedia,
    TrustedFriends,
    OutsideNarrowcastPlace,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Hydrators(u32);

impl Hydrators {
    pub const fn empty() -> Self {
        Self(0)
    }

    #[cfg(test)]
    pub fn all() -> Self {
        Self(u32::MAX >> (u32::BITS as usize - <Hydrator as strum::VariantArray>::VARIANTS.len()))
    }

    pub const fn of(hydrator: Hydrator) -> Self {
        Self(1 << hydrator as u8)
    }

    pub const fn with(self, hydrator: Hydrator) -> Self {
        self.union(Self::of(hydrator))
    }

    #[cfg(test)]
    pub const fn without(self, hydrator: Hydrator) -> Self {
        Self(self.0 & !Self::of(hydrator).0)
    }

    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    pub const fn contains(self, hydrator: Hydrator) -> bool {
        self.0 & Self::of(hydrator).0 != 0
    }

    pub const fn difference(self, other: Self) -> Self {
        Self(self.0 & !other.0)
    }

    pub const fn intersection(self, other: Self) -> Self {
        Self(self.0 & other.0)
    }

    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Part {
    Column,
    Fields(&'static [QueryFields]),
    Edge(Edge),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, VariantArray)]
#[repr(u8)]
pub(super) enum Edge {
    Follows,
    Blocks,
    Mutes,
    MuteRetweets,
    BlockedBy,
    SuperFollows,
    FollowedBy,
    SecondDegree,
    TrustedFriends,
    OutsidePlace,
}

impl Edge {
    pub(super) const fn flock(self) -> Option<(Graph, EdgeDirection)> {
        use EdgeDirection::{Forward, Reverse};
        match self {
            Edge::Follows => Some((Graph::Follows, Forward)),
            Edge::Blocks => Some((Graph::Blocks, Forward)),
            Edge::Mutes => Some((Graph::Mutes, Forward)),
            Edge::MuteRetweets => Some((Graph::MuteRetweets, Forward)),
            Edge::BlockedBy => Some((Graph::Blocks, Reverse)),
            Edge::SuperFollows => Some((Graph::SuperFollows, Forward)),
            Edge::FollowedBy => Some((Graph::Follows, Reverse)),
            Edge::SecondDegree | Edge::TrustedFriends | Edge::OutsidePlace => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum KeyOrigin {
    RequestTweets,
    Viewer,
    PureCoreAuthor,
    PureCoreRetweeter,
    PureCoreReplyRoot,
    ExclusiveConversationAuthor,
    TweetArticle,
    ConversationRoot(&'static [ConversationControlArm]),
    ConversationControlBlockAuthor,
    ConversationRootToFetch,
    ConversationRootPostAuthor,
    ViewerForCoAllowedList,
    MyNetworkRootNotFollowingViewer,
    CommunityPost,
    ModeratedCommunity,
    TweetCommunity,
    TrustedFriendsList,
    NarrowcastPlace,
}

const COMMUNITY_OR_MY_NETWORK: &[ConversationControlArm] = &[
    ConversationControlArm::Community,
    ConversationControlArm::MyNetwork,
];
const SUBSCRIBERS: &[ConversationControlArm] = &[ConversationControlArm::Subscribers];

pub(super) fn arm_starts_conversation_root_select(arm: ConversationControlArm) -> bool {
    [COMMUNITY_OR_MY_NETWORK, SUBSCRIBERS]
        .into_iter()
        .any(|arms| arms.contains(&arm))
}

#[derive(Clone, Copy, Debug)]
pub(super) struct NodeSpec {
    pub(super) source: Source,
    pub(super) part: Part,
    pub(super) key: KeyOrigin,
    pub(super) label: (&'static str, &'static str),
}

impl KeyOrigin {
    const fn input(self) -> Option<Hydrator> {
        match self {
            KeyOrigin::RequestTweets | KeyOrigin::Viewer => None,
            KeyOrigin::PureCoreAuthor
            | KeyOrigin::PureCoreRetweeter
            | KeyOrigin::PureCoreReplyRoot
            | KeyOrigin::ConversationRootToFetch => Some(Hydrator::PureCore),
            KeyOrigin::ConversationRootPostAuthor => Some(Hydrator::ConversationRootPost),
            KeyOrigin::ExclusiveConversationAuthor
            | KeyOrigin::CommunityPost
            | KeyOrigin::TweetCommunity
            | KeyOrigin::TweetArticle
            | KeyOrigin::TrustedFriendsList
            | KeyOrigin::NarrowcastPlace => Some(Hydrator::Tweet),
            KeyOrigin::ConversationRoot(_)
            | KeyOrigin::ConversationControlBlockAuthor
            | KeyOrigin::ViewerForCoAllowedList => Some(Hydrator::ConversationControl),
            KeyOrigin::MyNetworkRootNotFollowingViewer => Some(Hydrator::RootFollowsViewer),
            KeyOrigin::ModeratedCommunity => Some(Hydrator::CommunityModeration),
        }
    }

    pub(super) fn reads(self) -> Hydrators {
        let input = match self.input() {
            Some(input) => Hydrators::of(input),
            None => Hydrators::empty(),
        };
        match self {
            KeyOrigin::MyNetworkRootNotFollowingViewer | KeyOrigin::ConversationRootToFetch => {
                input.with(Hydrator::ConversationControl)
            }
            KeyOrigin::CommunityPost | KeyOrigin::ConversationControlBlockAuthor => {
                input.with(Hydrator::PureCore)
            }
            KeyOrigin::ConversationRootPostAuthor => input
                .with(Hydrator::PureCore)
                .with(Hydrator::ConversationControl),
            KeyOrigin::RequestTweets
            | KeyOrigin::Viewer
            | KeyOrigin::PureCoreAuthor
            | KeyOrigin::PureCoreRetweeter
            | KeyOrigin::PureCoreReplyRoot
            | KeyOrigin::ExclusiveConversationAuthor
            | KeyOrigin::TweetArticle
            | KeyOrigin::ConversationRoot(_)
            | KeyOrigin::ViewerForCoAllowedList
            | KeyOrigin::ModeratedCommunity
            | KeyOrigin::TweetCommunity
            | KeyOrigin::TrustedFriendsList
            | KeyOrigin::NarrowcastPlace => input,
        }
    }
}

impl Hydrator {
    pub(super) const fn spec(self) -> NodeSpec {
        use Hydrator as H;
        use KeyOrigin as K;
        use Source as S;
        const fn node(
            source: Source,
            part: Part,
            key: KeyOrigin,
            label: (&'static str, &'static str),
        ) -> NodeSpec {
            NodeSpec {
                source,
                part,
                key,
                label,
            }
        }
        const RELATIONSHIPS: (&str, &str) = ("socialgraph", "batch_check_relationships");
        const BLOCKED_BY: (&str, &str) = ("blocked_by", "batch_check_blocked_by");
        match self {
            H::PureCore => node(
                S::TesPureCoreData,
                Part::Column,
                K::RequestTweets,
                ("tes", "get_tweet_core_datas"),
            ),
            H::Tweet => node(
                S::TesTweet,
                Part::Column,
                K::RequestTweets,
                ("tes", "get_tweets"),
            ),
            H::ConversationControl => node(
                S::TesConversationControl,
                Part::Column,
                K::RequestTweets,
                ("conversation_control", "get_conversation_controls"),
            ),
            H::TweetSafetyLabels => node(
                S::SafetyLabels,
                Part::Column,
                K::RequestTweets,
                ("safety_labels", "get"),
            ),
            H::ViewerProfile => node(
                S::GizmoduckViewer,
                Part::Fields(&[
                    QueryFields::ACCOUNT,
                    QueryFields::EXTENDED_PROFILE,
                    QueryFields::SAFETY,
                ]),
                K::Viewer,
                ("gizmoduck", "get_viewer_data"),
            ),
            H::ViewerLabels => node(
                S::GizmoduckViewer,
                Part::Fields(&[QueryFields::LABELS]),
                K::Viewer,
                ("gizmoduck", "get_viewer_data"),
            ),
            H::AuthorSafety => node(
                S::GizmoduckAuthor,
                Part::Fields(&[QueryFields::SAFETY]),
                K::PureCoreAuthor,
                ("gizmoduck", "get_users"),
            ),
            H::AuthorLabels => node(
                S::GizmoduckAuthor,
                Part::Fields(&[QueryFields::LABELS]),
                K::PureCoreAuthor,
                ("gizmoduck", "get_users"),
            ),
            H::Follows => node(
                S::Flock,
                Part::Edge(Edge::Follows),
                K::PureCoreAuthor,
                RELATIONSHIPS,
            ),
            H::Blocks => node(
                S::Flock,
                Part::Edge(Edge::Blocks),
                K::PureCoreAuthor,
                RELATIONSHIPS,
            ),
            H::Mutes => node(
                S::Flock,
                Part::Edge(Edge::Mutes),
                K::PureCoreAuthor,
                RELATIONSHIPS,
            ),
            H::MuteRetweets => node(
                S::Flock,
                Part::Edge(Edge::MuteRetweets),
                K::PureCoreRetweeter,
                RELATIONSHIPS,
            ),
            H::BlockedByAuthor => node(
                S::Flock,
                Part::Edge(Edge::BlockedBy),
                K::PureCoreAuthor,
                BLOCKED_BY,
            ),
            H::BlockedByReplyRoot => node(
                S::Flock,
                Part::Edge(Edge::BlockedBy),
                K::PureCoreReplyRoot,
                BLOCKED_BY,
            ),
            H::SuperFollowsExclusive => node(
                S::Flock,
                Part::Edge(Edge::SuperFollows),
                K::ExclusiveConversationAuthor,
                ("exclusive_content", "batch_check_super_follows"),
            ),
            H::RootFollowsViewer => node(
                S::Flock,
                Part::Edge(Edge::FollowedBy),
                K::ConversationRoot(COMMUNITY_OR_MY_NETWORK),
                ("conversation_control", "batch_check_followed_by"),
            ),
            H::RootFollowsViewerSecondDegree => node(
                S::Wingman,
                Part::Edge(Edge::SecondDegree),
                K::MyNetworkRootNotFollowingViewer,
                ("conversation_control", "exists_intersect"),
            ),
            H::SuperFollowsRoot => node(
                S::Flock,
                Part::Edge(Edge::SuperFollows),
                K::ConversationRoot(SUBSCRIBERS),
                ("conversation_control", "batch_check_super_follows"),
            ),
            H::BlockedByConversationRoot => node(
                S::Flock,
                Part::Edge(Edge::BlockedBy),
                K::ConversationControlBlockAuthor,
                ("conversation_control", "batch_check_blocked_by"),
            ),
            H::ConversationRootPost => node(
                S::TesConversationRoot,
                Part::Column,
                K::ConversationRootToFetch,
                ("conversation_root", "get_tweet_core_datas"),
            ),
            H::BlockedByConversationRootPost => node(
                S::Flock,
                Part::Edge(Edge::BlockedBy),
                K::ConversationRootPostAuthor,
                ("conversation_root", "batch_check_blocked_by"),
            ),
            H::ViewerCountry => node(
                S::ViewerCountry,
                Part::Column,
                K::ViewerForCoAllowedList,
                ("conversation_control", "tfe_top_country"),
            ),
            H::CommunityModeration => node(
                S::CommunityModeration,
                Part::Column,
                K::CommunityPost,
                ("communities", "moderation_state"),
            ),
            H::CommunityModerator => node(
                S::CommunityModerator,
                Part::Column,
                K::ModeratedCommunity,
                ("communities", "visibility_features"),
            ),
            H::CommunityViewerRemoved => node(
                S::CommunityViewerRemoved,
                Part::Column,
                K::TweetCommunity,
                ("communities", "is_removed"),
            ),
            H::ArticleLifecycle => node(
                S::ArticleLifecycle,
                Part::Column,
                K::TweetArticle,
                ("article", "get_lifecycles"),
            ),
            H::ArticleMediaEntities => node(
                S::ArticleMediaEntities,
                Part::Column,
                K::TweetArticle,
                ("article", "get_media_entities"),
            ),
            H::ArticleCoverMedia => node(
                S::ArticleCoverMedia,
                Part::Column,
                K::TweetArticle,
                ("article", "get_cover_media"),
            ),
            H::TrustedFriends => node(
                S::TrustedFriends,
                Part::Edge(Edge::TrustedFriends),
                K::TrustedFriendsList,
                ("trusted_friends", "is_member_or_owner"),
            ),
            H::OutsideNarrowcastPlace => node(
                S::UserLocation,
                Part::Edge(Edge::OutsidePlace),
                K::NarrowcastPlace,
                ("geoduck", "user_location"),
            ),
        }
    }

    pub(crate) const fn is_best_effort(self) -> bool {
        use Hydrator as H;
        match self {
            H::ConversationControl
            | H::ConversationRootPost
            | H::Follows
            | H::BlockedByAuthor
            | H::BlockedByReplyRoot
            | H::BlockedByConversationRoot
            | H::BlockedByConversationRootPost
            | H::RootFollowsViewer
            | H::RootFollowsViewerSecondDegree
            | H::SuperFollowsRoot
            | H::ViewerCountry
            | H::CommunityViewerRemoved
            | H::OutsideNarrowcastPlace => true,
            H::PureCore
            | H::Tweet
            | H::TweetSafetyLabels
            | H::ViewerProfile
            | H::ViewerLabels
            | H::AuthorSafety
            | H::AuthorLabels
            | H::Blocks
            | H::Mutes
            | H::MuteRetweets
            | H::SuperFollowsExclusive
            | H::CommunityModeration
            | H::CommunityModerator
            | H::ArticleLifecycle
            | H::ArticleMediaEntities
            | H::ArticleCoverMedia
            | H::TrustedFriends => false,
        }
    }

    pub(super) const fn input(self) -> Option<Hydrator> {
        self.spec().key.input()
    }

    pub(super) const fn edge(self) -> Option<Edge> {
        match self.spec().part {
            Part::Edge(edge) => Some(edge),
            Part::Column | Part::Fields(_) => None,
        }
    }

    pub(crate) const fn is_edge(self) -> bool {
        self.edge().is_some()
    }

    pub(crate) const fn needs_viewer(self) -> bool {
        self.is_edge()
            || matches!(
                self.spec().key,
                KeyOrigin::Viewer
                    | KeyOrigin::ConversationRootToFetch
                    | KeyOrigin::ViewerForCoAllowedList
                    | KeyOrigin::ModeratedCommunity
                    | KeyOrigin::TweetCommunity
            )
    }
}

const MUST_HAVE: Hydrators = {
    let mut must_have = Hydrators::empty();
    let mut rest = Hydrator::VARIANTS;
    while let [node, tail @ ..] = rest {
        if !node.is_best_effort() {
            must_have = must_have.with(*node);
        }
        rest = tail;
    }
    must_have
};

impl Hydrators {
    pub(crate) const fn must_have(self) -> Self {
        self.intersection(MUST_HAVE)
    }

    pub(crate) const fn has_must_have(self) -> bool {
        !self.must_have().is_empty()
    }
}

const fn inputs_precede_nodes() -> bool {
    let mut rest = Hydrator::VARIANTS;
    while let [node, tail @ ..] = rest {
        if let Some(input) = node.input()
            && input as u8 >= *node as u8
        {
            return false;
        }
        rest = tail;
    }
    true
}

const fn best_effort_inputs_key_only_best_effort_nodes() -> bool {
    let mut rest = Hydrator::VARIANTS;
    while let [node, tail @ ..] = rest {
        if let Some(input) = node.input()
            && input.is_best_effort()
            && !node.is_best_effort()
        {
            return false;
        }
        rest = tail;
    }
    true
}

const fn miss_policies_fit_their_nodes() -> bool {
    let mut rest = Hydrator::VARIANTS;
    while let [node, tail @ ..] = rest {
        let spec = node.spec();
        let fits = match spec.source.miss_policy() {
            MissPolicy::Unresolves(Subject::Tweet) => matches!(spec.key, KeyOrigin::RequestTweets),
            MissPolicy::Unresolves(Subject::Author) => {
                matches!(spec.key, KeyOrigin::PureCoreAuthor)
            }
            MissPolicy::FailsNode => true,
            MissPolicy::ReadsNoEdge => node.is_edge(),
            MissPolicy::ReadsNotFound => !node.is_edge(),
        };
        if !fits {
            return false;
        }
        rest = tail;
    }
    true
}

const fn viewer_scopes_fit_their_nodes() -> bool {
    let mut rest = Hydrator::VARIANTS;
    while let [node, tail @ ..] = rest {
        let is_viewer_scope = matches!(node.spec().source.scope(), Scope::Viewer);
        let fits = if matches!(node, Hydrator::ConversationRootPost) {
            node.needs_viewer() && !is_viewer_scope
        } else {
            node.needs_viewer() == is_viewer_scope
        };
        if !fits {
            return false;
        }
        rest = tail;
    }
    true
}

const _: () = assert!(inputs_precede_nodes());
const _: () = assert!(best_effort_inputs_key_only_best_effort_nodes());
const _: () = assert!(miss_policies_fit_their_nodes());
const _: () = assert!(viewer_scopes_fit_their_nodes());
const _: () = assert!(Hydrator::VARIANTS.len() <= u32::BITS as usize);

impl Hydrators {
    pub const fn closed(self) -> Self {
        let mut closed = self.with(Hydrator::PureCore);
        let mut rest = Hydrator::VARIANTS;
        while let [head @ .., node] = rest {
            if let Some(input) = node.input()
                && closed.contains(*node)
            {
                closed = closed.with(input);
            }
            rest = head;
        }
        closed
    }

    pub(crate) fn iter(self) -> impl Iterator<Item = Hydrator> {
        Hydrator::VARIANTS
            .iter()
            .copied()
            .filter(move |&node| self.contains(node))
    }
}

impl fmt::Display for KeyOrigin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KeyOrigin::RequestTweets => f.write_str("tweets"),
            KeyOrigin::Viewer => f.write_str("viewer"),
            KeyOrigin::PureCoreAuthor => f.write_str("author"),
            KeyOrigin::PureCoreRetweeter => f.write_str("retweeter"),
            KeyOrigin::PureCoreReplyRoot => f.write_str("reply_root"),
            KeyOrigin::ExclusiveConversationAuthor => f.write_str("exclusive_author"),
            KeyOrigin::TweetArticle => f.write_str("article"),
            KeyOrigin::ConversationRoot(arms) => {
                let arms: Vec<String> = arms.iter().map(|arm| format!("{arm:?}")).collect();
                write!(f, "root:{}", arms.join("|"))
            }
            KeyOrigin::ConversationControlBlockAuthor => f.write_str("control_root_author"),
            KeyOrigin::ConversationRootToFetch => f.write_str("conversation_root_to_fetch"),
            KeyOrigin::ConversationRootPostAuthor => f.write_str("conversation_root_post_author"),
            KeyOrigin::ViewerForCoAllowedList => f.write_str("viewer:co_allowed_list"),
            KeyOrigin::MyNetworkRootNotFollowingViewer => {
                f.write_str("root:MyNetwork:not_followed")
            }
            KeyOrigin::CommunityPost => f.write_str("community_post"),
            KeyOrigin::ModeratedCommunity => f.write_str("moderated_community"),
            KeyOrigin::TweetCommunity => f.write_str("tweet_community"),
            KeyOrigin::TrustedFriendsList => f.write_str("trusted_friends_list"),
            KeyOrigin::NarrowcastPlace => f.write_str("narrowcast_place"),
        }
    }
}
