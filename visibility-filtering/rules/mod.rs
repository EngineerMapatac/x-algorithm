mod author_rules;
#[cfg(test)]
mod catalog;
mod context;
pub mod engine;
#[cfg(test)]
pub(crate) mod fixtures;
#[cfg(test)]
mod golden_corpus;
#[cfg(test)]
mod legacy_table;
mod levels;
mod rule_spec;
mod safety_level;
mod tweet_rules;

#[cfg(test)]
use crate::models::{PostCandidate, ViewerFeatures};
#[cfg(test)]
use crate::params::CountryLists;
use context::RuleContext;
pub use engine::{Effect, PolicyTrace, RowTrace, RuleEngine};
pub(crate) use rule_spec::observed::NoTrace;
pub use rule_spec::observed::{NodeKind, NodeTrace, Skip, Witness};
pub use rule_spec::{
    ArticleStageLeaf, AuthorLabelsLeaf, AuthorSafetyLeaf, CommunityModeratorLeaf,
    CommunityViewerRemovedLeaf, CompoundLeaf, Contribution, EdgeLeaf, PostFactsLeaf,
    PostModerationLeaf, Predicate, ReplyControlLeaf, RequestLeaf, SourceTweetIdLeaf, Truth,
    TweetLeaf, TweetSafetyLabelsLeaf, ViewerAccountLeaf,
};
pub use safety_level::SafetyLevel;

#[cfg(test)]
pub(crate) fn test_context<'a>(
    viewer: &'a ViewerFeatures,
    candidate: &'a PostCandidate,
) -> RuleContext<'a> {
    use std::sync::LazyLock;

    static COUNTRY_LISTS: LazyLock<CountryLists> = LazyLock::new(CountryLists::starting_at_default);
    RuleContext::new(viewer, candidate, &COUNTRY_LISTS)
}
