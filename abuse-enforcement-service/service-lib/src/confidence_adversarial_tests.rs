use xai_abuse_proto::enforcement::{self as proto, ScoreResult};

use super::*;
use crate::action_control::test_support::record;
use crate::action_control::{
    ActionControlRecord, ConfidenceEvidence, ControlMode, EvidenceOverride, LevelRow, PageAction,
};
use crate::confidence::ConfidenceLevel;
use crate::replay_tests::{
    message, on_config, Account, BedConfig, TestBed, BAKED_POST_RULES, BAKED_USER_RULES, HEADS,
};

const DAY_MS: i64 = 86_400_000;
const BB1: (&str, &str) = HEADS[0];
const LABEL: &str = "anchor_campaign_suspend";

fn evidence(level: ConfidenceLevel) -> ConfidenceEvidence {
    let (lower_bound, reads, humans) = match level {
        ConfidenceLevel::Low => (0.72, 12, 0),
        ConfidenceLevel::Mid => (0.88, 50, 2),
        ConfidenceLevel::High => (0.951, 59, 0),
    };
    ConfidenceEvidence {
        precision_lower_bound: lower_bound,
        reads,
        humans,
        read_at_ms: now_millis() - DAY_MS,
        link: "documents/read.md".into(),
    }
}

fn proto_evidence(e: &ConfidenceEvidence) -> proto::ConfidenceEvidence {
    proto::ConfidenceEvidence {
        precision_lower_bound: e.precision_lower_bound,
        reads: e.reads,
        humans: e.humans,
        read_at_ms: e.read_at_ms,
        link: e.link.clone(),
    }
}

fn high() -> ScoreResult {
    let mut score = message(EntityType::User, BB1.0, BB1.1, &[LABEL], Account::Clean);
    score.confidence = proto::Confidence::High as i32;
    score.confidence_evidence = Some(proto_evidence(&evidence(ConfidenceLevel::High)));
    score
}

fn moved() -> ActionControlRecord {
    record(BB1.0, None, None, ControlMode::Rows, vec![])
}

fn high_row() -> ActionControlRecord {
    ActionControlRecord {
        confidence: Some(LevelRow::High),
        ..record(
            BB1.0,
            None,
            None,
            ControlMode::Live,
            vec![PageAction::Suspend {}],
        )
    }
}

async fn bed(config: serde_json::Value, row: ActionControlRecord) -> TestBed {
    TestBed::new(BedConfig {
        user_rules: BAKED_USER_RULES,
        post_rules: BAKED_POST_RULES,
        config,
        off_switches: vec![],
        control_records: vec![moved(), row],
    })
    .await
}

#[tokio::test]
async fn malformed_confidence_config_forces_live_level_rows_to_record_only() {
    let _serial = overturn_hold::tests::METRICS_LOCK.lock().await;
    let mut config = on_config(false, None);
    config["action_control"]["confidence"] = serde_json::json!({
        "label_map": {LABEL: "high"}
    });
    let trace = bed(config, high_row()).await.run_fresh(&high()).await;
    assert_eq!(trace.status, "dry_run", "{trace:?}");
    assert!(trace.ais.is_empty(), "{trace:?}");
    assert_eq!(trace.tickets, 0, "{trace:?}");
    assert!(
        trace
            .dedup_calls
            .iter()
            .all(|call| call.starts_with("peek")),
        "{trace:?}"
    );
    let info = &trace.outcomes[0].info;
    assert_eq!(info["logged_because"], "weak_evidence");
    assert_eq!(
        info[action_control::FORCED_RECORDING_KEY],
        "confidence_config_unreadable"
    );
}

#[tokio::test]
async fn a_future_record_timestamp_cannot_extend_an_override_past_seven_days() {
    let _serial = overturn_hold::tests::METRICS_LOCK.lock().await;
    let now = now_millis();
    let mut row = high_row();
    row.updated_at_ms = now + 30 * DAY_MS;
    row.evidence_override = Some(Box::new(EvidenceOverride {
        by: "operator@example.com".into(),
        reason: "temporary exception".into(),
        expires_at_ms: now + 30 * DAY_MS,
    }));
    let mut score = high();
    score.confidence_evidence = None;

    let trace = bed(on_config(false, None), row)
        .await
        .run_fresh(&score)
        .await;
    assert_eq!(trace.status, action_control::STATUS_NO_ACTIONS, "{trace:?}");
    assert!(trace.ais.is_empty(), "{trace:?}");
    assert_eq!(trace.tickets, 0, "{trace:?}");
    assert_eq!(trace.outcomes[0].info["logged_because"], "unreadable");
}

#[tokio::test]
async fn an_override_expiring_before_its_future_write_cannot_act() {
    let _serial = overturn_hold::tests::METRICS_LOCK.lock().await;
    let now = now_millis();
    let mut row = high_row();
    row.updated_at_ms = now + crate::confidence::CLOCK_SKEW_MS;
    row.evidence_override = Some(Box::new(EvidenceOverride {
        by: "operator@example.com".into(),
        reason: "temporary exception".into(),
        expires_at_ms: now + 60_000,
    }));
    let mut score = high();
    score.confidence_evidence = None;

    let trace = bed(on_config(false, None), row)
        .await
        .run_fresh(&score)
        .await;
    assert_eq!(trace.status, action_control::STATUS_NO_ACTIONS, "{trace:?}");
    assert!(trace.ais.is_empty(), "{trace:?}");
    assert_eq!(trace.tickets, 0, "{trace:?}");
}

#[test]
fn five_minute_and_seven_day_boundaries_are_exact() {
    let now = 1_791_000_000_000;
    let mut ev = evidence(ConfidenceLevel::High);
    ev.read_at_ms = now + crate::confidence::CLOCK_SKEW_MS;
    assert_eq!(
        crate::confidence::supported_level(
            &ev,
            now,
            crate::confidence::DEFAULT_EVIDENCE_MAX_AGE_DAYS * DAY_MS,
        ),
        Some(ConfidenceLevel::High)
    );
    ev.read_at_ms += 1;
    assert_eq!(
        crate::confidence::supported_level(
            &ev,
            now,
            crate::confidence::DEFAULT_EVIDENCE_MAX_AGE_DAYS * DAY_MS,
        ),
        None
    );

    let future_write = now + crate::confidence::CLOCK_SKEW_MS;
    let mut ov = EvidenceOverride {
        by: "operator@example.com".into(),
        reason: "temporary exception".into(),
        expires_at_ms: now + 7 * DAY_MS,
    };
    assert!(crate::confidence::override_active(&ov, future_write, now));
    ov.expires_at_ms += 1;
    assert!(!crate::confidence::override_active(&ov, future_write, now));

    ov.expires_at_ms = now + 60_000;
    assert!(
        !crate::confidence::override_active(&ov, future_write, now),
        "an override expiring before its stored write time must not act"
    );
}
