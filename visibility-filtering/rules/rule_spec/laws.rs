use super::conditions::{
    VIEWER_IS_NEITHER_ROOT_AUTHOR_NOR_INVITED,
    VIEWER_IS_NOT_AGE_VERIFIED_IN_AGE_VERIFICATION_COUNTRIES,
    VIEWER_IS_NOT_AGE_VERIFIED_IN_TOMBSTONE_COUNTRIES,
};
use super::features::{CompoundLeaf, ReplyControlLeaf, RequestLeaf};
use super::rule::{Rule, RuleCond};
use super::*;
use crate::hydration::Hydrator;
use crate::models::{ClientCapability, VerifyBlurSupport, ViewerFeatures};
use crate::rules::fixtures::{candidate, viewer, VIEWER_ID};
use crate::rules::golden_corpus::pairs;
use crate::rules::test_context;
use crate::rules::SafetyLevel;
use std::iter::once;
use strum::VariantArray;

fn truths() -> Vec<Truth> {
    let a = Hydrators::of(Hydrator::Follows);
    let b = Hydrators::of(Hydrator::Blocks);
    let mut truths = vec![Truth::True, Truth::False];
    for default in [false, true] {
        for failed in [a, b, a.union(b)] {
            truths.push(Truth::Unknown { default, failed });
        }
    }
    truths
}

fn wired_rules() -> Vec<&'static Rule> {
    let mut rules: Vec<&'static Rule> = Vec::new();
    for &level in SafetyLevel::VARIANTS {
        for &rule in level.spec().rules {
            if !rules.contains(&rule) {
                rules.push(rule);
            }
        }
    }
    rules
}

fn wired_leaves() -> Vec<Predicate> {
    let mut leaves = Vec::new();
    for rule in wired_rules() {
        let mut read = Vec::new();
        rule.audience.condition().leaves(&mut read);
        rule.when.leaves(&mut read);
        for leaf in read {
            if !leaves.contains(&leaf) {
                leaves.push(leaf);
            }
        }
    }
    leaves
}

#[test]
fn kleene_algebra() {
    let truths = truths();
    for &a in &truths {
        assert_eq!(a.and(Truth::True), a, "{a:?}");
        assert_eq!(a.or(Truth::False), a, "{a:?}");
        assert_eq!(a.and(Truth::False), Truth::False, "{a:?}");
        assert_eq!(a.or(Truth::True), Truth::True, "{a:?}");
        assert_eq!(!!a, a, "{a:?}");
        assert_eq!((!a).resolves_true(), !a.resolves_true(), "{a:?}");
        for &b in &truths {
            assert_eq!(a.and(b), b.and(a), "{a:?} {b:?}");
            assert_eq!(a.or(b), b.or(a), "{a:?} {b:?}");
            assert_eq!(!a.and(b), (!a).or(!b), "{a:?} {b:?}");
            assert_eq!(!a.or(b), (!a).and(!b), "{a:?} {b:?}");
            assert_eq!(
                a.and(b).resolves_true(),
                a.resolves_true() && b.resolves_true(),
                "{a:?} {b:?}"
            );
            assert_eq!(
                a.or(b).resolves_true(),
                a.resolves_true() || b.resolves_true(),
                "{a:?} {b:?}"
            );
            for &c in &truths {
                assert_eq!(a.and(b).and(c), a.and(b.and(c)), "{a:?} {b:?} {c:?}");
                assert_eq!(a.or(b).or(c), a.or(b.or(c)), "{a:?} {b:?} {c:?}");
            }
        }
    }
}

#[test]
fn a_failed_read_is_local() {
    let viewer = viewer(VIEWER_ID);
    let fetched = candidate().build();
    for leaf in wired_leaves() {
        let clean = leaf.truth(&test_context(&viewer, &fetched));
        for &hydrator in Hydrator::VARIANTS {
            let mut failed = fetched.clone();
            failed.failed = Hydrators::of(hydrator);
            let expected = if leaf.requirements().contains(hydrator) {
                Truth::Unknown {
                    default: clean.resolves_true(),
                    failed: Hydrators::of(hydrator),
                }
            } else {
                clean
            };
            assert_eq!(
                leaf.truth(&test_context(&viewer, &failed)),
                expected,
                "{leaf:?} with {hydrator:?} failed"
            );
        }
    }
}

const ACCEPTED_BEST_EFFORT_READS: &[(&str, Hydrator, &str)] = &[
    (
        "protected_author/drop",
        Hydrator::Follows,
        "the default drops the post for real followers",
    ),
    (
        "abusive_high_recall_user_label/drop/unspecified",
        Hydrator::Follows,
        "the default drops the post for real followers",
    ),
    (
        "do_not_amplify_user_label/drop/unspecified",
        Hydrator::Follows,
        "the default drops the post for real followers",
    ),
    (
        "fosnr_abuse_insults_non_follower/drop/undesirable",
        Hydrator::Follows,
        "the default drops the post for real followers",
    ),
    (
        "fosnr_abuse_insults_follower/soft_intervention/abuse",
        Hydrator::Follows,
        "the non-follower drop on the same label fires instead",
    ),
    (
        "author_blocks_viewer_exclusive_content/drop/unspecified",
        Hydrator::BlockedByAuthor,
        "only a subscriber the author blocked sees the exclusive post",
    ),
];

#[test]
fn a_rule_reading_a_best_effort_node_limits_engagement_or_is_accepted() {
    let mut unaccepted = Vec::new();
    let mut accepted = Vec::new();
    for rule in wired_rules() {
        let name = rule.name();
        let limits = matches!(rule.action.spec(), ActionSpec::LimitedEngagement(_));
        for node in rule
            .requirements()
            .iter()
            .filter(|node| node.is_best_effort())
        {
            match ACCEPTED_BEST_EFFORT_READS
                .iter()
                .find(|&&(by, accepted, _)| by == name && accepted == node)
            {
                Some(&(by, node, _)) => accepted.push((by, node)),
                None if limits => {}
                None => unaccepted.push((name.clone(), node)),
            }
        }
    }
    assert!(unaccepted.is_empty(), "{unaccepted:?}");
    for &(by, node, _) in ACCEPTED_BEST_EFFORT_READS {
        assert!(
            accepted.contains(&(by, node)),
            "{by} no longer reads {node:?}"
        );
    }
}

fn holds(condition: RuleCond, context: &RuleContext<'_>) -> bool {
    match condition {
        RuleCond::Leaf(leaf) => leaf.holds(context),
        RuleCond::All(children) => children.iter().all(|&child| holds(child, context)),
        RuleCond::Any(children) => children.iter().any(|&child| holds(child, context)),
        RuleCond::Not(child) => !holds(*child, context),
        RuleCond::Named(named) => holds(named.condition, context),
    }
}

#[test]
fn the_rule_evaluator_is_faithful() {
    let rules = wired_rules();
    let failures: Vec<Hydrators> = once(Hydrators::empty())
        .chain(
            Hydrator::VARIANTS
                .iter()
                .map(|&hydrator| Hydrators::of(hydrator)),
        )
        .collect();
    for (viewer, fetched) in pairs() {
        for &failed in &failures {
            let mut candidate = fetched.clone();
            candidate.failed = failed;
            let context = test_context(&viewer, &candidate);
            for &rule in &rules {
                if rule.requirements().intersection(failed) != failed {
                    continue;
                }
                let (truth, ()) =
                    observed::evaluate_rule(rule, &context, &mut observed::NoTrace, None);
                let (total, _) = observed::evaluate_rule(
                    rule,
                    &context,
                    &mut observed::NodeTracer::default(),
                    None,
                );
                assert_eq!(truth, total, "{} with {failed:?} failed", rule.name());
                assert_eq!(
                    truth.resolves_true(),
                    holds(rule.audience.condition(), &context) && holds(rule.when, &context),
                    "{} with {failed:?} failed",
                    rule.name()
                );
            }
        }
    }
}

fn readers_through(definitions: &[RuleCond], leaves: &[Predicate]) -> usize {
    fn walk(
        condition: RuleCond,
        definitions: &[RuleCond],
        negated: bool,
        outside: &mut Vec<Predicate>,
        uses: &mut Vec<bool>,
    ) {
        match condition {
            condition if definitions.contains(&condition) => uses.push(negated),
            RuleCond::Leaf(leaf) => outside.push(leaf),
            RuleCond::All(children) | RuleCond::Any(children) => {
                for &child in children {
                    walk(child, definitions, negated, outside, uses);
                }
            }
            RuleCond::Not(child) => walk(*child, definitions, !negated, outside, uses),
            RuleCond::Named(named) => walk(named.condition, definitions, negated, outside, uses),
        }
    }
    let mut readers = 0;
    for rule in wired_rules() {
        let (mut outside, mut uses) = (Vec::new(), Vec::new());
        for root in [rule.audience.condition(), rule.when] {
            walk(root, definitions, false, &mut outside, &mut uses);
        }
        for leaf in leaves {
            assert!(
                !outside.contains(leaf),
                "{} reads {leaf:?} outside its named definition",
                rule.name()
            );
        }
        if !uses.is_empty() {
            assert_eq!(
                uses,
                [false],
                "{} must read one definition once, positively",
                rule.name()
            );
            readers += 1;
        }
    }
    readers
}

#[test]
fn the_age_gate_is_whole() {
    let in_list = |list| Predicate::Request(RequestLeaf::RequestCountryIn(list));
    assert_ne!(
        readers_through(
            &[VIEWER_IS_NOT_AGE_VERIFIED_IN_AGE_VERIFICATION_COUNTRIES],
            &[in_list(CountryList::AgeVerification)]
        ),
        0
    );
    assert_ne!(
        readers_through(
            &[VIEWER_IS_NOT_AGE_VERIFIED_IN_TOMBSTONE_COUNTRIES],
            &[in_list(CountryList::Tombstone)]
        ),
        0
    );
    assert_ne!(
        readers_through(
            &[
                VIEWER_IS_NOT_AGE_VERIFIED_IN_AGE_VERIFICATION_COUNTRIES,
                VIEWER_IS_NOT_AGE_VERIFIED_IN_TOMBSTONE_COUNTRIES
            ],
            &[Predicate::Compound(CompoundLeaf::ViewerAgeVerified)]
        ),
        0
    );
}

#[test]
fn the_conversation_root_exemption_is_whole() {
    assert_ne!(
        readers_through(
            &[VIEWER_IS_NEITHER_ROOT_AUTHOR_NOR_INVITED],
            &[
                Predicate::ReplyControl(ReplyControlLeaf::ViewerIsConversationRootAuthor),
                Predicate::ReplyControl(ReplyControlLeaf::ViewerIsInvitedToConversation),
            ],
        ),
        0
    );
}

#[test]
fn every_client_capability_is_live() {
    let base = ClientCapability::default();
    let ClientCapability {
        verify_blur_support,
        modern_blur,
        stale_tweet_limits,
        community_viewer_removed_limits,
        gore_blur_ignores_settings,
        fosnr_rules,
        fosnr_fallback_drops,
    } = base;
    let blur_supports = once(None)
        .chain(VerifyBlurSupport::VARIANTS.iter().copied().map(Some))
        .filter(|&support| support != verify_blur_support)
        .map(|support| ClientCapability {
            verify_blur_support: support,
            ..base
        })
        .collect();
    let toggled: [(&str, Vec<ClientCapability>); 7] = [
        ("verify_blur_support", blur_supports),
        (
            "modern_blur",
            vec![ClientCapability {
                modern_blur: !modern_blur,
                ..base
            }],
        ),
        (
            "stale_tweet_limits",
            vec![ClientCapability {
                stale_tweet_limits: !stale_tweet_limits,
                ..base
            }],
        ),
        (
            "community_viewer_removed_limits",
            vec![ClientCapability {
                community_viewer_removed_limits: !community_viewer_removed_limits,
                ..base
            }],
        ),
        (
            "gore_blur_ignores_settings",
            vec![ClientCapability {
                gore_blur_ignores_settings: !gore_blur_ignores_settings,
                ..base
            }],
        ),
        (
            "fosnr_rules",
            vec![ClientCapability {
                fosnr_rules: !fosnr_rules,
                ..base
            }],
        ),
        (
            "fosnr_fallback_drops",
            vec![ClientCapability {
                fosnr_fallback_drops: !fosnr_fallback_drops,
                ..base
            }],
        ),
    ];
    let leaves = wired_leaves();
    let fetched = candidate().build();
    let holds = |client_capability: ClientCapability, leaf: Predicate| {
        let features = ViewerFeatures {
            client_capability,
            ..viewer(VIEWER_ID)
        };
        leaf.holds(&test_context(&features, &fetched))
    };
    for (field, capabilities) in toggled {
        let live = capabilities.iter().any(|&capability| {
            leaves
                .iter()
                .any(|&leaf| holds(capability, leaf) != holds(base, leaf))
        });
        assert!(
            live,
            "no wired leaf reads ClientCapability::{field}: wire a rule that reads it, or delete it"
        );
    }
}
