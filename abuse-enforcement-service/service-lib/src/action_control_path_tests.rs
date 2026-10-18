use std::collections::BTreeMap;
use std::time::Duration;

use xai_abuse_proto::enforcement::ScoreResult;

use super::*;
use crate::action_control::test_support::record;
use crate::action_control::{ActionControlRecord, ControlMode, Lifetime, PageAction};
use crate::replay_tests::{
    Account, BAKED_POST_RULES, BAKED_USER_RULES, BedConfig, HEADS, TestBed, Trace, base_config,
    message, on_config, scenarios, switched_off,
};

const SUSPEND: PageAction = PageAction::Suspend {};
const LIVENESS: PageAction = PageAction::LivenessCheck {};
const SHR7: PageAction = PageAction::SpamHighRecall {
    lifetime: Lifetime::Days7,
};

const BB1: (&str, &str) = HEADS[0];

fn suspend_label() -> String {
    "anchor_campaign_suspend".into()
}

async fn bed(records: Vec<ActionControlRecord>, dry_run: bool) -> TestBed {
    bed_with(records, on_config(dry_run, None), BAKED_USER_RULES).await
}

async fn bed_with(
    records: Vec<ActionControlRecord>,
    config: serde_json::Value,
    user_rules: &str,
) -> TestBed {
    TestBed::new(BedConfig {
        user_rules,
        post_rules: BAKED_POST_RULES,
        config,
        off_switches: vec![switched_off()],
        control_records: records,
    })
    .await
}

fn bb1(labels: &[&str], account: Account) -> ScoreResult {
    message(EntityType::User, BB1.0, BB1.1, labels, account)
}

fn info(t: &Trace) -> &BTreeMap<String, String> {
    &t.outcomes.last().expect("one outcome").info
}

fn ais_names(t: &Trace) -> Vec<String> {
    t.ais
        .iter()
        .flat_map(|(_, r)| {
            let actions: Vec<&serde_json::Value> = match r.get("actions") {
                Some(serde_json::Value::Array(a)) => a.iter().collect(),
                _ => r.get("action").into_iter().collect(),
            };
            actions
                .into_iter()
                .filter_map(|a| a.as_object().and_then(|o| o.keys().next().cloned()))
                .collect::<Vec<_>>()
        })
        .collect()
}

fn live(head: &str, band: Option<&str>, actions: Vec<PageAction>) -> ActionControlRecord {
    record(head, None, band, ControlMode::Live, actions)
}

#[tokio::test]
async fn a_user_record_on_a_shared_head_never_touches_its_post_decisions() {
    let _serial = overturn_hold::tests::METRICS_LOCK.lock().await;
    let head = "SharedSpamBot";
    let model = "shared_spam_v1";
    let none = bed(vec![], false).await;
    let user_live = bed(vec![live(head, None, vec![SUSPEND])], false).await;
    let mut acted = 0;
    for label in crate::replay_tests::rule_labels(BAKED_POST_RULES) {
        let post = message(EntityType::Post, head, model, &[&label], Account::Clean);
        let was = none.run_fresh(&post).await;
        let now = user_live.run_fresh(&post).await;
        assert_eq!(now.status, was.status, "{label}");
        assert_eq!(ais_names(&now), ais_names(&was), "{label}");
        assert_eq!(
            (now.tickets, &now.dedup_calls),
            (was.tickets, &was.dedup_calls),
            "{label}"
        );
        if let Some(o) = now.outcomes.last() {
            assert_eq!(
                o.info.get("action_control_mode").map(String::as_str),
                Some("rules")
            );
            assert!(!o.info.contains_key("action_control_lkey"), "{label}");
        }
        acted += usize::from(!now.ais.is_empty());
    }
    assert!(acted > 0, "some post label acts through the rules");
    let user = message(
        EntityType::User,
        head,
        model,
        &[&suspend_label()],
        Account::Clean,
    );
    let t = user_live.run_fresh(&user).await;
    let i = info(&t);
    assert_eq!(i["action_control_mode"], "live");
    assert_eq!(i["action_control_lkey"], "user/h/SharedSpamBot");
    assert_eq!(i["matched_rule_id"], "page");
}

#[tokio::test]
async fn recording_takes_no_dedup_claim_no_ticket_and_no_ais_call() {
    let _serial = overturn_hold::tests::METRICS_LOCK.lock().await;
    let label = suspend_label();
    for rec in [
        record(BB1.0, None, None, ControlMode::Recording, vec![]),
        record(BB1.0, None, None, ControlMode::Recording, vec![LIVENESS]),
    ] {
        let page_actions = !rec.actions.is_empty();
        let b = bed(vec![rec], false).await;
        let t = b.run_fresh(&bb1(&[&label], Account::Clean)).await;
        assert_eq!(t.status, "dry_run");
        let want_peek = if page_actions {
            "peek(class=2)=None"
        } else {
            "peek(class=3)=None"
        };
        assert_eq!(
            t.dedup_calls,
            vec![want_peek],
            "a peek, never a claim or a write"
        );
        assert_eq!(t.tickets, 0, "no daily-cap ticket");
        assert!(t.ais.is_empty(), "no AIS call");
        let i = info(&t);
        assert_eq!(i["dry_run_source"], "page");
        assert_eq!(i["action_control_mode"], "recording");
        assert_eq!(i["would_dedup_skip"], "false");
        assert_eq!(i["would_rate_cap"], "false");
        assert_eq!(i["env"], "prod");
        let want_specs = if page_actions {
            r#"[{"kind":"liveness_check"}]"#
        } else {
            r#"[{"kind":"suspend","permanent":false,"policy":"PlatformManipulation"}]"#
        };
        assert_eq!(i["action_specs"], want_specs);
        assert!(!i.contains_key("acted_class"));

        b.dedup.hold(EntityType::User, Account::Clean.user_id(), 3);
        let t = b.run(&bb1(&[&label], Account::Clean)).await;
        assert_eq!(t.status, "dry_run");
        assert_eq!(info(&t)["would_dedup_skip"], "true");
        assert_eq!(info(&t)["would_dedup_held_class"], "3");
        assert!(
            t.dedup_calls.iter().all(|c| c.starts_with("peek")),
            "{:?}",
            t.dedup_calls
        );
    }
}

#[tokio::test]
async fn producer_dry_run_records_only() {
    let _serial = overturn_hold::tests::METRICS_LOCK.lock().await;
    let label = suspend_label();
    let b = bed(vec![], false).await;
    let mut by_field = bb1(&[&label], Account::Clean);
    by_field.dry_run = true;
    let prefixed_label = format!("dry_run__{label}");
    let by_prefix = bb1(&[&prefixed_label], Account::Clean);
    for score in [by_field, by_prefix] {
        let t = b.run_fresh(&score).await;
        assert_eq!(t.status, "dry_run", "the rules still route on the label");
        assert_eq!(t.dedup_calls, vec!["peek(class=3)=None"]);
        assert_eq!(t.tickets, 0);
        assert!(t.ais.is_empty());
        let i = info(&t);
        assert_eq!(i["dry_run_source"], "producer");
        assert_eq!(i["matched_rule_id"], "anchor_campaign_suspend");
        assert_eq!(i["action_control_mode"], "rules");
    }
    let t = b.run_fresh(&bb1(&[&prefixed_label], Account::Clean)).await;
    assert_eq!(t.outcomes[0].labels, vec![prefixed_label.clone()]);
}

#[tokio::test]
async fn admission_strips_the_prefix_and_an_old_image_drops_it() {
    let _serial = overturn_hold::tests::METRICS_LOCK.lock().await;
    let label = suspend_label();
    let b = bed(vec![], false).await;
    let mut p = b.processor();
    p.labels = HashSet::from([label.clone()]);
    let prefixed = format!("dry_run__{label}");
    let msg = |labels: &[&str]| xai_kafka::KafkaMessage {
        topic: p.topic.clone(),
        payload: Some(bb1(labels, Account::Clean).encode_to_vec()),
        ..Default::default()
    };
    p.process_batch(&[msg(&[&prefixed])]).await.unwrap();
    let outcomes = std::mem::take(&mut *b.outcomes.lock().unwrap());
    assert_eq!(outcomes.len(), 1, "admitted");
    assert_eq!(outcomes[0].status, "dry_run");
    assert_eq!(outcomes[0].info["dry_run_source"], "producer");
    assert!(b.ais.take().is_empty());
    assert!(!p.labels.contains(&prefixed));
    p.process_batch(&[msg(&["unsubscribed"])]).await.unwrap();
    assert!(
        b.outcomes.lock().unwrap().is_empty(),
        "unsubscribed labels are still dropped"
    );
}

#[tokio::test]
async fn live_page_actions_replace_the_rules_and_pass_every_gate() {
    let _serial = overturn_hold::tests::METRICS_LOCK.lock().await;
    let label = suspend_label();
    let b = bed(
        vec![
            live(BB1.0, None, vec![SHR7, LIVENESS]),
            live(crate::replay_tests::SWITCHED_OFF_HEAD, None, vec![LIVENESS]),
        ],
        false,
    )
    .await;
    let t = b.run_fresh(&bb1(&[&label], Account::Clean)).await;
    assert_eq!(t.status, "success");
    assert_eq!(
        ais_names(&t),
        vec!["addLabelsV2", "bounceViaTemplate"],
        "label, then liveness check"
    );
    assert_eq!(t.tickets, 1);
    assert_eq!(
        t.dedup_calls,
        vec!["try_insert(class=2)=new", "update(class=2,own_claim=true)"]
    );
    let i = info(&t);
    assert_eq!(i["matched_rule_id"], "page");
    assert_eq!(i["rules_would_rule_id"], "anchor_campaign_suspend");
    assert_eq!(i["action_control_mode"], "live");
    assert_eq!(i["action_control_scope"], "head");
    assert!(!i.contains_key("dry_run_source"));

    for (account, reason) in [
        (Account::Followers5k, "high_follower_count"),
        (Account::Followers200k, "very_high_follower_count"),
        (Account::HighCred, "pagerank_skipped"),
        (Account::NotFound, "user_not_found"),
    ] {
        let t = b.run_fresh(&bb1(&[&label], account)).await;
        assert_eq!(t.status, reason, "{account:?}");
        assert_eq!(info(&t)["matched_rule_id"], reason);
        assert!(t.ais.is_empty());
        assert_eq!(t.tickets, 0);
    }
    let off = message(
        EntityType::User,
        crate::replay_tests::SWITCHED_OFF_HEAD,
        "clf_v1",
        &[&label],
        Account::Clean,
    );
    let t = b.run_fresh(&off).await;
    assert_eq!(t.status, off_switch::STATUS_SWITCHED_OFF);
    assert_eq!(info(&t)["action_specs"], r#"[{"kind":"liveness_check"}]"#);
    assert!(t.ais.is_empty());

    let b = bed(vec![live(BB1.0, None, vec![SUSPEND])], false).await;
    let t = b.run_fresh(&bb1(&[&label], Account::Suspended)).await;
    assert_eq!(t.status, ALREADY_SUSPENDED);
    assert!(t.ais.is_empty());

    let b = bed_with(
        vec![live(BB1.0, None, vec![SUSPEND])],
        on_config(false, Some(0)),
        BAKED_USER_RULES,
    )
    .await;
    let t = b.run_fresh(&bb1(&[&label], Account::Clean)).await;
    assert_eq!(t.status, "max_enforcement_reached");
    assert!(t.ais.is_empty());
}

#[tokio::test]
async fn repeat_label_skip_is_label_only() {
    let _serial = overturn_hold::tests::METRICS_LOCK.lock().await;
    let label = suspend_label();
    let b = bed(vec![live(BB1.0, None, vec![SHR7])], false).await;
    let t = b
        .run_fresh(&bb1(&[&label], Account::HasSpamHighRecall))
        .await;
    assert_eq!(t.status, "already_spam_high_recall_labeled");
    assert!(t.ais.is_empty());
    let b = bed(vec![live(BB1.0, None, vec![SHR7, LIVENESS])], false).await;
    let t = b
        .run_fresh(&bb1(&[&label], Account::HasSpamHighRecall))
        .await;
    assert_eq!(t.status, "success");
    assert_eq!(
        ais_names(&t),
        vec!["addLabelsV2", "bounceViaTemplate"],
        "a composite is never split"
    );
}

#[tokio::test]
async fn band_records_split_a_two_band_head() {
    let _serial = overturn_hold::tests::METRICS_LOCK.lock().await;
    let b = bed(
        vec![live(BB1.0, Some("anchor_campaign_user"), vec![LIVENESS])],
        false,
    )
    .await;
    let t = b
        .run_fresh(&bb1(&["anchor_campaign_user"], Account::Clean))
        .await;
    assert_eq!(ais_names(&t), vec!["bounceViaTemplate"]);
    assert_eq!(info(&t)["action_control_scope"], "band");
    let t = b
        .run_fresh(&bb1(&["anchor_campaign_suspend"], Account::Clean))
        .await;
    assert_eq!(
        ais_names(&t),
        vec!["suspendUser"],
        "the other band follows the rules"
    );
    assert_eq!(info(&t)["matched_rule_id"], "anchor_campaign_suspend");
    assert_eq!(info(&t)["action_control_mode"], "rules");
}

#[tokio::test]
async fn global_dry_run_records_only_like_the_other_two_sources() {
    let _serial = overturn_hold::tests::METRICS_LOCK.lock().await;
    let label = suspend_label();
    for (records, want_peek) in [
        (
            vec![live(BB1.0, None, vec![LIVENESS])],
            "peek(class=2)=None",
        ),
        (vec![], "peek(class=3)=None"),
    ] {
        let b = bed(records, true).await;
        let t = b.run_fresh(&bb1(&[&label], Account::Clean)).await;
        assert_eq!(t.status, "dry_run");
        assert!(t.ais.is_empty());
        let i = info(&t);
        assert_eq!(i["dry_run_source"], "global");
        assert_eq!(
            t.dedup_calls,
            vec![want_peek],
            "a peek, never a claim or a write"
        );
        assert_eq!(t.tickets, 0, "no daily-cap ticket");
        assert_eq!(i["would_dedup_skip"], "false");
        assert_eq!(i["would_rate_cap"], "false");

        b.dedup.hold(EntityType::User, Account::Clean.user_id(), 3);
        let t = b.run(&bb1(&[&label], Account::Clean)).await;
        assert_eq!(t.status, "dry_run");
        assert_eq!(t.dedup_calls, vec![want_peek.replace("None", "Some(3)")]);
        assert_eq!(t.tickets, 0);
        assert_eq!(info(&t)["would_dedup_skip"], "true");
        assert_eq!(info(&t)["would_dedup_held_class"], "3");
    }
}

const RULES_WITH_A_CSE_SUSPEND: &str = r#"
for_entity: user
rules:
  - id: user_in_allowlist
    when: user_allowlist.is_allowlisted
    then: { kind: skip, reason: user_in_allowlist }
  - id: user_not_found
    when: "!user.present"
    then: { kind: skip, reason: user_not_found }
  - id: very_high_follower_count
    when: cred.follower_count >= 12345.67
    then: { kind: skip, reason: very_high_follower_count }
  - id: high_follower_count
    when: cred.follower_count >= 12.34
    then: { kind: skip, reason: high_follower_count }
  - id: pagerank_skipped
    when: cred.is_high || cred.score >= 50.0
    then: { kind: skip, reason: pagerank_skipped }
  - id: act_cse_label
    when: '"cse_label" in score.labels'
    then: { kind: act_suspend_user, perm: true, policy: "Cse" }
  - id: terminal
    when: "true"
    then: { kind: skip, reason: no_match }
"#;

#[tokio::test]
async fn a_cse_action_chosen_by_the_rules_leaves_a_live_page_setting_acting() {
    let _serial = overturn_hold::tests::METRICS_LOCK.lock().await;
    let b = bed_with(
        vec![live("SomeHead", None, vec![SUSPEND])],
        on_config(false, None),
        RULES_WITH_A_CSE_SUSPEND,
    )
    .await;
    let t = b
        .run_fresh(&message(
            EntityType::User,
            "SomeHead",
            "clf",
            &["cse_label"],
            Account::Clean,
        ))
        .await;
    assert_eq!(t.status, "success");
    assert!(!t.ais.is_empty());
    let i = info(&t);
    assert_eq!(i["matched_rule_id"], "page");
    assert_eq!(i["rules_would_rule_id"], "act_cse_label");
    assert_eq!(i["action_control_mode"], "live");
    assert!(!i.contains_key("action_control_forced_recording"), "{i:?}");
}

#[tokio::test]
async fn a_rules_eval_error_leaves_a_live_page_setting_acting() {
    let _serial = overturn_hold::tests::METRICS_LOCK.lock().await;
    let rules = RULES_WITH_A_CSE_SUSPEND.replace(
        r#"'"cse_label" in score.labels'"#,
        r#"'int("not a number") == 1'"#,
    );
    let seen_calls = || {
        ["queued", "recent", "dropped", "invalid"]
            .iter()
            .map(|l| {
                metrics::ACTION_CONTROL_SEEN_TOTAL
                    .with_label_values(&[l])
                    .get()
            })
            .sum::<u64>()
    };
    let b = bed_with(
        vec![live("SomeHead", None, vec![SUSPEND])],
        on_config(false, None),
        &rules,
    )
    .await;
    let before = seen_calls();
    let t = b
        .run_fresh(&message(
            EntityType::User,
            "SomeHead",
            "clf",
            &["cse_label"],
            Account::Clean,
        ))
        .await;
    assert_eq!(
        seen_calls(),
        before,
        "no seen row when the rules couldn't be evaluated"
    );
    assert_eq!(t.status, "success");
    assert!(!t.ais.is_empty());
    let i = info(&t);
    assert_eq!(i["matched_rule_id"], "page");
    assert!(!i.contains_key("rules_would_rule_id"));
    assert!(!i.contains_key("action_control_forced_recording"), "{i:?}");
}

#[tokio::test]
async fn a_post_page_decision_skips_a_suspended_or_deactivated_author() {
    let _serial = overturn_hold::tests::METRICS_LOCK.lock().await;
    let head = "SpamEmbeddingPtosDistilled";
    let label = crate::replay_tests::rule_labels(BAKED_POST_RULES)
        .into_iter()
        .next()
        .expect("a post routing label");
    let b = bed(
        vec![live(
            head,
            None,
            vec![
                PageAction::PostSpamHighRecall {
                    lifetime: Lifetime::Days30,
                },
                PageAction::SuspendAuthor {},
            ],
        )],
        false,
    )
    .await;
    for (account, want) in [
        (Account::Suspended, ALREADY_SUSPENDED),
        (Account::Deactivated, "already_deactivated"),
    ] {
        let t = b
            .run_fresh(&message(EntityType::Post, head, "clf", &[&label], account))
            .await;
        assert_eq!(t.status, want, "{account:?}");
        assert!(t.ais.is_empty(), "{account:?}");
        assert_eq!(t.tickets, 0, "{account:?}");
    }
}

fn live_everywhere(actions: Vec<PageAction>) -> Vec<ActionControlRecord> {
    HEADS
        .iter()
        .map(|(h, _)| *h)
        .chain(std::iter::once(crate::replay_tests::SWITCHED_OFF_HEAD))
        .map(|h| live(h, None, actions.clone()))
        .collect()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn the_rollback_plan_keeps_an_old_image_from_acting_on_page_managed_heads() {
    let _serial = overturn_hold::tests::METRICS_LOCK.lock().await;
    let mut records = live_everywhere(vec![SUSPEND]);
    records.extend(HEADS.iter().map(|(h, _)| {
        record(
            h,
            None,
            None,
            ControlMode::Recording,
            vec![PageAction::PostSpamHighRecall {
                lifetime: Lifetime::Days30,
            }],
        )
    }));
    let on = on_config(false, None);
    let current = bed_with(records.clone(), on.clone(), BAKED_USER_RULES).await;
    let rules = crate::rules::RulesCache::new();
    let safety = action_control::SafetyChecksByEntity {
        user: safety_checks::status(EntityType::User, &rules.resolve(EntityType::User, None)),
        post: safety_checks::status(EntityType::Post, &rules.resolve(EntityType::Post, None)),
    };
    let list = current
        .ctx
        .action_control
        .list(
            &action_control::ControlConfig::from_config(Some(&on)),
            safety,
        )
        .await;
    let planned = crate::action_control_rollback::plan(&list, "operator@example.com");
    assert!(!planned.is_empty());
    let switches: Vec<off_switch::OffSwitchRecord> = planned
        .iter()
        .map(|p| off_switch::test_support::record(&p.head, p.classifier.as_deref(), true, None))
        .collect();
    let managed: std::collections::HashSet<&str> =
        records.iter().map(|r| r.head.as_str()).collect();
    let mut old_image = base_config(false, None);
    old_image["action_control"] = serde_json::json!({"ignore_store": true});
    let old_bed = |off_switches| {
        TestBed::new(BedConfig {
            user_rules: BAKED_USER_RULES,
            post_rules: BAKED_POST_RULES,
            config: old_image.clone(),
            off_switches,
            control_records: records.clone(),
        })
    };
    let (with_plan, without_plan) = (old_bed(switches).await, old_bed(vec![]).await);
    let (mut ran, mut acted_without_plan) = (0, 0);
    for sc in scenarios(BAKED_USER_RULES, BAKED_POST_RULES)
        .into_iter()
        .filter(|s| !s.dry_run && s.cap.is_none() && s.held_class.is_none())
        .filter(|s| managed.contains(s.id.split('|').nth(1).unwrap_or("")))
    {
        let t = with_plan.run_fresh(&sc.score).await;
        assert!(t.ais.is_empty(), "{}: {} {:?}", sc.id, t.status, t.ais);
        assert_eq!(t.tickets, 0, "{}", sc.id);
        acted_without_plan += usize::from(!without_plan.run_fresh(&sc.score).await.ais.is_empty());
        ran += 1;
    }
    assert!(ran > 100, "ran {ran}");
    assert!(
        acted_without_plan > 0,
        "without the switches the old image acts, so the test bites"
    );
}

async fn assert_no_ais_anywhere(
    config: serde_json::Value,
    user_rules: &str,
    age: Option<Duration>,
) -> usize {
    let b = bed_with(live_everywhere(vec![SUSPEND]), config, user_rules).await;
    if let Some(age) = age {
        b.ctx.action_control.age_for_test(age);
    }
    let mut n = 0;
    for sc in scenarios(user_rules, BAKED_POST_RULES)
        .into_iter()
        .filter(|s| s.id.starts_with("user|") && !s.id.contains("|factory|") && !s.dry_run)
    {
        let t = b.run_fresh(&sc.score).await;
        assert!(t.ais.is_empty(), "{}: {:?}", sc.id, t.ais);
        assert_eq!(t.tickets, 0, "{}", sc.id);
        assert!(
            t.dedup_calls.iter().all(|c| c.starts_with("peek")),
            "{}: {:?}",
            sc.id,
            t.dedup_calls
        );
        n += 1;
    }
    n
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn no_failure_state_adds_an_ais_call() {
    let _serial = overturn_hold::tests::METRICS_LOCK.lock().await;
    let mut killed = on_config(false, None);
    killed["action_control"] = serde_json::json!({"enabled": false});
    let mut malformed = on_config(false, None);
    malformed["action_control"] = serde_json::json!({"enabled": "off"});
    let missing_ceiling = BAKED_USER_RULES.replacen(
        "cred.follower_count >= ",
        "false && cred.follower_count >= ",
        1,
    );
    assert_ne!(missing_ceiling, BAKED_USER_RULES);
    let cases: Vec<(&str, serde_json::Value, &str, Option<Duration>)> = vec![
        ("kill switch", killed, BAKED_USER_RULES, None),
        (
            "no action_control object in GrowthBook",
            base_config(false, None),
            BAKED_USER_RULES,
            None,
        ),
        (
            "malformed GrowthBook action_control",
            malformed,
            BAKED_USER_RULES,
            None,
        ),
        (
            "stale copy",
            on_config(false, None),
            BAKED_USER_RULES,
            Some(action_control::STALE_AFTER + Duration::from_secs(1)),
        ),
        (
            "a safety skip loosened in the rules",
            on_config(false, None),
            &missing_ceiling,
            None,
        ),
    ];
    for (name, config, rules, age) in cases {
        let n = assert_no_ais_anywhere(config, rules, age).await;
        assert!(n > 100, "{name}: ran {n}");
    }
}

#[tokio::test]
async fn damaged_rules_still_stop_page_decisions_with_the_built_in_checks() {
    let _serial = overturn_hold::tests::METRICS_LOCK.lock().await;
    let loosened = BAKED_USER_RULES.replacen(
        "cred.follower_count >= ",
        "false && cred.follower_count >= ",
        1,
    );
    let b = bed_with(
        vec![record(
            BB1.0,
            None,
            None,
            ControlMode::Recording,
            vec![SUSPEND],
        )],
        on_config(false, None),
        &loosened,
    )
    .await;
    let t = b
        .run(&bb1(&[&suspend_label()], Account::Followers200k))
        .await;
    assert_eq!(t.status, "very_high_follower_count");
}


fn as_page_actions(specs: &serde_json::Value, entity: EntityType) -> Option<Vec<PageAction>> {
    let mut out = Vec::new();
    for s in specs.as_array()? {
        let kind = s.get("kind")?.as_str()?;
        let permanent = s
            .get("permanent")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let days = s.get("ttl_days").and_then(|v| v.as_i64());
        let lifetime = || match (permanent, days) {
            (true, _) => Some(Lifetime::Permanent),
            (false, Some(7)) => Some(Lifetime::Days7),
            (false, Some(30)) => Some(Lifetime::Days30),
            _ => None,
        };
        let label_ok = s.get("label").and_then(|v| v.as_str()) == Some("SpamHighRecall");
        out.push(match (kind, entity) {
            ("suspend", EntityType::User)
                if !permanent && s.get("policy")?.as_str()? == "PlatformManipulation" =>
            {
                PageAction::Suspend {}
            }
            ("suspend", EntityType::Post)
                if !permanent && s.get("policy")?.as_str()? == "PlatformManipulation" =>
            {
                PageAction::SuspendAuthor {}
            }
            ("liveness_check", EntityType::User) => PageAction::LivenessCheck {},
            ("label", EntityType::User) if label_ok => PageAction::SpamHighRecall {
                lifetime: lifetime()?,
            },
            ("post_label", EntityType::Post) if label_ok && lifetime()? != Lifetime::Days7 => {
                PageAction::PostSpamHighRecall {
                    lifetime: lifetime()?,
                }
            }
            _ => return None,
        });
    }
    (!out.is_empty()).then_some(out)
}

const SAFETY_REASONS: &[&str] = &[
    "user_in_allowlist",
    "post_in_allowlist",
    "user_not_found",
    "very_high_follower_count",
    "high_follower_count",
    "pagerank_skipped",
];

async fn move_to_page(
    user_rules: &str,
    post_rules: &str,
) -> BTreeMap<String, (usize, Vec<String>)> {
    let all = scenarios(user_rules, post_rules);
    let mut base: HashMap<String, Trace> = HashMap::new();
    let mut beds: HashMap<(bool, Option<u64>), TestBed> = HashMap::new();
    for sc in &all {
        let key = (sc.dry_run, sc.cap);
        if let std::collections::hash_map::Entry::Vacant(slot) = beds.entry(key) {
            slot.insert(
                TestBed::new(BedConfig {
                    user_rules,
                    post_rules,
                    config: on_config(sc.dry_run, sc.cap),
                    off_switches: vec![switched_off()],
                    control_records: vec![],
                })
                .await,
            );
        }
        let b = &beds[&key];
        let (et, id) = Facts::resolve_target(&sc.score);
        b.dedup.slots.lock().unwrap().clear();
        if let Some(c) = sc.held_class {
            b.dedup.hold(et, id, c);
        }
        base.insert(sc.id.clone(), b.run(&sc.score).await);
    }
    let key_of = |id: &str| id.split('|').take(3).collect::<Vec<_>>().join("|");
    let mut choice: HashMap<String, serde_json::Value> = HashMap::new();
    for sc in &all {
        if sc.id.contains("|Clean|dry=0|held=-")
            && let Some(t) = base.get(&sc.id)
            && t.status == "success"
            && let Some(specs) = t.outcomes[0].info.get("action_specs")
        {
            choice.insert(key_of(&sc.id), serde_json::from_str(specs).unwrap());
        }
    }
    let mut counts: BTreeMap<String, (usize, Vec<String>)> = BTreeMap::new();
    let bump = |counts: &mut BTreeMap<String, (usize, Vec<String>)>, k: String, id: &str| {
        let e = counts.entry(k).or_default();
        e.0 += 1;
        if e.1.len() < 3 {
            e.1.push(id.to_owned());
        }
    };
    let mut unexplained = Vec::new();
    for sc in &all {
        let parts: Vec<&str> = sc.id.split('|').collect();
        let (entity, head, label) = (parts[0], parts[1], parts[2]);
        let entity_type = if entity == "post" {
            EntityType::Post
        } else {
            EntityType::User
        };
        let Some(specs) = choice.get(&key_of(&sc.id)) else {
            bump(
                &mut counts,
                "not moved: the rules do not act on this label".into(),
                &sc.id,
            );
            continue;
        };
        let Some(actions) = as_page_actions(specs, entity_type) else {
            bump(
                &mut counts,
                "not moved: the rules' actions are outside the page menu".into(),
                &sc.id,
            );
            continue;
        };
        let head = if head == "factory" {
            "IsDelayedSuspendSpam"
        } else {
            head
        };
        let band = (!label.is_empty() && !label.starts_with("cap_full")).then_some(label);
        let rec = record(head, None, band, ControlMode::Live, actions);
        let b = &beds[&(sc.dry_run, sc.cap)];
        b.set_records(&[rec]).await;
        let (et, id) = Facts::resolve_target(&sc.score);
        b.dedup.slots.lock().unwrap().clear();
        if let Some(c) = sc.held_class {
            b.dedup.hold(et, id, c);
        }
        let moved = b.run(&sc.score).await;
        let was = &base[&sc.id];
        let same_ais = ais_names(was) == ais_names(&moved);
        let same_effect = was.status == moved.status && same_ais && was.tickets == moved.tickets;
        let category = if same_effect && was.dedup_calls == moved.dedup_calls {
            "identical".to_owned()
        } else if same_effect {
            "same status, actions and tickets; only the dedup calls differ".into()
        } else if entity == "post"
            && was.status == "high_follower_count"
            && moved.status == "very_high_follower_count"
            && moved.ais.is_empty()
            && moved.tickets == 0
        {
            "post author at the page's follower ceiling (the rules skip from a lower count)".into()
        } else if SAFETY_REASONS.contains(&was.status.as_str()) {
            if moved.status != was.status || !moved.ais.is_empty() || moved.tickets != 0 {
                unexplained.push(format!(
                    "{}: safety skip {} became {}",
                    sc.id, was.status, moved.status
                ));
                "UNEXPLAINED".into()
            } else {
                "same safety skip; only the dedup class or claim differs".into()
            }
        } else if (moved.status == ALREADY_SUSPENDED || moved.status == "already_deactivated")
            && moved.ais.is_empty()
        {
            "page decisions skip suspended or deactivated accounts (the rules acted or skipped differently)".into()
        } else if (was.status == "success" || was.status == "dry_run")
            && moved.status == "already_spam_high_recall_labeled"
        {
            "label-only page set skips an account that already has the label (the rule re-labels)"
                .into()
        } else if was.status == "dedup_skipped" && moved.status != "dedup_skipped" {
            "dedup class from the page's actions outranks the held entry (auto)".into()
        } else if head == "IsDelayedSuspendSpam" && moved.ais.is_empty() {
            "factory head: live record runs as recording only".into()
        } else if was.status == "already_deactivated" {
            "deactivated account: `already_deactivated` is a routing rule, not a safety check"
                .into()
        } else if was.status == "already_spam_high_recall_labeled" {
            "already-labelled skip of a label-only rule vs the page's set".into()
        } else {
            unexplained.push(format!(
                "{}: {} {:?} cap={} {:?} → {} {:?} cap={} {:?}",
                sc.id,
                was.status,
                ais_names(was),
                was.tickets,
                was.dedup_calls,
                moved.status,
                ais_names(&moved),
                moved.tickets,
                moved.dedup_calls
            ));
            "UNEXPLAINED".into()
        };
        bump(&mut counts, category, &sc.id);
    }
    assert!(
        unexplained.is_empty(),
        "differences outside the approved list:\n{}",
        unexplained.join("\n")
    );
    counts
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn moving_every_head_to_the_page_with_its_current_actions() {
    let _serial = overturn_hold::tests::METRICS_LOCK.lock().await;
    let counts = move_to_page(BAKED_USER_RULES, BAKED_POST_RULES).await;
    let summary: String = counts
        .iter()
        .map(|(k, (n, ex))| format!("{n:>5}  {k}\n         e.g. {}\n", ex.join("  ")))
        .collect();
    eprintln!("move-to-page replay (baked rules):\n{summary}");
    if let Ok(path) = std::env::var("ACTION_CONTROL_MOVE_REPORT") {
        std::fs::write(path, &summary).unwrap();
    }
    let same = |k: &str| counts.get(k).map_or(0, |c| c.0);
    assert!(
        same("identical") + same("same status, actions and tickets; only the dedup calls differ")
            > 100,
        "{counts:?}"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn moving_every_head_to_the_page_live_rules() {
    let Ok(dir) = std::env::var("ACTION_CONTROL_REPLAY_RULES_DIR") else {
        return;
    };
    let _serial = overturn_hold::tests::METRICS_LOCK.lock().await;
    let user = std::fs::read_to_string(format!("{dir}/rule_user.yaml")).unwrap();
    let post = std::fs::read_to_string(format!("{dir}/rule_post.yaml")).unwrap();
    let counts = move_to_page(&user, &post).await;
    let summary: String = counts
        .iter()
        .map(|(k, (n, ex))| format!("{n:>5}  {k}\n         e.g. {}\n", ex.join("  ")))
        .collect();
    eprintln!("move-to-page replay (live rules):\n{summary}");
    if let Ok(path) = std::env::var("ACTION_CONTROL_MOVE_REPORT_LIVE") {
        std::fs::write(path, &summary).unwrap();
    }
}

#[tokio::test]
async fn a_recording_decision_never_writes_or_releases_the_slot_on_retry() {
    let decision = DecisionContext {
        control: action_control::Resolved {
            mode: ControlMode::Recording,
            ..Default::default()
        },
        ..Default::default()
    };
    assert!(!decision.claimed());
    let producer = DecisionContext {
        control: action_control::Resolved {
            producer_dry: true,
            ..Default::default()
        },
        ..Default::default()
    };
    assert!(!producer.claimed());
    let global = DecisionContext {
        global_dry_run: true,
        ..Default::default()
    };
    assert!(!global.claimed());
    assert!(DecisionContext::default().claimed());
}

#[tokio::test]
async fn a_decision_pinned_to_global_dry_run_never_acts_after_the_flag_flips() {
    let _serial = overturn_hold::tests::METRICS_LOCK.lock().await;
    let label = suspend_label();
    let b = bed(vec![], false).await;
    let ctx = &b.ctx;
    let score = bb1(&[&label], Account::Clean);
    let pinned = DecisionContext {
        global_dry_run: true,
        ..Default::default()
    };
    let tickets = b.tickets.load(std::sync::atomic::Ordering::Relaxed);
    let out = run_enforcement(ctx, &score, "t", &pinned).await.unwrap();
    assert_eq!(out.status, "dry_run");
    assert!(out.dry_run);
    assert_eq!(out.info["dry_run_source"], "global");
    assert_eq!(out.info["would_dedup_skip"], "false");
    assert!(b.ais.take().is_empty(), "no AIS call");
    assert_eq!(
        b.tickets.load(std::sync::atomic::Ordering::Relaxed),
        tickets,
        "no daily-cap ticket"
    );
    let live = run_enforcement(ctx, &score, "t", &DecisionContext::default())
        .await
        .unwrap();
    assert_eq!(
        live.status, "success",
        "the same message acts when not pinned"
    );
}

const RULES_FACT_GATED_SUSPEND: &str = r#"
for_entity: user
rules:
  - id: user_not_found
    when: "!user.present"
    then: { kind: skip, reason: user_not_found }
  - id: act_suspend_if_labeled
    when: '"fact_gated_suspend" in score.labels && "SpamHighRecall" in user.labels'
    then: { kind: act_suspend_user, perm: false, policy: "PlatformManipulation" }
  - id: act_liveness_fallback
    when: '"fact_gated_suspend" in score.labels'
    then: { kind: act_spam_liveness_check }
  - id: terminal
    when: "true"
    then: { kind: skip, reason: no_match }
"#;

async fn fact_gated_bed() -> TestBed {
    bed_with(vec![], base_config(false, None), RULES_FACT_GATED_SUSPEND).await
}

fn held(b: &TestBed, account: Account) -> Option<(u8, bool, Option<u64>)> {
    b.dedup
        .slots
        .lock()
        .unwrap()
        .get(&(EntityType::User.as_str().to_owned(), account.user_id()))
        .copied()
}

#[tokio::test]
async fn a_weaker_fallback_chosen_after_the_fetch_is_deduped_against_an_equal_held_entry() {
    let _serial = overturn_hold::tests::METRICS_LOCK.lock().await;
    let b = fact_gated_bed().await;
    b.dedup.slots.lock().unwrap().clear();
    b.dedup.hold(
        EntityType::User,
        Account::Clean.user_id(),
        dedup_cache::CLASS_CHALLENGE,
    );
    let t = b.run(&bb1(&["fact_gated_suspend"], Account::Clean)).await;
    assert_eq!(t.status, "dedup_skipped");
    assert_eq!(
        t.dedup_calls,
        vec![
            "try_insert(class=3)=bypass",
            "update_skip(class=0,own_claim=true)"
        ]
    );
    assert!(t.ais.is_empty(), "no AIS call: {:?}", t.ais);
    assert_eq!(t.tickets, 0, "no daily-cap ticket");
    assert_eq!(info(&t)["matched_rule_id"], "act_liveness_fallback");
    assert_eq!(info(&t)["dedup_held_class"], "2");
    assert_eq!(
        held(&b, Account::Clean),
        Some((dedup_cache::CLASS_CHALLENGE, false, None)),
        "the held liveness check is put back"
    );
}

#[tokio::test]
async fn a_fact_gated_suspend_that_fires_acts_over_a_held_liveness_check() {
    let _serial = overturn_hold::tests::METRICS_LOCK.lock().await;
    let b = fact_gated_bed().await;
    b.dedup.slots.lock().unwrap().clear();
    b.dedup.hold(
        EntityType::User,
        Account::HasSpamHighRecall.user_id(),
        dedup_cache::CLASS_CHALLENGE,
    );
    let t = b
        .run(&bb1(&["fact_gated_suspend"], Account::HasSpamHighRecall))
        .await;
    assert_eq!(t.status, "success");
    assert_eq!(ais_names(&t), vec!["suspendUser"]);
    assert_eq!(t.tickets, 1);
    assert_eq!(
        t.dedup_calls,
        vec![
            "try_insert(class=3)=bypass",
            "update(class=3,own_claim=true)"
        ]
    );
    assert_eq!(
        held(&b, Account::HasSpamHighRecall),
        Some((dedup_cache::CLASS_SUSPEND, false, None))
    );

    let t = b
        .run_fresh(&bb1(&["fact_gated_suspend"], Account::Clean))
        .await;
    assert_eq!(t.status, "success");
    assert_eq!(ais_names(&t), vec!["bounceViaTemplate"]);
    assert_eq!(t.tickets, 1);
}
