use super::inputs::Inputs;
use super::spelled_label;
use super::Truth;
use crate::hydration::{Hydrator, Hydrators};
use crate::models::{AuthorLabel, SafetyLabelType, VerifyBlurSupport};
use crate::params::CountryList;
use crate::rules::context::{At, CoreFacts};
use crate::rules::RuleContext;
use strum::VariantArray;
use xai_core_entities::entities::ConversationControlArm;

mod article;
mod author;
mod community;
mod compound;
mod edges;
mod post;
mod request;
mod tweet;
mod viewer;

pub use article::ArticleStageLeaf;
pub use author::{AuthorLabelsLeaf, AuthorSafetyLeaf};
pub use community::{CommunityModeratorLeaf, CommunityViewerRemovedLeaf, PostModerationLeaf};
pub use compound::CompoundLeaf;
pub use edges::EdgeLeaf;
pub use post::{PostFactsLeaf, ReplyControlLeaf, SourceTweetIdLeaf, TweetSafetyLabelsLeaf};
pub use request::RequestLeaf;
pub use tweet::TweetLeaf;
pub use viewer::ViewerAccountLeaf;

pub(in crate::rules) use post::TweetSafetyLabels;

pub(in crate::rules) mod leaves {
    pub(in crate::rules) use super::ArticleStageLeaf::*;
    pub(in crate::rules) use super::AuthorLabelsLeaf::*;
    pub(in crate::rules) use super::AuthorSafetyLeaf::*;
    pub(in crate::rules) use super::CommunityModeratorLeaf::*;
    pub(in crate::rules) use super::CommunityViewerRemovedLeaf::*;
    pub(in crate::rules) use super::CompoundLeaf::*;
    pub(in crate::rules) use super::EdgeLeaf::*;
    pub(in crate::rules) use super::PostFactsLeaf::*;
    pub(in crate::rules) use super::PostModerationLeaf::*;
    pub(in crate::rules) use super::ReplyControlLeaf::*;
    pub(in crate::rules) use super::RequestLeaf::*;
    pub(in crate::rules) use super::SourceTweetIdLeaf::*;
    pub(in crate::rules) use super::TweetLeaf::*;
    pub(in crate::rules) use super::TweetSafetyLabelsLeaf::*;
    pub(in crate::rules) use super::ViewerAccountLeaf::*;
}

pub(in crate::rules) trait Feature {
    const REQUIRED: Hydrators;
    type Value<'a>: Copy;
    fn read<'a>(at: At<'_, 'a>) -> Self::Value<'a>;
}

pub(in crate::rules) trait Leaf: Copy + Into<&'static str> {
    type Feature: Feature;
    fn meaning<'a, M: Meaning<'a, Self::Feature>>(self, meaning: M) -> M::Out;
    fn sentence(self, _truth: Truth) -> Option<String> {
        None
    }
}

pub(in crate::rules) trait Meaning<'a, F: Feature> {
    type Out;
    fn run(self, kernel: impl FnOnce(F::Value<'a>, CoreFacts<'a>) -> bool) -> Self::Out;
    fn run_with(
        self,
        payload: impl Payload,
        kernel: impl FnOnce(F::Value<'a>, CoreFacts<'a>) -> bool,
    ) -> Self::Out;
}

pub(in crate::rules) trait Payload {
    fn spelled(self) -> String;
}

impl Payload for SafetyLabelType {
    fn spelled(self) -> String {
        spelled_label(self)
    }
}

impl Payload for u64 {
    fn spelled(self) -> String {
        self.to_string()
    }
}

impl Payload for &'static str {
    fn spelled(self) -> String {
        self.to_owned()
    }
}

impl Payload for AuthorLabel {
    fn spelled(self) -> String {
        format!("{self:?}")
    }
}

impl Payload for CountryList {
    fn spelled(self) -> String {
        format!("{self:?}")
    }
}

impl Payload for ConversationControlArm {
    fn spelled(self) -> String {
        format!("{self:?}")
    }
}

impl Payload for VerifyBlurSupport {
    fn spelled(self) -> String {
        format!("{self:?}")
    }
}

pub(super) struct Spelling;

impl<'a, F: Feature> Meaning<'a, F> for Spelling {
    type Out = Option<String>;

    fn run(self, _: impl FnOnce(F::Value<'a>, CoreFacts<'a>) -> bool) -> Option<String> {
        None
    }

    fn run_with(
        self,
        payload: impl Payload,
        _: impl FnOnce(F::Value<'a>, CoreFacts<'a>) -> bool,
    ) -> Option<String> {
        Some(payload.spelled())
    }
}

pub(super) struct EvaluateFeature<'c, 'a>(pub(super) &'c RuleContext<'a>);

impl<'a, F: Feature> Meaning<'a, F> for EvaluateFeature<'_, 'a> {
    type Out = Truth;

    #[inline]
    fn run(self, kernel: impl FnOnce(F::Value<'a>, CoreFacts<'a>) -> bool) -> Truth {
        let context = self.0;
        truth(
            kernel(context.acquire::<F>(), context.facts()),
            context.failed().intersection(F::REQUIRED),
        )
    }

    #[inline]
    fn run_with(
        self,
        _: impl Payload,
        kernel: impl FnOnce(F::Value<'a>, CoreFacts<'a>) -> bool,
    ) -> Truth {
        Meaning::<F>::run(self, kernel)
    }
}

#[inline]
pub(super) fn truth(value: bool, failed: Hydrators) -> Truth {
    match (failed.is_empty(), value) {
        (false, default) => Truth::Unknown { default, failed },
        (true, true) => Truth::True,
        (true, false) => Truth::False,
    }
}

pub(in crate::rules) enum EdgeFeature<const N: u8> {}

impl<const N: u8> EdgeFeature<N> {
    #[expect(
        clippy::indexing_slicing,
        reason = "N is a Hydrator discriminant; an out-of-range N fails const evaluation"
    )]
    const NODE: Hydrator = <Hydrator as VariantArray>::VARIANTS[N as usize];
}

impl<const N: u8> Feature for EdgeFeature<N> {
    const REQUIRED: Hydrators = {
        assert!(
            Self::NODE.is_edge(),
            "EdgeFeature<N> names a node that is not an edge"
        );
        Hydrators::of(Self::NODE)
    };
    type Value<'a> = bool;

    #[inline]
    fn read<'a>(at: At<'_, 'a>) -> bool {
        at.edge(Self::NODE)
    }
}

pub(in crate::rules) struct In<'a, F: Feature>(pub(in crate::rules) F::Value<'a>);

impl<'a, F: Feature> Inputs<'a> for In<'a, F> {
    const REQUIRED: Hydrators = F::REQUIRED;

    #[inline]
    fn acquire(context: &RuleContext<'a>) -> Self {
        Self(context.acquire::<F>())
    }
}
