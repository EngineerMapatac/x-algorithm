#[cfg(test)]
use super::leaves::TweetHasLabel;
use super::leaves::{ViewerFollowsAuthor, ViewerIsAuthor};
#[cfg(test)]
use super::{rule_name, spelled_label};
use super::{Predicate, RuleAction, RuleId};
use crate::hydration::Hydrators;
#[cfg(test)]
use crate::models::SafetyLabelType;
use std::ptr;
#[cfg(test)]
use std::slice;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::rules) struct Rule {
    pub(in crate::rules) id: RuleId,
    pub(in crate::rules) audience: Audience,
    pub(in crate::rules) when: RuleCond,
    pub(in crate::rules) action: RuleAction,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::rules) enum RuleCond {
    Leaf(Predicate),
    All(&'static [RuleCond]),
    Any(&'static [RuleCond]),
    Not(&'static RuleCond),
    Named(&'static NamedCond),
}

macro_rules! all {
    ($($operand:tt)*) => {
        $crate::rules::rule_spec::rule::RuleCond::All(
            &$crate::rules::rule_spec::rule::conds!(@ [] $($operand)*),
        )
    };
}

macro_rules! any {
    ($($operand:tt)*) => {
        $crate::rules::rule_spec::rule::RuleCond::Any(
            &$crate::rules::rule_spec::rule::conds!(@ [] $($operand)*),
        )
    };
}

macro_rules! conds {
    (@ [$($done:expr),*]) => { [$($done),*] };
    (@ [$($done:expr),*] all ! [$($list:tt)*] $(, $($rest:tt)*)?) => {
        $crate::rules::rule_spec::rule::conds!(
            @ [$($done,)* $crate::rules::rule_spec::rule::all![$($list)*]]
            $($($rest)*)?
        )
    };
    (@ [$($done:expr),*] any ! [$($list:tt)*] $(, $($rest:tt)*)?) => {
        $crate::rules::rule_spec::rule::conds!(
            @ [$($done,)* $crate::rules::rule_spec::rule::any![$($list)*]]
            $($($rest)*)?
        )
    };
    (@ [$($done:expr),*] ! all ! [$($list:tt)*] $(, $($rest:tt)*)?) => {
        $crate::rules::rule_spec::rule::conds!(
            @ [$($done,)* $crate::rules::rule_spec::rule::RuleCond::Not(
                &$crate::rules::rule_spec::rule::all![$($list)*],
            )]
            $($($rest)*)?
        )
    };
    (@ [$($done:expr),*] ! any ! [$($list:tt)*] $(, $($rest:tt)*)?) => {
        $crate::rules::rule_spec::rule::conds!(
            @ [$($done,)* $crate::rules::rule_spec::rule::RuleCond::Not(
                &$crate::rules::rule_spec::rule::any![$($list)*],
            )]
            $($($rest)*)?
        )
    };
    (@ [$($done:expr),*] ! $operand:expr $(, $($rest:tt)*)?) => {
        $crate::rules::rule_spec::rule::conds!(
            @ [$($done,)* $crate::rules::rule_spec::rule::RuleCond::Not(&$operand.leaf())]
            $($($rest)*)?
        )
    };
    (@ [$($done:expr),*] $operand:expr $(, $($rest:tt)*)?) => {
        $crate::rules::rule_spec::rule::conds!(@ [$($done,)* $operand.leaf()] $($($rest)*)?)
    };
}

pub(in crate::rules) use {all, any, conds};

#[derive(Debug, PartialEq, Eq)]
pub(in crate::rules) struct NamedCond {
    pub(in crate::rules) name: &'static str,
    pub(in crate::rules) condition: RuleCond,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::rules) enum Audience {
    Everyone,
    NonAuthors,
    NonFollowers,
    OnlyAuthor,
}

impl Audience {
    pub(in crate::rules) fn condition(self) -> RuleCond {
        match self {
            Self::Everyone => all![],
            Self::NonAuthors => const { RuleCond::Not(&ViewerIsAuthor.leaf()) },
            Self::NonFollowers => const { all![!ViewerIsAuthor, !ViewerFollowsAuthor] },
            Self::OnlyAuthor => ViewerIsAuthor.leaf(),
        }
    }
}

impl Rule {
    pub(in crate::rules) fn requirements(&self) -> Hydrators {
        self.audience
            .condition()
            .requirements()
            .union(self.when.requirements())
            .union(self.action.spec().requirements())
    }
}

impl RuleCond {
    pub(in crate::rules) const fn leaf(self) -> Self {
        self
    }

    pub(in crate::rules) fn requirements(self) -> Hydrators {
        match self {
            Self::Leaf(leaf) => leaf.requirements(),
            Self::All(children) | Self::Any(children) => children
                .iter()
                .fold(Hydrators::empty(), |hydrators, child| {
                    hydrators.union(child.requirements())
                }),
            Self::Not(child) => child.requirements(),
            Self::Named(named) => named.condition.requirements(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::rules) enum DescriptionError {
    Cycle,
    ConflictingName(&'static str),
}

pub(in crate::rules) fn validate<'a>(
    rules: impl IntoIterator<Item = &'a Rule>,
) -> Result<(), DescriptionError> {
    enum Step {
        Enter(&'static RuleCond),
        Leave,
    }
    let mut named: Vec<&'static NamedCond> = Vec::new();
    let mut steps: Vec<Step> = Vec::new();
    let mut children = |condition: RuleCond, steps: &mut Vec<Step>| match condition {
        RuleCond::Leaf(_) => {}
        RuleCond::All(children) | RuleCond::Any(children) => {
            steps.extend(children.iter().rev().map(Step::Enter));
        }
        RuleCond::Not(child) => steps.push(Step::Enter(child)),
        RuleCond::Named(definition) => {
            named.push(definition);
            steps.push(Step::Enter(&definition.condition));
        }
    };
    let mut active: Vec<*const RuleCond> = Vec::new();
    let mut finished: Vec<*const RuleCond> = Vec::new();
    for rule in rules {
        for root in [rule.audience.condition(), rule.when] {
            children(root, &mut steps);
            while let Some(step) = steps.pop() {
                let Step::Enter(condition) = step else {
                    finished.extend(active.pop());
                    continue;
                };
                let at: *const RuleCond = condition;
                if active.contains(&at) {
                    return Err(DescriptionError::Cycle);
                }
                if !finished.contains(&at) {
                    active.push(at);
                    steps.push(Step::Leave);
                    children(*condition, &mut steps);
                }
            }
        }
    }
    for (index, definition) in named.iter().enumerate() {
        if named.iter().take(index).any(|other| {
            other.name == definition.name
                && !ptr::eq(*other, *definition)
                && other.condition != definition.condition
        }) {
            return Err(DescriptionError::ConflictingName(definition.name));
        }
    }
    Ok(())
}

#[cfg(test)]
impl RuleCond {
    pub(in crate::rules) fn conjuncts(&self) -> &[RuleCond] {
        match self {
            Self::All(children) => children,
            Self::Leaf(_) | Self::Any(_) | Self::Not(_) | Self::Named(_) => slice::from_ref(self),
        }
    }

    pub(in crate::rules) fn named(self, named: &mut Vec<&'static NamedCond>) {
        match self {
            Self::Leaf(_) => {}
            Self::All(children) | Self::Any(children) => {
                for child in children {
                    child.named(named);
                }
            }
            Self::Not(child) => child.named(named),
            Self::Named(definition) => {
                if !named.contains(&definition) {
                    named.push(definition);
                }
                definition.condition.named(named);
            }
        }
    }

    pub(in crate::rules) fn positive_tweet_labels(self) -> Vec<SafetyLabelType> {
        match self {
            Self::Leaf(Predicate::TweetSafetyLabels(TweetHasLabel(label))) => vec![label],
            Self::Leaf(_) | Self::Not(_) => Vec::new(),
            Self::All(children) | Self::Any(children) => children
                .iter()
                .flat_map(|child| child.positive_tweet_labels())
                .collect(),
            Self::Named(named) => named.condition.positive_tweet_labels(),
        }
    }

    pub(in crate::rules) fn own_tweet_label(self) -> Option<String> {
        match self {
            Self::Leaf(Predicate::TweetSafetyLabels(TweetHasLabel(label))) => {
                Some(spelled_label(label))
            }
            Self::Not(Self::Leaf(Predicate::TweetSafetyLabels(TweetHasLabel(label)))) => {
                Some(format!("not {}", spelled_label(*label)))
            }
            Self::Leaf(_) | Self::All(_) | Self::Any(_) | Self::Not(_) | Self::Named(_) => None,
        }
    }

    pub(in crate::rules) fn leaves(self, leaves: &mut Vec<Predicate>) {
        match self {
            Self::Leaf(leaf) => leaves.push(leaf),
            Self::All(children) | Self::Any(children) => {
                for child in children {
                    child.leaves(leaves);
                }
            }
            Self::Not(child) => child.leaves(leaves),
            Self::Named(named) => named.condition.leaves(leaves),
        }
    }

    pub(in crate::rules) fn spelled(self) -> String {
        self.spelled_with(|named| format!("{}={}", named.name, named.condition.spelled()))
    }

    pub(in crate::rules) fn spelled_with(self, named: fn(&NamedCond) -> String) -> String {
        let joined = |children: &[RuleCond], separator: &str| {
            children
                .iter()
                .map(|child| child.spelled_with(named))
                .collect::<Vec<_>>()
                .join(separator)
        };
        match self {
            Self::Leaf(leaf) => leaf.spelled(),
            Self::All(children) => format!("all[{}]", joined(children, " & ")),
            Self::Any(children) => format!("any[{}]", joined(children, " | ")),
            Self::Not(child) => format!("not({})", child.spelled_with(named)),
            Self::Named(definition) => named(definition),
        }
    }
}

#[cfg(test)]
impl Rule {
    pub(in crate::rules) fn name(&self) -> String {
        rule_name(self.id, &self.action)
    }

    pub(in crate::rules) fn authored(&self) -> String {
        format!(
            "{} | audience={:?} | when={} | {:?}",
            self.name(),
            self.audience,
            self.when.spelled(),
            self.action
        )
    }
}
