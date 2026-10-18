use super::article::{ArticleCoverMedia, ArticleMediaEntities};
use super::post::{PostFacts, ReplyControl};
use super::tweet::Tweet;
use super::viewer::{ViewerAccount, ViewerCountry, ViewerLabels};
use super::{EdgeFeature, In};
use crate::hydration::Hydrator;
use crate::models::region::allows_country;
use crate::rules::rule_spec::inputs::Interpreter;
use crate::rules::rule_spec::rule::RuleCond;
use crate::rules::rule_spec::Predicate;

type SuperFollowsExclusive = EdgeFeature<{ Hydrator::SuperFollowsExclusive as u8 }>;
type RootFollowsViewer = EdgeFeature<{ Hydrator::RootFollowsViewer as u8 }>;
type RootFollowsViewerSecondDegree = EdgeFeature<{ Hydrator::RootFollowsViewerSecondDegree as u8 }>;
type BlockedByReplyRoot = EdgeFeature<{ Hydrator::BlockedByReplyRoot as u8 }>;
type BlockedByConversationRoot = EdgeFeature<{ Hydrator::BlockedByConversationRoot as u8 }>;
type BlockedByConversationRootPost = EdgeFeature<{ Hydrator::BlockedByConversationRootPost as u8 }>;

#[derive(Clone, Copy, Debug, PartialEq, Eq, strum::IntoStaticStr)]
#[strum(serialize_all = "snake_case")]
pub enum CompoundLeaf {
    TweetHasMedia,
    TweetIsSupersededEdit,
    ViewerAgeVerified,
    ViewerSuperFollowsAuthor,
    ViewerIsInConversationRootAuthorNetwork,
    ViewerIsBlockedByConversationRootAuthor,
    ViewerIsInAllowedCountry,
}

impl CompoundLeaf {
    #[inline]
    pub(in crate::rules) fn interpret<'a, I: Interpreter<'a>>(self, i: &I) -> I::Output<bool> {
        match self {
            Self::TweetHasMedia => i.run(
                |(
                    In::<Tweet>(tweet),
                    In::<ArticleMediaEntities>(body),
                    In::<ArticleCoverMedia>(cover),
                )| { tweet.has_media() || body || cover },
            ),
            Self::TweetIsSupersededEdit => i.run(|(In::<PostFacts>(post), In::<Tweet>(tweet))| {
                tweet.is_superseded_edit(post.tweet_id())
            }),
            Self::ViewerAgeVerified => i.run(
                |(In::<ViewerAccount>(profile), In::<ViewerLabels>(age_verified_18))| {
                    age_verified_18 || profile.is_some_and(|profile| profile.has_idv_premium)
                },
            ),
            Self::ViewerSuperFollowsAuthor => i.run(
                |(In::<Tweet>(tweet), In::<SuperFollowsExclusive>(super_follows))| {
                    tweet.exclusive_conversation_author_id.is_some() && super_follows
                },
            ),
            Self::ViewerIsInConversationRootAuthorNetwork => i.run(
                |(In::<RootFollowsViewer>(first), In::<RootFollowsViewerSecondDegree>(second))| {
                    first || second
                },
            ),
            Self::ViewerIsBlockedByConversationRootAuthor => i.run(
                |(
                    In::<PostFacts>(post),
                    In::<BlockedByReplyRoot>(direct),
                    In::<BlockedByConversationRoot>(from_control),
                    In::<BlockedByConversationRootPost>(fetched),
                )| { direct || fetched || (from_control && post.is_reply()) },
            ),
            Self::ViewerIsInAllowedCountry => i.run(
                |(In::<ReplyControl>(control), In::<ViewerCountry>(country))| {
                    control.zip(country).is_some_and(|(control, country)| {
                        allows_country(&control.allowed_country_codes, country)
                    })
                },
            ),
        }
    }

    pub(in crate::rules) const fn leaf(self) -> RuleCond {
        RuleCond::Leaf(Predicate::Compound(self))
    }
}
