use super::{Feature, Leaf, Meaning};
use crate::hydration::{Hydrator, Hydrators};
use crate::models::{AuthorFeatures, AuthorLabel, AuthorLabelSet};
use crate::rules::context::At;
use crate::rules::rule_spec::rule::RuleCond;
use crate::rules::rule_spec::Predicate;

pub(in crate::rules) enum AuthorSafety {}

impl Feature for AuthorSafety {
    const REQUIRED: Hydrators = Hydrators::of(Hydrator::AuthorSafety);
    type Value<'a> = AuthorFeatures;

    #[inline]
    fn read<'a>(at: At<'_, 'a>) -> AuthorFeatures {
        at.author().features()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, strum::IntoStaticStr)]
#[strum(serialize_all = "snake_case")]
pub enum AuthorSafetyLeaf {
    AuthorIsSuspended,
    AuthorIsDeactivated,
    AuthorIsErased,
    AuthorIsOffboarded,
    AuthorIsProtected,
    AuthorIsNsfwUser,
    AuthorIsNsfwAdmin,
}

impl Leaf for AuthorSafetyLeaf {
    type Feature = AuthorSafety;

    #[inline]
    fn meaning<'a, M: Meaning<'a, AuthorSafety>>(self, m: M) -> M::Out {
        match self {
            Self::AuthorIsSuspended => m.run(|author, _| author.is_suspended),
            Self::AuthorIsDeactivated => m.run(|author, _| author.is_deactivated),
            Self::AuthorIsErased => m.run(|author, _| author.is_erased),
            Self::AuthorIsOffboarded => m.run(|author, _| author.is_offboarded),
            Self::AuthorIsProtected => m.run(|author, _| author.is_protected),
            Self::AuthorIsNsfwUser => m.run(|author, _| author.is_nsfw_user),
            Self::AuthorIsNsfwAdmin => m.run(|author, _| author.is_nsfw_admin),
        }
    }
}

impl AuthorSafetyLeaf {
    pub(in crate::rules) const fn leaf(self) -> RuleCond {
        RuleCond::Leaf(Predicate::AuthorSafety(self))
    }
}

pub(in crate::rules) enum AuthorLabels {}

impl Feature for AuthorLabels {
    const REQUIRED: Hydrators = Hydrators::of(Hydrator::AuthorLabels);
    type Value<'a> = AuthorLabelSet;

    #[inline]
    fn read<'a>(at: At<'_, 'a>) -> AuthorLabelSet {
        at.author().labels
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, strum::IntoStaticStr)]
#[strum(serialize_all = "snake_case")]
pub enum AuthorLabelsLeaf {
    AuthorHasLabel(AuthorLabel),
}

impl Leaf for AuthorLabelsLeaf {
    type Feature = AuthorLabels;

    #[inline]
    fn meaning<'a, M: Meaning<'a, AuthorLabels>>(self, m: M) -> M::Out {
        match self {
            Self::AuthorHasLabel(label) => m.run_with(label, |labels, _| labels.has_label(label)),
        }
    }
}

impl AuthorLabelsLeaf {
    pub(in crate::rules) const fn leaf(self) -> RuleCond {
        RuleCond::Leaf(Predicate::AuthorLabels(self))
    }
}
