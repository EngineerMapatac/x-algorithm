use std::collections::HashSet;

use serde::Serialize;
use tracing::warn;

use crate::decision::ActionSpec;
use crate::facts::{EntityType, RequestedActionFacts};

pub const USER_KINDS: &[&str] = &[
    "suspend",
    "label",
    "bounce_captcha",
    "bounce_arkose",
    "spam_liveness_check",
];

pub const POST_KINDS: &[&str] = &["post_label", "suspend_author"];

pub const MAX_REQUESTED_ACTIONS_PER_MESSAGE: usize = 16;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GenericActionAllowlist {
    pub kinds: HashSet<String>,
    pub suspend_policies: HashSet<String>,
    pub labels: HashSet<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SkippedRequestedAction {
    pub kind: String,
    pub head: String,
    pub reason: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<usize>,
}

#[derive(Debug, Default)]
pub struct ResolvedRequestedActions {
    pub specs: Vec<ActionSpec>,
    pub skipped: Vec<SkippedRequestedAction>,
}

impl ResolvedRequestedActions {
    pub fn skipped_info_json(&self) -> Option<String> {
        if self.skipped.is_empty() {
            return None;
        }
        serde_json::to_string(&self.skipped).ok()
    }
}

fn hardcoded_kinds(entity_type: EntityType) -> &'static [&'static str] {
    match entity_type {
        EntityType::User => USER_KINDS,
        EntityType::Post => POST_KINDS,
    }
}

fn metric_kind(kind: &str) -> &str {
    if USER_KINDS.contains(&kind) || POST_KINDS.contains(&kind) {
        kind
    } else {
        "unknown"
    }
}

fn sanitize_for_log(s: &str) -> String {
    const MAX: usize = 64;
    let mut out: String = s
        .chars()
        .take(MAX)
        .map(|c| if c.is_control() { '?' } else { c })
        .collect();
    if s.chars().count() > MAX {
        out.push('…');
    }
    out
}

fn action_to_spec(
    entity_type: EntityType,
    action: &RequestedActionFacts,
    allowlist: &GenericActionAllowlist,
) -> Result<ActionSpec, &'static str> {
    let kind = action.kind.as_str();
    if !hardcoded_kinds(entity_type).contains(&kind) {
        let other = match entity_type {
            EntityType::User => EntityType::Post,
            EntityType::Post => EntityType::User,
        };
        return Err(if hardcoded_kinds(other).contains(&kind) {
            "entity_type_mismatch"
        } else {
            "unknown_kind"
        });
    }
    if !allowlist.kinds.contains(kind) {
        return Err("kind_not_allowlisted");
    }
    match kind {
        "suspend" | "suspend_author" => {
            if !allowlist.suspend_policies.contains(&action.policy) {
                return Err("policy_not_allowlisted");
            }
            Ok(ActionSpec::SuspendUser {
                perm: action.perm,
                policy: action.policy.clone(),
            })
        }
        "label" | "post_label" => {
            if action.labels.is_empty() {
                return Err("no_labels");
            }
            if action.labels.iter().any(|l| !allowlist.labels.contains(l)) {
                return Err("label_not_allowlisted");
            }
            if action.ttl_msec < 0 {
                return Err("invalid_ttl");
            }
            let ttl_msec = (action.ttl_msec > 0).then_some(action.ttl_msec);
            Ok(if kind == "label" {
                ActionSpec::AddLabelsV2 {
                    labels: action.labels.clone(),
                    ttl_msec,
                }
            } else {
                ActionSpec::AddPostLabelsV2 {
                    labels: action.labels.clone(),
                    ttl_msec,
                }
            })
        }
        "bounce_captcha" => Ok(ActionSpec::Captcha),
        "bounce_arkose" => Ok(ActionSpec::Arkose),
        "spam_liveness_check" => Ok(ActionSpec::SpamLivenessCheck),
        _ => Err("unknown_kind"),
    }
}

pub fn resolve_requested_actions(
    entity_type: EntityType,
    requested: &[RequestedActionFacts],
    allowlist: &GenericActionAllowlist,
) -> ResolvedRequestedActions {
    let mut out = ResolvedRequestedActions::default();
    let considered = &requested[..requested.len().min(MAX_REQUESTED_ACTIONS_PER_MESSAGE)];
    let overflow = requested.len() - considered.len();
    for action in considered {
        let resolved = action_to_spec(entity_type, action, allowlist).and_then(|spec| {
            if out.specs.contains(&spec) {
                Err("duplicate_action")
            } else {
                Ok(spec)
            }
        });
        match resolved {
            Ok(spec) => {
                crate::metrics::GENERIC_ACTION_TOTAL
                    .with_label_values(&[
                        entity_type.as_str(),
                        metric_kind(&action.kind),
                        "resolved",
                        "",
                    ])
                    .inc();
                out.specs.push(spec);
            }
            Err(reason) => {
                warn!(
                    entity_type = entity_type.as_str(),
                    kind = %sanitize_for_log(&action.kind),
                    kind_len = action.kind.len(),
                    head = %sanitize_for_log(&action.head),
                    head_len = action.head.len(),
                    reason,
                    "requested action skipped (fail-closed)"
                );
                crate::metrics::GENERIC_ACTION_TOTAL
                    .with_label_values(&[
                        entity_type.as_str(),
                        metric_kind(&action.kind),
                        "skipped",
                        reason,
                    ])
                    .inc();
                out.skipped.push(SkippedRequestedAction {
                    kind: action.kind.clone(),
                    head: action.head.clone(),
                    reason,
                    count: None,
                });
            }
        }
    }
    if overflow > 0 {
        warn!(
            entity_type = entity_type.as_str(),
            overflow,
            cap = MAX_REQUESTED_ACTIONS_PER_MESSAGE,
            reason = "too_many_actions",
            "requested actions past the per-message cap skipped (fail-closed, aggregated)"
        );
        crate::metrics::GENERIC_ACTION_TOTAL
            .with_label_values(&[
                entity_type.as_str(),
                "aggregate",
                "skipped",
                "too_many_actions",
            ])
            .inc();
        out.skipped.push(SkippedRequestedAction {
            kind: String::new(),
            head: String::new(),
            reason: "too_many_actions",
            count: Some(overflow),
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn full_user_allowlist() -> GenericActionAllowlist {
        GenericActionAllowlist {
            kinds: USER_KINDS.iter().map(|s| (*s).to_owned()).collect(),
            suspend_policies: ["PlatformManipulation"]
                .iter()
                .map(|s| (*s).to_owned())
                .collect(),
            labels: ["SpamHighRecall"].iter().map(|s| (*s).to_owned()).collect(),
        }
    }

    fn full_post_allowlist() -> GenericActionAllowlist {
        GenericActionAllowlist {
            kinds: POST_KINDS.iter().map(|s| (*s).to_owned()).collect(),
            suspend_policies: ["Cse"].iter().map(|s| (*s).to_owned()).collect(),
            labels: ["SpamHighRecall"].iter().map(|s| (*s).to_owned()).collect(),
        }
    }

    fn req(kind: &str) -> RequestedActionFacts {
        RequestedActionFacts {
            kind: kind.into(),
            head: "SomeHead".into(),
            ..Default::default()
        }
    }

    fn suspend_req(kind: &str, policy: &str, perm: bool) -> RequestedActionFacts {
        RequestedActionFacts {
            kind: kind.into(),
            perm,
            policy: policy.into(),
            head: "SomeHead".into(),
            ..Default::default()
        }
    }

    fn label_req(kind: &str, labels: &[&str], ttl_msec: i64) -> RequestedActionFacts {
        RequestedActionFacts {
            kind: kind.into(),
            labels: labels.iter().map(|s| (*s).to_owned()).collect(),
            ttl_msec,
            head: "SomeHead".into(),
            ..Default::default()
        }
    }

    #[test]
    fn user_suspend_maps_to_suspend_user_spec() {
        let r = resolve_requested_actions(
            EntityType::User,
            &[suspend_req("suspend", "PlatformManipulation", true)],
            &full_user_allowlist(),
        );
        assert!(r.skipped.is_empty());
        assert_eq!(
            r.specs,
            vec![ActionSpec::SuspendUser {
                perm: true,
                policy: "PlatformManipulation".into(),
            }]
        );
    }

    #[test]
    fn user_label_maps_to_add_labels_v2_with_ttl() {
        let r = resolve_requested_actions(
            EntityType::User,
            &[label_req("label", &["SpamHighRecall"], 86_400_000)],
            &full_user_allowlist(),
        );
        assert!(r.skipped.is_empty());
        assert_eq!(
            r.specs,
            vec![ActionSpec::AddLabelsV2 {
                labels: vec!["SpamHighRecall".into()],
                ttl_msec: Some(86_400_000),
            }]
        );
    }

    #[test]
    fn label_ttl_zero_maps_to_no_expiry() {
        let r = resolve_requested_actions(
            EntityType::User,
            &[label_req("label", &["SpamHighRecall"], 0)],
            &full_user_allowlist(),
        );
        assert_eq!(
            r.specs,
            vec![ActionSpec::AddLabelsV2 {
                labels: vec!["SpamHighRecall".into()],
                ttl_msec: None,
            }]
        );
    }

    #[test]
    fn user_challenge_kinds_map_to_their_specs() {
        let r = resolve_requested_actions(
            EntityType::User,
            &[
                req("bounce_captcha"),
                req("bounce_arkose"),
                req("spam_liveness_check"),
            ],
            &full_user_allowlist(),
        );
        assert!(r.skipped.is_empty());
        assert_eq!(
            r.specs,
            vec![
                ActionSpec::Captcha,
                ActionSpec::Arkose,
                ActionSpec::SpamLivenessCheck,
            ]
        );
    }

    #[test]
    fn post_label_maps_to_add_post_labels_v2() {
        let r = resolve_requested_actions(
            EntityType::Post,
            &[label_req("post_label", &["SpamHighRecall"], 86_400_000)],
            &full_post_allowlist(),
        );
        assert!(r.skipped.is_empty());
        assert_eq!(
            r.specs,
            vec![ActionSpec::AddPostLabelsV2 {
                labels: vec!["SpamHighRecall".into()],
                ttl_msec: Some(86_400_000),
            }]
        );
    }

    #[test]
    fn post_suspend_author_maps_to_suspend_user_spec() {
        let r = resolve_requested_actions(
            EntityType::Post,
            &[suspend_req("suspend_author", "Cse", true)],
            &full_post_allowlist(),
        );
        assert!(r.skipped.is_empty());
        assert_eq!(
            r.specs,
            vec![ActionSpec::SuspendUser {
                perm: true,
                policy: "Cse".into(),
            }]
        );
    }

    #[test]
    fn post_kinds_on_user_entity_are_refused_even_when_allowlisted() {
        let mut allowlist = full_user_allowlist();
        allowlist.kinds.insert("post_label".into());
        allowlist.kinds.insert("suspend_author".into());
        for kind in POST_KINDS {
            let r = resolve_requested_actions(
                EntityType::User,
                &[suspend_req(kind, "PlatformManipulation", false)],
                &allowlist,
            );
            assert!(r.specs.is_empty(), "{kind} must not dispatch on user");
            assert_eq!(r.skipped[0].reason, "entity_type_mismatch");
        }
    }

    #[test]
    fn user_kinds_on_post_entity_are_refused_even_when_allowlisted() {
        let mut allowlist = full_post_allowlist();
        for kind in USER_KINDS {
            allowlist.kinds.insert((*kind).to_owned());
        }
        allowlist
            .suspend_policies
            .insert("PlatformManipulation".into());
        for kind in USER_KINDS {
            let r = resolve_requested_actions(
                EntityType::Post,
                &[RequestedActionFacts {
                    kind: (*kind).to_owned(),
                    policy: "PlatformManipulation".into(),
                    labels: vec!["SpamHighRecall".into()],
                    head: "SomeHead".into(),
                    ..Default::default()
                }],
                &allowlist,
            );
            assert!(r.specs.is_empty(), "{kind} must not dispatch on post");
            assert_eq!(r.skipped[0].reason, "entity_type_mismatch");
        }
    }

    #[test]
    fn unknown_kind_is_refused() {
        for entity in [EntityType::User, EntityType::Post] {
            let allowlist = match entity {
                EntityType::User => full_user_allowlist(),
                EntityType::Post => full_post_allowlist(),
            };
            let r = resolve_requested_actions(entity, &[req("bounce")], &allowlist);
            assert!(r.specs.is_empty());
            assert_eq!(r.skipped[0].reason, "unknown_kind");
        }
    }

    #[test]
    fn empty_allowlist_refuses_everything() {
        let empty = GenericActionAllowlist::default();
        let r = resolve_requested_actions(
            EntityType::User,
            &[
                suspend_req("suspend", "PlatformManipulation", false),
                label_req("label", &["SpamHighRecall"], 0),
                req("bounce_captcha"),
                req("bounce_arkose"),
                req("spam_liveness_check"),
            ],
            &empty,
        );
        assert!(r.specs.is_empty());
        assert_eq!(r.skipped.len(), 5);
        assert!(r.skipped.iter().all(|s| s.reason == "kind_not_allowlisted"));

        let r = resolve_requested_actions(
            EntityType::Post,
            &[
                label_req("post_label", &["SpamHighRecall"], 0),
                suspend_req("suspend_author", "Cse", true),
            ],
            &empty,
        );
        assert!(r.specs.is_empty());
        assert_eq!(r.skipped.len(), 2);
    }

    #[test]
    fn non_allowlisted_kind_is_refused() {
        let mut allowlist = full_user_allowlist();
        allowlist.kinds.remove("suspend");
        let r = resolve_requested_actions(
            EntityType::User,
            &[suspend_req("suspend", "PlatformManipulation", false)],
            &allowlist,
        );
        assert!(r.specs.is_empty());
        assert_eq!(r.skipped[0].reason, "kind_not_allowlisted");
    }

    #[test]
    fn non_allowlisted_suspend_policy_is_refused() {
        for (entity, kind, allowlist) in [
            (EntityType::User, "suspend", full_user_allowlist()),
            (EntityType::Post, "suspend_author", full_post_allowlist()),
        ] {
            let r = resolve_requested_actions(
                entity,
                &[suspend_req(kind, "SomeOtherPolicy", false)],
                &allowlist,
            );
            assert!(r.specs.is_empty(), "{kind} with foreign policy");
            assert_eq!(r.skipped[0].reason, "policy_not_allowlisted");

            let r = resolve_requested_actions(entity, &[suspend_req(kind, "", false)], &allowlist);
            assert_eq!(r.skipped[0].reason, "policy_not_allowlisted");
        }
    }

    #[test]
    fn non_allowlisted_label_is_refused() {
        for (entity, kind, allowlist) in [
            (EntityType::User, "label", full_user_allowlist()),
            (EntityType::Post, "post_label", full_post_allowlist()),
        ] {
            let r = resolve_requested_actions(
                entity,
                &[label_req(kind, &["SpamHighRecall", "SomethingElse"], 0)],
                &allowlist,
            );
            assert!(r.specs.is_empty(), "{kind} with foreign label");
            assert_eq!(r.skipped[0].reason, "label_not_allowlisted");
        }
    }

    #[test]
    fn label_kind_with_no_labels_is_refused() {
        let r = resolve_requested_actions(
            EntityType::User,
            &[label_req("label", &[], 0)],
            &full_user_allowlist(),
        );
        assert!(r.specs.is_empty());
        assert_eq!(r.skipped[0].reason, "no_labels");
    }

    #[test]
    fn negative_ttl_is_refused() {
        for (entity, kind, allowlist) in [
            (EntityType::User, "label", full_user_allowlist()),
            (EntityType::Post, "post_label", full_post_allowlist()),
        ] {
            let r = resolve_requested_actions(
                entity,
                &[label_req(kind, &["SpamHighRecall"], -1)],
                &allowlist,
            );
            assert!(r.specs.is_empty(), "{kind} with negative ttl");
            assert_eq!(r.skipped[0].reason, "invalid_ttl");
        }
    }

    #[test]
    fn entries_past_the_hardcoded_cap_are_refused_as_one_aggregate() {
        let mut requested: Vec<RequestedActionFacts> = (0..MAX_REQUESTED_ACTIONS_PER_MESSAGE)
            .map(|i| label_req("label", &["SpamHighRecall"], (i as i64 + 1) * 1000))
            .collect();
        requested.push(req("bounce_captcha"));
        requested.push(req("bounce_arkose"));
        let r = resolve_requested_actions(EntityType::User, &requested, &full_user_allowlist());
        assert_eq!(r.specs.len(), MAX_REQUESTED_ACTIONS_PER_MESSAGE);
        assert_eq!(
            r.skipped,
            vec![SkippedRequestedAction {
                kind: String::new(),
                head: String::new(),
                reason: "too_many_actions",
                count: Some(2),
            }]
        );
        let json = r.skipped_info_json().expect("aggregate serializes");
        assert!(json.contains("\"count\":2"), "{json}");
    }

    #[test]
    fn per_entry_refusals_omit_the_count_field() {
        let r = resolve_requested_actions(
            EntityType::User,
            &[req("post_label")],
            &full_user_allowlist(),
        );
        assert_eq!(r.skipped[0].count, None);
        let json = r.skipped_info_json().expect("skipped list serializes");
        assert!(!json.contains("count"), "{json}");
    }

    #[test]
    fn identical_resolved_specs_are_deduplicated() {
        let r = resolve_requested_actions(
            EntityType::Post,
            &[
                label_req("post_label", &["SpamHighRecall"], 0),
                suspend_req("suspend_author", "Cse", true),
                label_req("post_label", &["SpamHighRecall"], 0),
                suspend_req("suspend_author", "Cse", true),
            ],
            &full_post_allowlist(),
        );
        assert_eq!(
            r.specs,
            vec![
                ActionSpec::AddPostLabelsV2 {
                    labels: vec!["SpamHighRecall".into()],
                    ttl_msec: None,
                },
                ActionSpec::SuspendUser {
                    perm: true,
                    policy: "Cse".into(),
                },
            ]
        );
        assert_eq!(r.skipped.len(), 2);
        assert!(r.skipped.iter().all(|s| s.reason == "duplicate_action"));
    }

    #[test]
    fn mixed_list_dispatches_allowed_and_skips_refused_in_order() {
        let r = resolve_requested_actions(
            EntityType::User,
            &[
                label_req("label", &["SpamHighRecall"], 1000),
                req("post_label"),
                suspend_req("suspend", "PlatformManipulation", false),
            ],
            &full_user_allowlist(),
        );
        assert_eq!(
            r.specs,
            vec![
                ActionSpec::AddLabelsV2 {
                    labels: vec!["SpamHighRecall".into()],
                    ttl_msec: Some(1000),
                },
                ActionSpec::SuspendUser {
                    perm: false,
                    policy: "PlatformManipulation".into(),
                },
            ]
        );
        assert_eq!(r.skipped.len(), 1);
        assert_eq!(r.skipped[0].kind, "post_label");

        let json = r.skipped_info_json().expect("skipped list serializes");
        assert!(json.contains("\"post_label\""));
        assert!(json.contains("entity_type_mismatch"));
    }

    #[test]
    fn skipped_info_json_none_when_nothing_skipped() {
        let r = resolve_requested_actions(
            EntityType::User,
            &[req("bounce_captcha")],
            &full_user_allowlist(),
        );
        assert!(r.skipped_info_json().is_none());
    }

    #[test]
    fn metric_kind_collapses_unknown_kinds() {
        assert_eq!(metric_kind("suspend"), "suspend");
        assert_eq!(metric_kind("post_label"), "post_label");
        assert_eq!(metric_kind("totally-made-up"), "unknown");
        assert_eq!(metric_kind(""), "unknown");
    }

    #[test]
    fn sanitize_for_log_strips_control_chars_and_truncates() {
        assert_eq!(sanitize_for_log("suspend"), "suspend");
        assert_eq!(
            sanitize_for_log("evil\nline\x1b[31mred"),
            "evil?line?[31mred"
        );
        let long = "a".repeat(200);
        let sanitized = sanitize_for_log(&long);
        assert_eq!(sanitized.chars().count(), 65);
        assert!(sanitized.ends_with('…'));
    }
}
