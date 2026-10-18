use crate::hydration::sources::{exchange, Exchange, Scope, Visit};
use strum::VariantArray;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum MissPolicy {
    Unresolves(Subject),
    FailsNode,
    ReadsNoEdge,
    ReadsNotFound,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Subject {
    Tweet,
    Author,
}

pub(super) struct Row {
    pub(super) miss: MissPolicy,
    pub(super) batch: Batch,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Batch {
    Unrecorded,
    Keys,
    Requested,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, VariantArray)]
pub(crate) enum Source {
    TesPureCoreData,
    TesConversationRoot,
    TesTweet,
    TesConversationControl,
    SafetyLabels,
    GizmoduckViewer,
    GizmoduckAuthor,
    Flock,
    ViewerCountry,
    Wingman,
    CommunityModeration,
    CommunityModerator,
    CommunityViewerRemoved,
    ArticleLifecycle,
    ArticleMediaEntities,
    ArticleCoverMedia,
    TrustedFriends,
    UserLocation,
}

impl Source {
    pub(super) const fn row(self) -> Row {
        match self {
            Source::TesPureCoreData => Row {
                miss: MissPolicy::Unresolves(Subject::Tweet),
                batch: Batch::Keys,
            },
            Source::TesConversationRoot => Row {
                miss: MissPolicy::FailsNode,
                batch: Batch::Keys,
            },
            Source::TesTweet => Row {
                miss: MissPolicy::Unresolves(Subject::Tweet),
                batch: Batch::Unrecorded,
            },
            Source::TesConversationControl => Row {
                miss: MissPolicy::FailsNode,
                batch: Batch::Keys,
            },
            Source::SafetyLabels => Row {
                miss: MissPolicy::FailsNode,
                batch: Batch::Requested,
            },
            Source::GizmoduckViewer => Row {
                miss: MissPolicy::FailsNode,
                batch: Batch::Unrecorded,
            },
            Source::GizmoduckAuthor => Row {
                miss: MissPolicy::Unresolves(Subject::Author),
                batch: Batch::Keys,
            },
            Source::Flock => Row {
                miss: MissPolicy::FailsNode,
                batch: Batch::Requested,
            },
            Source::ViewerCountry => Row {
                miss: MissPolicy::FailsNode,
                batch: Batch::Unrecorded,
            },
            Source::Wingman => Row {
                miss: MissPolicy::ReadsNoEdge,
                batch: Batch::Requested,
            },
            Source::CommunityModeration => Row {
                miss: MissPolicy::FailsNode,
                batch: Batch::Keys,
            },
            Source::CommunityModerator => Row {
                miss: MissPolicy::FailsNode,
                batch: Batch::Keys,
            },
            Source::CommunityViewerRemoved => Row {
                miss: MissPolicy::FailsNode,
                batch: Batch::Keys,
            },
            Source::ArticleLifecycle => Row {
                miss: MissPolicy::FailsNode,
                batch: Batch::Keys,
            },
            Source::ArticleMediaEntities => Row {
                miss: MissPolicy::ReadsNotFound,
                batch: Batch::Keys,
            },
            Source::ArticleCoverMedia => Row {
                miss: MissPolicy::ReadsNotFound,
                batch: Batch::Keys,
            },
            Source::TrustedFriends => Row {
                miss: MissPolicy::ReadsNoEdge,
                batch: Batch::Keys,
            },
            Source::UserLocation => Row {
                miss: MissPolicy::FailsNode,
                batch: Batch::Unrecorded,
            },
        }
    }

    pub(super) const fn miss_policy(self) -> MissPolicy {
        self.row().miss
    }

    pub(super) const fn scope(self) -> Scope {
        match self {
            Source::TesPureCoreData => <exchange::TesPureCoreData as Exchange>::SCOPE,
            Source::TesConversationRoot => <exchange::TesConversationRoot as Exchange>::SCOPE,
            Source::TesTweet => <exchange::TesTweet as Exchange>::SCOPE,
            Source::TesConversationControl => <exchange::TesConversationControl as Exchange>::SCOPE,
            Source::SafetyLabels => <exchange::SafetyLabels as Exchange>::SCOPE,
            Source::GizmoduckViewer => <exchange::GizmoduckViewer as Exchange>::SCOPE,
            Source::GizmoduckAuthor => <exchange::GizmoduckAuthor as Exchange>::SCOPE,
            Source::Flock => <exchange::Flock as Exchange>::SCOPE,
            Source::ViewerCountry => <exchange::ViewerCountry as Exchange>::SCOPE,
            Source::Wingman => <exchange::Wingman as Exchange>::SCOPE,
            Source::CommunityModeration => <exchange::CommunityModeration as Exchange>::SCOPE,
            Source::CommunityModerator => <exchange::CommunityModerator as Exchange>::SCOPE,
            Source::CommunityViewerRemoved => <exchange::CommunityViewerRemoved as Exchange>::SCOPE,
            Source::ArticleLifecycle => <exchange::ArticleLifecycle as Exchange>::SCOPE,
            Source::ArticleMediaEntities => <exchange::ArticleMediaEntities as Exchange>::SCOPE,
            Source::ArticleCoverMedia => <exchange::ArticleCoverMedia as Exchange>::SCOPE,
            Source::TrustedFriends => <exchange::TrustedFriends as Exchange>::SCOPE,
            Source::UserLocation => <exchange::UserLocation as Exchange>::SCOPE,
        }
    }

    pub(crate) fn visit(self, visitor: &mut impl Visit) {
        match self {
            Source::TesPureCoreData => visitor.exchange::<exchange::TesPureCoreData>(),
            Source::TesConversationRoot => visitor.exchange::<exchange::TesConversationRoot>(),
            Source::TesTweet => visitor.exchange::<exchange::TesTweet>(),
            Source::TesConversationControl => {
                visitor.exchange::<exchange::TesConversationControl>()
            }
            Source::SafetyLabels => visitor.exchange::<exchange::SafetyLabels>(),
            Source::GizmoduckViewer => visitor.exchange::<exchange::GizmoduckViewer>(),
            Source::GizmoduckAuthor => visitor.exchange::<exchange::GizmoduckAuthor>(),
            Source::Flock => visitor.exchange::<exchange::Flock>(),
            Source::ViewerCountry => visitor.exchange::<exchange::ViewerCountry>(),
            Source::Wingman => visitor.exchange::<exchange::Wingman>(),
            Source::CommunityModeration => visitor.exchange::<exchange::CommunityModeration>(),
            Source::CommunityModerator => visitor.exchange::<exchange::CommunityModerator>(),
            Source::CommunityViewerRemoved => {
                visitor.exchange::<exchange::CommunityViewerRemoved>()
            }
            Source::ArticleLifecycle => visitor.exchange::<exchange::ArticleLifecycle>(),
            Source::ArticleMediaEntities => visitor.exchange::<exchange::ArticleMediaEntities>(),
            Source::ArticleCoverMedia => visitor.exchange::<exchange::ArticleCoverMedia>(),
            Source::TrustedFriends => visitor.exchange::<exchange::TrustedFriends>(),
            Source::UserLocation => visitor.exchange::<exchange::UserLocation>(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::any::type_name;

    struct Visited(Vec<(&'static str, Scope)>);

    impl Visit for Visited {
        fn exchange<X: Exchange>(&mut self) {
            self.0.push((type_name::<X>(), X::SCOPE));
        }
    }

    #[test]
    fn each_source_visits_the_exchange_named_after_it() {
        let (module, _) = type_name::<exchange::TesTweet>().rsplit_once("::").unwrap();
        for &source in Source::VARIANTS {
            let mut visited = Visited(Vec::new());
            source.visit(&mut visited);
            let [(exchange, scope)] = visited.0[..] else {
                panic!("{source:?} visits {:?}", visited.0);
            };
            assert_eq!(exchange, format!("{module}::{source:?}"));
            assert_eq!(source.scope(), scope, "{source:?}");
        }
    }
}
