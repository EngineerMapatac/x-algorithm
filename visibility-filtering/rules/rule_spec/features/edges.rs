use crate::hydration::Hydrator;
use crate::rules::rule_spec::rule::RuleCond;
use crate::rules::rule_spec::{Predicate, Truth};

#[derive(Clone, Copy, Debug, PartialEq, Eq, strum::IntoStaticStr)]
#[strum(serialize_all = "snake_case")]
pub enum EdgeLeaf {
    ViewerFollowsAuthor,
    ViewerBlocksAuthor,
    ViewerMutesAuthor,
    ViewerMutesRetweetsFromAuthor,
    ViewerIsFollowedByConversationRootAuthor,
    ViewerSuperFollowsConversationRootAuthor,
    ViewerIsBlockedByAuthor,
    ViewerIsTrustedFriendsListMemberOrOwner,
    ViewerIsOutsideNarrowcastPlace,
}

impl EdgeLeaf {
    #[inline]
    pub(in crate::rules) const fn node(self) -> Hydrator {
        match self {
            Self::ViewerFollowsAuthor => Hydrator::Follows,
            Self::ViewerBlocksAuthor => Hydrator::Blocks,
            Self::ViewerMutesAuthor => Hydrator::Mutes,
            Self::ViewerMutesRetweetsFromAuthor => Hydrator::MuteRetweets,
            Self::ViewerIsFollowedByConversationRootAuthor => Hydrator::RootFollowsViewer,
            Self::ViewerSuperFollowsConversationRootAuthor => Hydrator::SuperFollowsRoot,
            Self::ViewerIsBlockedByAuthor => Hydrator::BlockedByAuthor,
            Self::ViewerIsTrustedFriendsListMemberOrOwner => Hydrator::TrustedFriends,
            Self::ViewerIsOutsideNarrowcastPlace => Hydrator::OutsideNarrowcastPlace,
        }
    }

    pub(in crate::rules) fn sentence(self, truth: Truth) -> Option<String> {
        (self == Self::ViewerFollowsAuthor).then(|| {
            match truth {
                Truth::True => "The viewer follows the author.",
                Truth::False => "The viewer does not follow the author.",
                Truth::Unknown { .. } => "Whether the viewer follows the author.",
            }
            .to_owned()
        })
    }

    pub(in crate::rules) const fn leaf(self) -> RuleCond {
        RuleCond::Leaf(Predicate::Edge(self))
    }
}
