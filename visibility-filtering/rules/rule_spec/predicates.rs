use super::features::Feature;
use super::features::{
    truth, ArticleStageLeaf, AuthorLabelsLeaf, AuthorSafetyLeaf, CommunityModeratorLeaf,
    CommunityViewerRemovedLeaf, CompoundLeaf, EdgeLeaf, EvaluateFeature, Leaf, PostFactsLeaf,
    PostModerationLeaf, ReplyControlLeaf, RequestLeaf, SourceTweetIdLeaf, Spelling, TweetLeaf,
    TweetSafetyLabelsLeaf, ViewerAccountLeaf,
};
use super::*;

pub(in crate::rules) fn spelled_label(label: SafetyLabelType) -> String {
    label_meta(label).map_or_else(|| format!("{label:?}"), |meta| meta.ident.to_owned())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Predicate {
    Request(RequestLeaf),
    PostFacts(PostFactsLeaf),
    SourceTweetId(SourceTweetIdLeaf),
    Tweet(TweetLeaf),
    TweetSafetyLabels(TweetSafetyLabelsLeaf),
    ReplyControl(ReplyControlLeaf),
    PostModeration(PostModerationLeaf),
    ArticleStage(ArticleStageLeaf),
    CommunityModerator(CommunityModeratorLeaf),
    CommunityViewerRemoved(CommunityViewerRemovedLeaf),
    ViewerAccount(ViewerAccountLeaf),
    AuthorSafety(AuthorSafetyLeaf),
    AuthorLabels(AuthorLabelsLeaf),
    Edge(EdgeLeaf),
    Compound(CompoundLeaf),
}

trait PredicateVisitor {
    type Out;
    fn leaf<L: Leaf>(self, leaf: L) -> Self::Out;
    fn edge(self, leaf: EdgeLeaf) -> Self::Out;
    fn compound(self, leaf: CompoundLeaf) -> Self::Out;
}

impl Predicate {
    fn visit<V: PredicateVisitor>(self, visitor: V) -> V::Out {
        match self {
            Self::Request(leaf) => visitor.leaf(leaf),
            Self::PostFacts(leaf) => visitor.leaf(leaf),
            Self::SourceTweetId(leaf) => visitor.leaf(leaf),
            Self::Tweet(leaf) => visitor.leaf(leaf),
            Self::TweetSafetyLabels(leaf) => visitor.leaf(leaf),
            Self::ReplyControl(leaf) => visitor.leaf(leaf),
            Self::PostModeration(leaf) => visitor.leaf(leaf),
            Self::ArticleStage(leaf) => visitor.leaf(leaf),
            Self::CommunityModerator(leaf) => visitor.leaf(leaf),
            Self::CommunityViewerRemoved(leaf) => visitor.leaf(leaf),
            Self::ViewerAccount(leaf) => visitor.leaf(leaf),
            Self::AuthorSafety(leaf) => visitor.leaf(leaf),
            Self::AuthorLabels(leaf) => visitor.leaf(leaf),
            Self::Edge(leaf) => visitor.edge(leaf),
            Self::Compound(leaf) => visitor.compound(leaf),
        }
    }

    pub fn render(self, truth: Truth) -> String {
        self.visit(Sentence(truth))
            .unwrap_or_else(|| format!("{}: {truth:?}", self.spelled()))
    }

    pub(in crate::rules) fn spelled(self) -> String {
        self.visit(Spelled)
    }

    pub(in crate::rules) fn requirements(self) -> Hydrators {
        self.visit(Requirements)
    }

    #[cfg(test)]
    pub(super) fn holds(self, context: &RuleContext<'_>) -> bool {
        self.truth(context).resolves_true()
    }

    pub(in crate::rules) fn truth(self, context: &RuleContext<'_>) -> Truth {
        self.visit(TruthOf(context))
    }
}

struct Spelled;

impl PredicateVisitor for Spelled {
    type Out = String;

    fn leaf<L: Leaf>(self, leaf: L) -> String {
        let name: &str = leaf.into();
        match leaf.meaning(Spelling) {
            Some(payload) => format!("{name}({payload})"),
            None => name.to_owned(),
        }
    }

    fn edge(self, leaf: EdgeLeaf) -> String {
        <&str>::from(leaf).to_owned()
    }

    fn compound(self, leaf: CompoundLeaf) -> String {
        <&str>::from(leaf).to_owned()
    }
}

struct Sentence(Truth);

impl PredicateVisitor for Sentence {
    type Out = Option<String>;

    fn leaf<L: Leaf>(self, leaf: L) -> Option<String> {
        leaf.sentence(self.0)
    }

    fn edge(self, leaf: EdgeLeaf) -> Option<String> {
        leaf.sentence(self.0)
    }

    fn compound(self, _: CompoundLeaf) -> Option<String> {
        None
    }
}

struct Requirements;

impl PredicateVisitor for Requirements {
    type Out = Hydrators;

    fn leaf<L: Leaf>(self, _: L) -> Hydrators {
        <L::Feature as Feature>::REQUIRED
    }

    fn edge(self, leaf: EdgeLeaf) -> Hydrators {
        Hydrators::of(leaf.node())
    }

    fn compound(self, leaf: CompoundLeaf) -> Hydrators {
        leaf.interpret(&inputs::Required)
    }
}

struct TruthOf<'c, 'a>(&'c RuleContext<'a>);

impl PredicateVisitor for TruthOf<'_, '_> {
    type Out = Truth;

    fn leaf<L: Leaf>(self, leaf: L) -> Truth {
        leaf.meaning(EvaluateFeature(self.0))
    }

    fn edge(self, leaf: EdgeLeaf) -> Truth {
        let node = leaf.node();
        truth(
            self.0.acquire_edge(node),
            self.0.failed().intersection(Hydrators::of(node)),
        )
    }

    fn compound(self, leaf: CompoundLeaf) -> Truth {
        let inputs::Acquired { value, failed } = leaf.interpret(&inputs::EvaluateInputs(self.0));
        truth(value, failed)
    }
}
