use std::sync::LazyLock;

use cel::Program;
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
        id: "very_high_follower_count",
        reason: "very_high_follower_count",
        when: "cred.follower_count >= 12345.67",
    },
    SafetyCheck {
        id: "high_follower_count",
        reason: "high_follower_count",
        when: "cred.follower_count >= 12.34",
    },
    SafetyCheck {
        id: "pagerank_skipped",
        reason: "pagerank_skipped",
        when: "cred.is_high || cred.score >= 50.0",
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
        when: "(cred.is_high || cred.score >= 50.0) && !score.skip_author_credibility_prechecks",
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
    let leading = rules.leading_skips();
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

pub const PAGE_POST_AUTHOR_CEILING: SafetyCheck = SafetyCheck {
    id: "very_high_follower_count",
    reason: "very_high_follower_count",
    when: "cred.follower_count >= 12345.67",
};

static PAGE_POST_AUTHOR_CEILING_FOLLOWERS: LazyLock<f64> = LazyLock::new(|| {
    PAGE_POST_AUTHOR_CEILING
        .when
        .rsplit(">=")
        .next()
        .and_then(|n| n.trim().parse().ok())
        .expect("the post author ceiling is `cred.follower_count >= N`")
});

const POST_PRE_FOLLOWER: &[&str] = &["post_in_allowlist", "user_in_allowlist", "user_not_found"];

pub fn first_page_stop(
    entity: EntityType,
    facts: &Facts,
) -> Result<Option<&'static SafetyCheck>, EvalError> {
    let listed = first_stop(entity, facts);
    if entity != EntityType::Post {
        return listed;
    }
    if let Ok(Some(check)) = &listed
        && POST_PRE_FOLLOWER.contains(&check.id)
    {
        return listed;
    }
    if facts
        .cred()
        .follower_count
        .is_none_or(|n| n as f64 >= *PAGE_POST_AUTHOR_CEILING_FOLLOWERS)
    {
        return Ok(Some(&PAGE_POST_AUTHOR_CEILING));
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
        assert!(PAGE_POST_AUTHOR_CEILING_FOLLOWERS.is_finite());
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
        let st = edited_user(|y| without(y, "very_high_follower_count"));
        assert!(!st.ok);
        assert_eq!(
            st.checks
                .iter()
                .find(|c| c.id == "very_high_follower_count")
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
            let (s, e) = rule_block(y, "very_high_follower_count");
            let block = y[s..e].to_owned();
            let rest = format!("{}{}", &y[..s], &y[e..]);
            let (_, after) = rule_block(&rest, "high_follower_count");
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
        let st = edited_user(|y| in_block(y, "very_high_follower_count", " >= ", "   >=   "));
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
        let stop = |f: &Facts| first_stop(EntityType::User, f).unwrap().map(|c| c.reason);
        assert_eq!(stop(&facts(true, 10)), None);
        assert_eq!(stop(&facts(true, 5_000)), Some("high_follower_count"));
        assert_eq!(
            stop(&facts(true, 200_000)),
            Some("very_high_follower_count")
        );
        assert_eq!(stop(&facts(false, 200_000)), Some("user_not_found"));
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
            first_page_stop(EntityType::Post, f)
                .unwrap()
                .map(|c| c.reason)
        };
        let ceiling = PAGE_POST_AUTHOR_CEILING_FOLLOWERS.ceil() as i64;
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
            first_stop(EntityType::Post, &facts(Some(200_000))).unwrap(),
            None,
            "the rules' own list is unchanged"
        );
    }
}
