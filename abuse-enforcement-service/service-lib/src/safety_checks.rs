use std::sync::LazyLock;

use cel::Program;
use xai_abuse_enforcement_types::action_control::{ActionControlRecord, FollowerSkipView};
pub use xai_abuse_enforcement_types::action_control::{
    SafetyCheckView, SafetyChecksByEntity, SafetyChecksStatus,
};

use crate::facts::{EntityType, Facts};
use crate::rules::{CompiledRules, EvalError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SafetyCheck {
    pub id: &'static str,
    pub reason: &'static str,
    pub when: &'static str,
}

pub const USER_CHECKS: &[SafetyCheck] = &[
    SafetyCheck {
        id: "user_in_allowlist",
        reason: "user_in_allowlist",
        when: "user_allowlist.is_allowlisted",
    },
    SafetyCheck {
        id: "user_not_found",
        reason: "user_not_found",
        when: "!user.present",
    },
    SafetyCheck {
        id: "high_follower_count",
        reason: "high_follower_count",
        when: "cred.follower_count >= 12.34",
    },
    SafetyCheck {
        id: "pagerank_skipped",
        reason: "pagerank_skipped",
        when: "cred.is_high || cred.score >= 12.34",
    },
];

pub const POST_CHECKS: &[SafetyCheck] = &[
    SafetyCheck {
        id: "post_in_allowlist",
        reason: "post_in_allowlist",
        when: "post_allowlist.is_allowlisted",
    },
    SafetyCheck {
        id: "user_in_allowlist",
        reason: "user_in_allowlist",
        when: "user_allowlist.is_allowlisted",
    },
    SafetyCheck {
        id: "user_not_found",
        reason: "user_not_found",
        when: "!user.present",
    },
    SafetyCheck {
        id: "high_follower_count",
        reason: "high_follower_count",
        when: "cred.follower_count >= 12.34 && !score.skip_author_credibility_prechecks",
    },
    SafetyCheck {
        id: "pagerank_skipped",
        reason: "pagerank_skipped",
        when: "(cred.is_high || cred.score >= 12.34) && !score.skip_author_credibility_prechecks",
    },
];

pub fn checks(entity: EntityType) -> &'static [SafetyCheck] {
    match entity {
        EntityType::User => USER_CHECKS,
        EntityType::Post => POST_CHECKS,
    }
}

fn compile(list: &'static [SafetyCheck]) -> Vec<Program> {
    list.iter()
        .map(|c| {
            Program::compile(c.when)
                .unwrap_or_else(|e| panic!("safety check {} does not compile: {e:?}", c.id))
        })
        .collect()
}

static USER_PROGRAMS: LazyLock<Vec<Program>> = LazyLock::new(|| compile(USER_CHECKS));
static POST_PROGRAMS: LazyLock<Vec<Program>> = LazyLock::new(|| compile(POST_CHECKS));

fn normalize(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub fn status(entity: EntityType, rules: &CompiledRules) -> SafetyChecksStatus {
    let list = checks(entity);
    let leading: Vec<_> = rules
        .leading_skips()
        .into_iter()
        .filter(|r| r.id != FOLLOWER_CEILING.id)
        .collect();
    let mut problems = Vec::new();
    let mut views = Vec::with_capacity(list.len());
    for (i, check) in list.iter().enumerate() {
        let status = match leading.iter().position(|r| r.id == check.id) {
            None => {
                problems.push(format!(
                    "the rules have no safety skip `{}` at the top (it was removed, renamed, or moved below a rule that acts)",
                    check.id
                ));
                "missing"
            }
            Some(at) => {
                let r = leading[at];
                if r.reason != check.reason || normalize(r.when) != normalize(check.when) {
                    problems.push(format!(
                        "safety skip `{}` changed: the rules say `{}` → `{}`, the service expects `{}` → `{}`",
                        check.id,
                        normalize(r.when),
                        r.reason,
                        check.when,
                        check.reason
                    ));
                    "changed"
                } else if at != i {
                    problems.push(format!(
                        "safety skip `{}` moved: position {} in the rules, {} expected",
                        check.id,
                        at + 1,
                        i + 1
                    ));
                    "moved"
                } else {
                    "ok"
                }
            }
        };
        views.push(SafetyCheckView {
            id: check.id.into(),
            reason: check.reason.into(),
            when: check.when.into(),
            status: status.into(),
        });
    }
    for r in &leading {
        if !list.iter().any(|c| c.id == r.id) {
            problems.push(format!(
                "the rules start with a safety skip `{}` the service doesn't know; add it to the service's list",
                r.id
            ));
        }
    }
    SafetyChecksStatus {
        ok: problems.is_empty(),
        problems,
        checks: views,
    }
}

pub const FOLLOWER_CEILING: SafetyCheck = SafetyCheck {
    id: "very_high_follower_count",
    reason: "very_high_follower_count",
    when: "cred.follower_count >= 12345.67",
};

static FOLLOWER_CEILING_COUNT: LazyLock<f64> =
    LazyLock::new(|| followers_in(FOLLOWER_CEILING.when));

pub(crate) const PRE_CEILING: &[&str] =
    &["post_in_allowlist", "user_in_allowlist", "user_not_found"];

pub fn ceiling_stops(facts: &Facts, page_driven: bool) -> bool {
    match facts.cred().follower_count {
        Some(n) => n as f64 >= *FOLLOWER_CEILING_COUNT,
        None => page_driven && facts.entity_type == EntityType::Post,
    }
}

const FOLLOWER_SKIP: &str = "high_follower_count";
const CREDIBILITY_SKIP: &str = "pagerank_skipped";

fn followers_in(when: &str) -> f64 {
    when.split(">=")
        .nth(1)
        .and_then(|rest| rest.split_whitespace().next())
        .and_then(|n| n.parse().ok())
        .unwrap_or_else(|| panic!("`{when}` is not `cred.follower_count >= N ...`"))
}

static DEFAULT_FOLLOWER_SKIP: LazyLock<f64> = LazyLock::new(|| {
    let user = USER_CHECKS.iter().find(|c| c.id == FOLLOWER_SKIP);
    let post = POST_CHECKS.iter().find(|c| c.id == FOLLOWER_SKIP);
    let n = followers_in(user.expect("the user list has a follower skip").when);
    assert_eq!(
        n,
        followers_in(post.expect("the post list has a follower skip").when),
        "the user and post follower skips start at the same count"
    );
    n
});

pub fn follower_skip_view() -> FollowerSkipView {
    FollowerSkipView {
        today_threshold: DEFAULT_FOLLOWER_SKIP.ceil() as u64,
        ceiling: follower_ceiling(),
    }
}

pub fn follower_ceiling() -> u64 {
    FOLLOWER_CEILING_COUNT.ceil() as u64
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct AuthorSkips {
    pub follower_threshold: Option<u32>,
    pub skip_high_credibility: Option<bool>,
}

impl AuthorSkips {
    pub fn of(record: &ActionControlRecord) -> Self {
        Self {
            follower_threshold: record.follower_threshold,
            skip_high_credibility: record.skip_high_credibility,
        }
    }

        pub fn follower_floor(self) -> Option<f64> {
        match self.follower_threshold {
            None => Some(DEFAULT_FOLLOWER_SKIP.ceil()),
            Some(0) => None,
            Some(n) => Some(f64::from(n)),
        }
    }

    pub fn skips_high_credibility(self) -> bool {
        self.skip_high_credibility != Some(false)
    }

            pub fn harsher_than(self, before: Self) -> Vec<String> {
        let mut why = Vec::new();
        let floor = |s: Self| s.follower_floor().unwrap_or(f64::INFINITY);
        if floor(self) > floor(before) {
            let show = |s: Self| {
                s.follower_floor()
                    .map_or("off".to_owned(), |n| format!("{}+", n.ceil() as u64))
            };
            why.push(format!(
                "skips fewer high-follower accounts ({} to {})",
                show(before),
                show(self)
            ));
        }
        if before.skips_high_credibility() && !self.skips_high_credibility() {
            why.push("stops skipping high-credibility accounts".to_owned());
        }
        why
    }
}

pub fn first_page_stop(
    entity: EntityType,
    facts: &Facts,
    skips: AuthorSkips,
) -> Result<Option<&'static SafetyCheck>, EvalError> {
    let listed = if skips == AuthorSkips::default() {
        first_stop(entity, facts)
    } else {
        first_stop_with(entity, facts, skips)
    };
    if let Ok(Some(check)) = &listed
        && PRE_CEILING.contains(&check.id)
    {
        return listed;
    }
    if ceiling_stops(facts, true) {
        return Ok(Some(&FOLLOWER_CEILING));
    }
    listed
}

pub fn first_stop(
    entity: EntityType,
    facts: &Facts,
) -> Result<Option<&'static SafetyCheck>, EvalError> {
    let (list, programs) = match entity {
        EntityType::User => (USER_CHECKS, &*USER_PROGRAMS),
        EntityType::Post => (POST_CHECKS, &*POST_PROGRAMS),
    };
    let pairs: Vec<(&str, &Program)> = list.iter().map(|c| c.id).zip(programs.iter()).collect();
    Ok(crate::rules::first_true(&pairs, facts)?.map(|i| &list[i]))
}

fn first_stop_with(
    entity: EntityType,
    facts: &Facts,
    skips: AuthorSkips,
) -> Result<Option<&'static SafetyCheck>, EvalError> {
    let (list, programs) = match entity {
        EntityType::User => (USER_CHECKS, &*USER_PROGRAMS),
        EntityType::Post => (POST_CHECKS, &*POST_PROGRAMS),
    };
    let listed =
        |range: std::ops::Range<usize>| -> Result<Option<&'static SafetyCheck>, EvalError> {
            let kept: Vec<usize> = range
                .filter(|&i| list[i].id != CREDIBILITY_SKIP || skips.skips_high_credibility())
                .collect();
            if kept.is_empty() {
                return Ok(None);
            }
            let pairs: Vec<(&str, &Program)> =
                kept.iter().map(|&i| (list[i].id, &programs[i])).collect();
            Ok(crate::rules::first_true(&pairs, facts)?.map(|k| &list[kept[k]]))
        };
    let Some(floor_at) = list.iter().position(|c| c.id == FOLLOWER_SKIP) else {
        return listed(0..list.len());
    };
    if let Some(stop) = listed(0..floor_at)? {
        return Ok(Some(stop));
    }
    let waived = entity == EntityType::Post && facts.score.skip_author_credibility_prechecks;
    let followers = facts.cred().follower_count.unwrap_or(0) as f64;
    if !waived
        && skips
            .follower_floor()
            .is_some_and(|floor| followers >= floor)
    {
        return Ok(Some(&list[floor_at]));
    }
    listed(floor_at + 1..list.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    const BAKED_USER: &str = include_str!("../rules/enforcement_user.yaml");
    const BAKED_POST: &str = include_str!("../rules/enforcement_post.yaml");

    fn compiled(yaml: &str) -> CompiledRules {
        CompiledRules::from_yaml(yaml).unwrap()
    }

    #[test]
    fn baked_rules_match_the_list() {
        let user = status(EntityType::User, &compiled(BAKED_USER));
        assert!(user.ok, "{:?}", user.problems);
        assert!(user.checks.iter().all(|c| c.status == "ok"));
        let post = status(EntityType::Post, &compiled(BAKED_POST));
        assert!(post.ok, "{:?}", post.problems);
        assert_eq!(post.checks.len(), 5);
    }

    #[test]
    fn every_check_compiles() {
        assert_eq!(USER_PROGRAMS.len(), USER_CHECKS.len());
        assert_eq!(POST_PROGRAMS.len(), POST_CHECKS.len());
        assert!(FOLLOWER_CEILING_COUNT.is_finite());
    }

        fn edited_user(edit: impl Fn(&str) -> String) -> SafetyChecksStatus {
        let yaml = edit(BAKED_USER);
        status(EntityType::User, &compiled(&yaml))
    }

    fn rule_block(yaml: &str, id: &str) -> (usize, usize) {
        let start = yaml.find(&format!("  - id: {id}\n")).unwrap();
        let end = yaml[start + 3..]
            .find("\n  - id: ")
            .map_or(yaml.len(), |e| start + 3 + e + 1);
        (start, end)
    }

    fn without(yaml: &str, id: &str) -> String {
        let (s, e) = rule_block(yaml, id);
        format!("{}{}", &yaml[..s], &yaml[e..])
    }

        fn in_block(yaml: &str, id: &str, from: &str, to: &str) -> String {
        let (s, e) = rule_block(yaml, id);
        let block = yaml[s..e].replacen(from, to, 1);
        assert_ne!(block, yaml[s..e], "`{from}` is not in rule {id}");
        format!("{}{}{}", &yaml[..s], block, &yaml[e..])
    }

    #[test]
    fn a_missing_check_is_uncertain() {
        let st = edited_user(|y| without(y, "high_follower_count"));
        assert!(!st.ok);
        assert_eq!(
            st.checks
                .iter()
                .find(|c| c.id == "high_follower_count")
                .unwrap()
                .status,
            "missing"
        );
    }

    #[test]
    fn a_renamed_check_is_uncertain() {
        let st = edited_user(|y| {
            y.replacen(
                "  - id: high_follower_count\n",
                "  - id: high_followers\n",
                1,
            )
        });
        assert!(!st.ok);
        assert!(
            st.problems
                .iter()
                .any(|p| p.contains("high_follower_count"))
        );
        assert!(st.problems.iter().any(|p| p.contains("high_followers")));
    }

    #[test]
    fn a_reordered_check_is_uncertain() {
        let st = edited_user(|y| {
            let (s, e) = rule_block(y, "high_follower_count");
            let block = y[s..e].to_owned();
            let rest = format!("{}{}", &y[..s], &y[e..]);
            let (_, after) = rule_block(&rest, "pagerank_skipped");
            format!("{}{}{}", &rest[..after], block, &rest[after..])
        });
        assert!(!st.ok);
        assert!(st.checks.iter().any(|c| c.status == "moved"), "{st:?}");
    }

    #[test]
    fn a_check_moved_below_an_act_rule_is_missing() {
        let st = edited_user(|y| {
            let (s, e) = rule_block(y, "pagerank_skipped");
            let block = y[s..e].to_owned();
            let rest = format!("{}{}", &y[..s], &y[e..]);
            let (_, after) = rule_block(&rest, "anchor_campaign_suspend");
            format!("{}{}{}", &rest[..after], block, &rest[after..])
        });
        assert!(!st.ok);
        assert_eq!(
            st.checks
                .iter()
                .find(|c| c.id == "pagerank_skipped")
                .unwrap()
                .status,
            "missing"
        );
    }

    #[test]
    fn a_changed_condition_or_reason_is_uncertain() {
        let st = edited_user(|y| in_block(y, "high_follower_count", " >= ", " > "));
        assert!(!st.ok);
        assert_eq!(
            st.checks
                .iter()
                .find(|c| c.id == "high_follower_count")
                .unwrap()
                .status,
            "changed"
        );
        let st = edited_user(|y| {
            y.replacen(
                "      reason: user_not_found\n",
                "      reason: missing_user\n",
                1,
            )
        });
        assert!(!st.ok);
    }

    #[test]
    fn whitespace_in_a_condition_is_not_a_change() {
        let st = edited_user(|y| in_block(y, "high_follower_count", " >= ", "   >=   "));
        assert!(st.ok, "{:?}", st.problems);
    }

    #[test]
    fn a_new_leading_skip_is_uncertain() {
        let st = edited_user(|y| {
            y.replacen(
                "rules:\n",
                "rules:\n  - id: new_global_skip\n    when: user.deactivated\n    then: { kind: skip, reason: deactivated }\n",
                1,
            )
        });
        assert!(!st.ok);
        assert!(st.problems.iter().any(|p| p.contains("new_global_skip")));
    }

    #[test]
    fn malformed_rules_never_reach_the_comparison() {
        assert!(CompiledRules::from_yaml("for_entity: user\nrules: [{id: x}]").is_err());
    }

    #[test]
    fn checks_stop_in_list_order_with_the_rules_reason() {
        use crate::facts::{CredFacts, EntityFacts, GizmoduckFacts, ScoreFacts, UserFacts};
        let facts = |present: bool, followers: i64| Facts {
            entity_type: EntityType::User,
            entity_id: 1,
            user_id: 1,
            topic: "t".into(),
            score: ScoreFacts::default(),
            entity: EntityFacts::User(UserFacts {
                user: GizmoduckFacts {
                    present,
                    ..Default::default()
                },
                cred: CredFacts {
                    follower_count: Some(followers),
                    ..Default::default()
                },
                ..Default::default()
            }),
            uas: None,
        };
        let floor = DEFAULT_FOLLOWER_SKIP.ceil() as i64;
        let above = FOLLOWER_CEILING_COUNT.ceil() as i64 + 1;
        let stop = |f: &Facts| first_stop(EntityType::User, f).unwrap().map(|c| c.reason);
        assert_eq!(stop(&facts(true, floor - 1)), None);
        assert_eq!(stop(&facts(true, floor)), Some("high_follower_count"));
        assert_eq!(
            stop(&facts(true, above)),
            Some("high_follower_count"),
            "the ceiling is not in the list"
        );
        assert_eq!(stop(&facts(false, above)), Some("user_not_found"));
        let page = |f: &Facts| {
            first_page_stop(EntityType::User, f, AuthorSkips::default())
                .unwrap()
                .map(|c| c.reason)
        };
        assert_eq!(page(&facts(true, above)), Some("very_high_follower_count"));
        assert_eq!(page(&facts(false, above)), Some("user_not_found"));
    }

                #[test]
    fn a_ceiling_rule_in_the_rules_is_not_compared() {
        let rule = "  - id: very_high_follower_count\n    when: cred.follower_count >= 12.34\n    then:\n      kind: skip\n      reason: very_high_follower_count\n\n";
        for before in ["user_not_found", "high_follower_count", "user_in_allowlist"] {
            let st = edited_user(|y| {
                let (s, _) = rule_block(y, before);
                format!("{}{}{}", &y[..s], rule, &y[s..])
            });
            assert!(st.ok, "before {before}: {:?}", st.problems);
        }
        let post = status(
            EntityType::Post,
            &compiled(&{
                let (s, _) = rule_block(BAKED_POST, "high_follower_count");
                format!("{}{}{}", &BAKED_POST[..s], rule, &BAKED_POST[s..])
            }),
        );
        assert!(post.ok, "{:?}", post.problems);
        assert!(!BAKED_USER.contains("id: very_high_follower_count"));
        assert!(!BAKED_POST.contains("id: very_high_follower_count"));
    }

    #[test]
    fn page_post_decisions_stop_at_the_author_ceiling_whatever_the_waiver() {
        use crate::facts::{
            CredFacts, EntityFacts, GizmoduckFacts, PostFacts, ScoreFacts, UserFacts,
        };
        let facts = |followers: Option<i64>| Facts {
            entity_type: EntityType::Post,
            entity_id: 1,
            user_id: 2,
            topic: "t".into(),
            score: ScoreFacts {
                skip_author_credibility_prechecks: true,
                ..Default::default()
            },
            entity: EntityFacts::Post(PostFacts {
                author: UserFacts {
                    user: GizmoduckFacts {
                        present: true,
                        ..Default::default()
                    },
                    cred: CredFacts {
                        follower_count: followers,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            }),
            uas: None,
        };
        let page = |f: &Facts| {
            first_page_stop(EntityType::Post, f, AuthorSkips::default())
                .unwrap()
                .map(|c| c.reason)
        };
        let ceiling = FOLLOWER_CEILING_COUNT.ceil() as i64;
        assert_eq!(
            page(&facts(Some(ceiling - 1))),
            None,
            "waived below the ceiling"
        );
        assert_eq!(
            page(&facts(Some(ceiling))),
            Some("very_high_follower_count")
        );
        assert_eq!(
            page(&facts(None)),
            Some("very_high_follower_count"),
            "unknown count"
        );
        assert_eq!(
            first_stop(EntityType::Post, &facts(Some(ceiling + 1))).unwrap(),
            None,
            "the rules' own list is unchanged"
        );
    }

        #[derive(Clone, Copy, Default)]
    struct Author {
        followers: Option<i64>,
        high_cred: bool,
        waived: bool,
        allowlisted: bool,
    }

    fn author_facts(entity: EntityType, a: Author, labels: &[&str]) -> Facts {
        use crate::facts::{
            AllowlistFacts, CredFacts, EntityFacts, GizmoduckFacts, PostFacts, ScoreFacts,
            UserFacts,
        };
        let account = UserFacts {
            allowlist: AllowlistFacts {
                is_allowlisted: a.allowlisted,
                ..Default::default()
            },
            user: GizmoduckFacts {
                present: true,
                ..Default::default()
            },
            cred: CredFacts {
                follower_count: a.followers,
                is_high: Some(a.high_cred),
                ..Default::default()
            },
        };
        Facts {
            entity_type: entity,
            entity_id: 1,
            user_id: 2,
            topic: "t".into(),
            score: ScoreFacts {
                skip_author_credibility_prechecks: a.waived,
                labels: labels.iter().map(|l| (*l).to_owned()).collect(),
                ..Default::default()
            },
            entity: match entity {
                EntityType::User => EntityFacts::User(account),
                EntityType::Post => EntityFacts::Post(PostFacts {
                    author: account,
                    ..Default::default()
                }),
            },
            uas: None,
        }
    }

    fn skips(follower_threshold: Option<u32>, skip_high_credibility: Option<bool>) -> AuthorSkips {
        AuthorSkips {
            follower_threshold,
            skip_high_credibility,
        }
    }

    fn page_stop(entity: EntityType, a: Author, s: AuthorSkips) -> Option<&'static str> {
        first_page_stop(entity, &author_facts(entity, a, &[]), s)
            .unwrap()
            .map(|c| c.reason)
    }

    fn with_followers(n: i64) -> Author {
        Author {
            followers: Some(n),
            ..Default::default()
        }
    }

    const BOTH: [EntityType; 2] = [EntityType::User, EntityType::Post];

    #[test]
    fn the_default_and_the_ceiling_come_from_the_lists() {
        let view = follower_skip_view();
        assert_eq!(view.today_threshold, DEFAULT_FOLLOWER_SKIP.ceil() as u64);
        assert_eq!(view.ceiling, follower_ceiling());
        assert!(view.today_threshold < view.ceiling);
    }

    #[test]
    fn absent_skips_decide_exactly_as_the_list() {
        let d = DEFAULT_FOLLOWER_SKIP.ceil() as i64;
        let c = follower_ceiling() as i64;
        let mut authors: Vec<Author> = [None, Some(0), Some(d - 1), Some(d), Some(c - 1), Some(c)]
            .into_iter()
            .map(|followers| Author {
                followers,
                ..Default::default()
            })
            .collect();
        let more: Vec<Author> = authors
            .iter()
            .flat_map(|a| {
                [
                    Author {
                        high_cred: true,
                        ..*a
                    },
                    Author { waived: true, ..*a },
                    Author {
                        allowlisted: true,
                        ..*a
                    },
                ]
            })
            .collect();
        authors.extend(more);
        let today = [
            skips(None, Some(true)),
            skips(Some(d as u32), None),
            skips(Some(d as u32), Some(true)),
        ];
        for entity in BOTH {
            for a in &authors {
                let f = author_facts(entity, *a, &[]);
                let listed = first_stop(entity, &f).unwrap().map(|c| c.reason);
                for s in today {
                    let rebuilt = first_stop_with(entity, &f, s).unwrap().map(|c| c.reason);
                    assert_eq!(
                        listed, rebuilt,
                        "{entity:?} {s:?} followers={:?} cred={} waived={} allowlisted={}",
                        a.followers, a.high_cred, a.waived, a.allowlisted
                    );
                }
            }
        }
    }

    #[test]
    fn a_zero_threshold_turns_the_follower_skip_off_but_not_the_ceiling() {
        let d = DEFAULT_FOLLOWER_SKIP.ceil() as i64;
        let c = follower_ceiling() as i64;
        for entity in BOTH {
            let off = skips(Some(0), None);
            assert_eq!(
                page_stop(entity, with_followers(d), AuthorSkips::default()),
                Some("high_follower_count")
            );
            assert_eq!(
                page_stop(entity, with_followers(d), off),
                None,
                "{entity:?}"
            );
            assert_eq!(
                page_stop(entity, with_followers(c - 1), off),
                None,
                "{entity:?}"
            );
            assert_eq!(
                page_stop(entity, with_followers(c), off),
                Some("very_high_follower_count"),
                "{entity:?}"
            );
            let cred = Author {
                high_cred: true,
                ..with_followers(d)
            };
            assert_eq!(
                page_stop(entity, cred, off),
                Some("pagerank_skipped"),
                "{entity:?}"
            );
        }
    }

    #[test]
    fn a_threshold_skips_from_that_count_up() {
        let d = DEFAULT_FOLLOWER_SKIP.ceil() as i64;
        let c = follower_ceiling() as i64;
        for entity in BOTH {
            for n in [1_i64, 250, d - 1, d, d + 1, c - 1] {
                let s = skips(Some(n as u32), None);
                assert_eq!(
                    page_stop(entity, with_followers(n - 1), s),
                    None,
                    "{entity:?} {n}"
                );
                assert_eq!(
                    page_stop(entity, with_followers(n), s),
                    Some("high_follower_count"),
                    "{entity:?} {n}"
                );
            }
            assert_eq!(
                page_stop(EntityType::User, Author::default(), skips(Some(1), None)),
                None,
                "an unknown count reads as 0 on an account"
            );
            assert_eq!(
                page_stop(EntityType::Post, Author::default(), skips(Some(0), None)),
                Some("very_high_follower_count"),
                "and stops a post at the ceiling"
            );
        }
        assert_eq!(
            page_stop(EntityType::User, with_followers(c), skips(Some(1), None)),
            Some("very_high_follower_count")
        );
        assert_eq!(
            page_stop(EntityType::Post, with_followers(c), skips(Some(1), None)),
            Some("very_high_follower_count")
        );
    }

    #[test]
    fn the_producer_waiver_lifts_both_skips_on_posts_only() {
        let d = DEFAULT_FOLLOWER_SKIP.ceil() as i64;
        let c = follower_ceiling() as i64;
        let waived = |n: i64| Author {
            waived: true,
            ..with_followers(n)
        };
        let waived_cred = Author {
            high_cred: true,
            ..waived(1)
        };
        for s in [
            AuthorSkips::default(),
            skips(Some(250), None),
            skips(Some(0), Some(true)),
            skips(None, Some(true)),
        ] {
            assert_eq!(page_stop(EntityType::Post, waived(d), s), None, "{s:?}");
            assert_eq!(page_stop(EntityType::Post, waived(250), s), None, "{s:?}");
            assert_eq!(page_stop(EntityType::Post, waived_cred, s), None, "{s:?}");
            assert_eq!(
                page_stop(EntityType::Post, waived(c), s),
                Some("very_high_follower_count"),
                "{s:?}"
            );
        }
        assert_eq!(
            page_stop(EntityType::User, waived(d), AuthorSkips::default()),
            Some("high_follower_count"),
            "an account decision has no waiver"
        );
        assert_eq!(
            page_stop(EntityType::User, waived(250), skips(Some(250), None)),
            Some("high_follower_count")
        );
        assert_eq!(
            page_stop(EntityType::User, waived_cred, skips(Some(0), Some(true))),
            Some("pagerank_skipped")
        );
    }

    #[test]
    fn the_credibility_skip_follows_the_head() {
        let cred = Author {
            high_cred: true,
            ..with_followers(1)
        };
        for entity in BOTH {
            assert_eq!(
                page_stop(entity, cred, AuthorSkips::default()),
                Some("pagerank_skipped")
            );
            assert_eq!(
                page_stop(entity, cred, skips(None, Some(true))),
                Some("pagerank_skipped")
            );
            assert_eq!(
                page_stop(entity, cred, skips(None, Some(false))),
                None,
                "{entity:?}"
            );
            assert_eq!(
                page_stop(entity, cred, skips(Some(0), Some(false))),
                None,
                "{entity:?}"
            );
            let d = DEFAULT_FOLLOWER_SKIP.ceil() as i64;
            let both = Author {
                high_cred: true,
                ..with_followers(d)
            };
            assert_eq!(
                page_stop(entity, both, skips(None, Some(false))),
                Some("high_follower_count"),
                "{entity:?}"
            );
        }
    }

    #[test]
    fn the_checks_before_the_follower_skip_still_stop_first() {
        let off = skips(Some(0), Some(false));
        for entity in BOTH {
            let allowlisted = Author {
                allowlisted: true,
                ..with_followers(1)
            };
            assert_eq!(
                page_stop(entity, allowlisted, off),
                Some("user_in_allowlist")
            );
            let mut f = author_facts(entity, with_followers(1), &[]);
            match &mut f.entity {
                crate::facts::EntityFacts::User(u) => u.user.present = false,
                crate::facts::EntityFacts::Post(p) => p.author.user.present = false,
            }
            assert_eq!(
                first_page_stop(entity, &f, off).unwrap().map(|c| c.reason),
                Some("user_not_found")
            );
        }
    }

    #[test]
    fn no_routing_label_changes_the_author_skips() {
        let d = DEFAULT_FOLLOWER_SKIP.ceil() as i64;
        let labels = ["anchor_campaign_suspend_cse", "cse_reports_embedding_v2"];
        for entity in BOTH {
            for s in [
                AuthorSkips::default(),
                skips(Some(0), Some(false)),
                skips(Some(250), None),
            ] {
                for a in [
                    with_followers(d),
                    with_followers(250),
                    Author {
                        high_cred: true,
                        ..with_followers(1)
                    },
                ] {
                    let plain = first_page_stop(entity, &author_facts(entity, a, &[]), s).unwrap();
                    let cse =
                        first_page_stop(entity, &author_facts(entity, a, &labels), s).unwrap();
                    assert_eq!(plain.map(|c| c.reason), cse.map(|c| c.reason));
                }
            }
        }
    }

    #[test]
    fn lifting_a_skip_is_harsher_and_adding_one_is_not() {
        let d = DEFAULT_FOLLOWER_SKIP.ceil() as u32;
        let below = (d / 4).max(1);
        let today = AuthorSkips::default();
        assert!(today.harsher_than(today).is_empty());
        assert_eq!(
            skips(Some(0), None).harsher_than(today),
            [format!("skips fewer high-follower accounts ({d}+ to off)")]
        );
        assert_eq!(
            skips(Some(d * 5), None).harsher_than(today),
            [format!(
                "skips fewer high-follower accounts ({d}+ to {}+)",
                d * 5
            )]
        );
        assert!(skips(Some(below), None).harsher_than(today).is_empty());
        assert!(skips(Some(d), None).harsher_than(today).is_empty());
        assert!(today.harsher_than(skips(Some(0), None)).is_empty());
        assert!(
            skips(Some(below), None)
                .harsher_than(skips(Some(0), None))
                .is_empty()
        );
        assert_eq!(
            skips(None, Some(false)).harsher_than(today),
            ["stops skipping high-credibility accounts"]
        );
        assert!(
            skips(None, Some(true))
                .harsher_than(skips(None, Some(false)))
                .is_empty()
        );
        assert_eq!(skips(Some(0), Some(false)).harsher_than(today).len(), 2);
    }
}
