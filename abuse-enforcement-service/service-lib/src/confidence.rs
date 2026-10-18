use std::sync::Mutex;

use serde_json::Value;
use tracing::error;
use xai_abuse_proto::enforcement as abuse_proto;

pub use xai_abuse_enforcement_types::action_control::{
    ConfidenceEvidence, ConfidenceLevel, EvidenceOverride, LevelRow, EVIDENCE_OVERRIDE_MAX_MS,
};

pub const CONFIG_KEY: &str = "confidence";
pub const DEFAULT_EVIDENCE_MAX_AGE_DAYS: i64 = 30;
const DAY_MS: i64 = 86_400_000;
const MAX_EVIDENCE_AGE_DAYS: f64 = 365.0;
pub const CLOCK_SKEW_MS: i64 = 300_000;
const CONFIG_KEYS: &[&str] = &["evidence_max_age_days"];
const MAX_READS: u32 = 100_000;
const ALPHA: f64 = 0.05;

#[derive(Debug, Clone, PartialEq)]
pub struct Claim {
    pub row: LevelRow,
    pub unknown: bool,
    pub evidence: Option<ConfidenceEvidence>,
    pub now_ms: i64,
}

pub fn row_of(value: i32) -> (LevelRow, bool) {
    match abuse_proto::Confidence::try_from(value) {
        Ok(abuse_proto::Confidence::Unspecified) => (LevelRow::Missing, false),
        Ok(abuse_proto::Confidence::Low) => (LevelRow::Low, false),
        Ok(abuse_proto::Confidence::Mid) => (LevelRow::Mid, false),
        Ok(abuse_proto::Confidence::High) => (LevelRow::High, false),
        Err(_) => (LevelRow::Missing, true),
    }
}

pub fn evidence_from_proto(e: &abuse_proto::ConfidenceEvidence) -> ConfidenceEvidence {
    ConfidenceEvidence {
        precision_lower_bound: e.precision_lower_bound,
        reads: e.reads,
        humans: e.humans,
        read_at_ms: e.read_at_ms,
        link: e.link.clone(),
    }
}

pub fn message_claim(score: &abuse_proto::ScoreResult, now_ms: i64) -> Claim {
    let (row, unknown) = row_of(score.confidence);
    Claim {
        row,
        unknown,
        evidence: score.confidence_evidence.as_ref().map(evidence_from_proto),
        now_ms,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfidenceConfig {
    pub evidence_max_age_ms: i64,
    pub unreadable: Option<String>,
}

impl Default for ConfidenceConfig {
    fn default() -> Self {
        Self {
            evidence_max_age_ms: DEFAULT_EVIDENCE_MAX_AGE_DAYS * DAY_MS,
            unreadable: None,
        }
    }
}

impl ConfidenceConfig {
    pub fn from_config(config: Option<&Value>) -> Self {
        Self::from_value(
            config
                .and_then(|c| c.get(crate::action_control::CONFIG_KEY))
                .and_then(|a| a.get(CONFIG_KEY)),
        )
    }

    pub fn from_value(v: Option<&Value>) -> Self {
        static LAST_BAD: Mutex<Option<String>> = Mutex::new(None);
        let mut out = Self::default();
        let Some(v) = v.filter(|v| !v.is_null()) else {
            return out;
        };
        let mut bad = Vec::new();
        match v.as_object() {
            Some(o) => bad.extend(
                o.keys()
                    .filter(|k| !CONFIG_KEYS.contains(&k.as_str()))
                    .map(|k| format!("unknown key {k}")),
            ),
            None => bad.push(v.to_string()),
        }
        match v.get("evidence_max_age_days") {
            None | Some(Value::Null) => {}
            Some(days) => match days
                .as_f64()
                .filter(|d| d.is_finite() && *d > 0.0 && *d <= MAX_EVIDENCE_AGE_DAYS)
            {
                Some(d) => out.evidence_max_age_ms = (d * DAY_MS as f64) as i64,
                None => bad.push(format!("evidence_max_age_days={days}")),
            },
        }
        if bad.is_empty() {
            return out;
        }
        let bad = bad.join(", ");
        let mut last = LAST_BAD.lock().unwrap_or_else(|p| p.into_inner());
        if last.as_deref() != Some(bad.as_str()) {
            error!(
                value = %bad,
                "GrowthBook action_control.{CONFIG_KEY} is malformed; every live level row records only"
            );
            *last = Some(bad.clone());
        }
        Self {
            evidence_max_age_ms: 0,
            unreadable: Some(bad),
        }
    }
}

fn upper_tail(s: u32, n: u32, p: f64) -> f64 {
    if s == 0 {
        return 1.0;
    }
    if p <= 0.0 {
        return 0.0;
    }
    if p >= 1.0 {
        return 1.0;
    }
    let mut ln_fact = Vec::with_capacity(n as usize + 1);
    ln_fact.push(0.0f64);
    for i in 1..=n {
        ln_fact.push(ln_fact[i as usize - 1] + f64::from(i).ln());
    }
    let (lp, lq) = (p.ln(), (1.0 - p).ln());
    (s..=n)
        .map(|k| {
            let ln_c = ln_fact[n as usize] - ln_fact[k as usize] - ln_fact[(n - k) as usize];
            (ln_c + f64::from(k) * lp + f64::from(n - k) * lq).exp()
        })
        .sum()
}

fn read_meets(reads: u32, humans: u32, floor: f64) -> bool {
    upper_tail(reads - humans, reads, floor) <= ALPHA
}

pub fn supported_level(
    ev: &ConfidenceEvidence,
    now_ms: i64,
    max_age_ms: i64,
) -> Option<ConfidenceLevel> {
    let lb = ev.precision_lower_bound;
    if !(lb.is_finite() && (0.0..=1.0).contains(&lb))
        || ev.reads == 0
        || ev.reads > MAX_READS
        || ev.humans > ev.reads
        || ev.read_at_ms <= 0
        || ev.read_at_ms > now_ms.saturating_add(CLOCK_SKEW_MS)
        || now_ms.saturating_sub(ev.read_at_ms) > max_age_ms
    {
        return None;
    }
    ConfidenceLevel::ALL
        .into_iter()
        .rev()
        .find(|l| lb >= l.floor() && read_meets(ev.reads, ev.humans, l.floor()))
}

pub fn evidence_problem(ev: &ConfidenceEvidence, now_ms: i64) -> Option<String> {
    let lb = ev.precision_lower_bound;
    if !(lb.is_finite() && (0.0..=1.0).contains(&lb)) {
        return Some("precision_lower_bound must be a number from 0 to 1".into());
    }
    if ev.reads == 0 || ev.reads > MAX_READS {
        return Some(format!("reads must be 1 to {MAX_READS}"));
    }
    if ev.humans > ev.reads {
        return Some("humans can't exceed reads".into());
    }
    if ev.read_at_ms <= 0 || ev.read_at_ms > now_ms.saturating_add(CLOCK_SKEW_MS) {
        return Some("read_at_ms must be the read's finish time, not in the future".into());
    }
    if ev.link.len() > 1_000 {
        return Some("link is longer than 1000 bytes".into());
    }
    None
}

pub fn override_active(ov: &EvidenceOverride, record_updated_at_ms: i64, now_ms: i64) -> bool {
    let written = record_updated_at_ms;
    !ov.by.trim().is_empty()
        && !ov.reason.trim().is_empty()
        && now_ms < ov.expires_at_ms
        && written < ov.expires_at_ms
        && written <= now_ms.saturating_add(CLOCK_SKEW_MS)
        && now_ms >= written.saturating_sub(CLOCK_SKEW_MS)
        && ov.expires_at_ms <= written.min(now_ms).saturating_add(EVIDENCE_OVERRIDE_MAX_MS)
}

pub fn override_problem(ov: &EvidenceOverride, now_ms: i64) -> Option<String> {
    if ov.by.trim().is_empty() || ov.by.len() > 256 {
        return Some("evidence_override.by must name the person (1-256 bytes)".into());
    }
    if ov.reason.trim().is_empty() || ov.reason.len() > 1_000 {
        return Some("evidence_override.reason must say why (1-1000 bytes)".into());
    }
    if ov.expires_at_ms <= now_ms || ov.expires_at_ms > now_ms + EVIDENCE_OVERRIDE_MAX_MS {
        return Some("evidence_override.expires_at_ms must be in the next 7 days".into());
    }
    None
}

#[derive(Debug, Clone, PartialEq)]
pub struct RowOutcome {
    pub row: LevelRow,
    pub unknown: bool,
    pub evidence_supports: Option<Option<ConfidenceLevel>>,
    pub override_by: Option<String>,
}

impl RowOutcome {
    pub fn new(claim: &Claim) -> Self {
        Self {
            row: claim.row,
            unknown: claim.unknown,
            evidence_supports: None,
            override_by: None,
        }
    }

    pub fn info_keys(&self, logged: bool, forced: Option<&str>) -> Vec<(String, String)> {
        let mut keys = vec![("level_row".to_owned(), self.row.as_str().to_owned())];
        if logged {
            keys.push((
                "logged_because".to_owned(),
                logged_because(forced, self.unknown).to_owned(),
            ));
        }
        if let Some(s) = self.evidence_supports {
            keys.push((
                "confidence_evidence_supports".to_owned(),
                s.map_or("none", ConfidenceLevel::as_str).to_owned(),
            ));
        }
        if let Some(by) = &self.override_by {
            keys.push(("confidence_override_by".to_owned(), by.clone()));
        }
        keys
    }
}

pub fn logged_because(forced: Option<&str>, unknown: bool) -> &'static str {
    match forced {
        None if unknown => "unknown_level",
        None => "no_selection",
        Some("kill_switch") => "page_control_off",
        Some("stale_snapshot") => "stale_copy",
        Some("safety_checks_changed") => "safety_checks_changed",
        Some("evidence_missing" | "evidence_below_floor" | "confidence_config_unreadable") => {
            "weak_evidence"
        }
        Some("factory_managed") => "factory_managed",
        Some("wrong_entity") => "wrong_entity",
        Some(_) => "unreadable",
    }
}

pub const INFO_KEYS: &[&str] = &[
    "level_row",
    "logged_because",
    "confidence_evidence_supports",
    "confidence_override_by",
];

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const NOW: i64 = 1_791_000_000_000;

    fn ev(lb: f64, reads: u32, humans: u32) -> ConfidenceEvidence {
        ConfidenceEvidence {
            precision_lower_bound: lb,
            reads,
            humans,
            read_at_ms: NOW - DAY_MS,
            link: "documents/x".into(),
        }
    }

    fn supported(e: &ConfidenceEvidence) -> Option<ConfidenceLevel> {
        supported_level(e, NOW, DEFAULT_EVIDENCE_MAX_AGE_DAYS * DAY_MS)
    }

    #[test]
    fn the_memo_read_counts_meet_their_floors_and_one_fewer_does_not() {
        use ConfidenceLevel::*;
        assert_eq!(supported(&ev(0.95, 59, 0)), Some(High));
        assert_eq!(supported(&ev(0.95, 93, 1)), Some(High));
        assert_eq!(supported(&ev(0.95, 124, 2)), Some(High));
        assert_eq!(
            supported(&ev(0.95, 58, 0)),
            Some(Mid),
            "0 of 58 is under 0.95"
        );
        assert_eq!(supported(&ev(0.85, 19, 0)), Some(Mid));
        assert_eq!(supported(&ev(0.85, 50, 3)), Some(Mid));
        assert_eq!(supported(&ev(0.85, 68, 5)), Some(Mid));
        assert_eq!(supported(&ev(0.85, 18, 0)), Some(Low));
        assert_eq!(supported(&ev(0.70, 9, 0)), Some(Low));
        assert_eq!(supported(&ev(0.70, 8, 0)), None);
        assert_eq!(supported(&ev(0.70, 50, 25)), None);
    }

    #[test]
    fn a_typed_bound_cannot_claim_more_than_its_reads_show() {
        assert_eq!(
            supported(&ev(0.99, 10, 0)),
            Some(ConfidenceLevel::Low),
            "10 reads can't certify high or mid"
        );
        assert_eq!(
            supported(&ev(0.80, 200, 0)),
            Some(ConfidenceLevel::Low),
            "the stored bound also caps it"
        );
    }

    #[test]
    fn old_future_or_malformed_evidence_supports_nothing() {
        let mut old = ev(0.95, 59, 0);
        old.read_at_ms = NOW - 31 * DAY_MS;
        assert_eq!(supported(&old), None);
        let mut future = ev(0.95, 59, 0);
        future.read_at_ms = NOW + 2 * CLOCK_SKEW_MS;
        assert_eq!(supported(&future), None);
        for bad in [
            ev(f64::NAN, 59, 0),
            ev(1.5, 59, 0),
            ev(0.95, 0, 0),
            ev(0.95, 5, 6),
            ev(0.95, MAX_READS + 1, 0),
        ] {
            assert_eq!(supported(&bad), None, "{bad:?}");
        }
        let mut edge = ev(0.95, 59, 0);
        edge.read_at_ms = NOW + CLOCK_SKEW_MS;
        assert_eq!(
            supported(&edge),
            Some(ConfidenceLevel::High),
            "exactly the skew"
        );
        edge.read_at_ms += 1;
        assert_eq!(supported(&edge), None, "a millisecond past the skew");
        let mut never = ev(0.95, 59, 0);
        never.read_at_ms = 0;
        assert_eq!(supported(&never), None);
        assert_eq!(supported_level(&ev(0.95, 59, 0), NOW, 0), None, "max age 0");
    }

    #[test]
    fn unknown_proto_values_land_on_missing() {
        assert_eq!(row_of(0), (LevelRow::Missing, false));
        assert_eq!(row_of(1), (LevelRow::Low, false));
        assert_eq!(row_of(2), (LevelRow::Mid, false));
        assert_eq!(row_of(3), (LevelRow::High, false));
        assert_eq!(row_of(4), (LevelRow::Missing, true));
        assert_eq!(row_of(-1), (LevelRow::Missing, true));
    }

    #[test]
    fn logged_reasons_name_the_cause() {
        assert_eq!(logged_because(None, false), "no_selection");
        assert_eq!(logged_because(None, true), "unknown_level");
        assert_eq!(
            logged_because(Some("kill_switch"), true),
            "page_control_off"
        );
        assert_eq!(logged_because(Some("stale_snapshot"), false), "stale_copy");
        assert_eq!(
            logged_because(Some("evidence_missing"), false),
            "weak_evidence"
        );
        assert_eq!(
            logged_because(Some("evidence_below_floor"), false),
            "weak_evidence"
        );
        assert_eq!(logged_because(Some("band_ambiguous"), false), "unreadable");
    }

    #[test]
    fn anything_malformed_or_the_retired_label_map_makes_the_config_unreadable() {
        let shipped = ConfidenceConfig::from_value(None);
        assert_eq!(shipped.unreadable, None);
        assert_eq!(ConfidenceConfig::default().evidence_max_age_ms, 30 * DAY_MS);
        let fine = ConfidenceConfig::from_value(Some(&json!({"evidence_max_age_days": 7})));
        assert_eq!(fine.unreadable, None);
        assert_eq!(fine.evidence_max_age_ms, 7 * DAY_MS);
        for bad in [
            json!({"label_map": {"*_suspend": "high"}}),
            json!({"label_map": {}}),
            json!({"evidence_max_age_days": "30"}),
            json!({"evidence_max_age_days": -1}),
            json!({"evidence_max_age_days": 0}),
            json!({"evidence_max_age_days": 3650}),
            json!({"floors": {"high": 0.5}}),
            json!("on"),
            json!([]),
        ] {
            let cfg = ConfidenceConfig::from_value(Some(&bad));
            assert!(cfg.unreadable.is_some(), "{bad}");
            assert_eq!(cfg.evidence_max_age_ms, 0, "{bad}");
        }
        assert!(
            ConfidenceConfig::from_value(Some(&json!({"label_map": {}})))
                .unreadable
                .unwrap()
                .contains("unknown key label_map")
        );
    }

    #[test]
    fn overrides_need_a_person_a_reason_and_an_expiry_within_seven_days() {
        let ov = |by: &str, reason: &str, expires: i64| EvidenceOverride {
            by: by.into(),
            reason: reason.into(),
            expires_at_ms: expires,
        };
        let written = NOW - DAY_MS;
        assert!(override_active(
            &ov("operator@example.com", "seeded", NOW + DAY_MS),
            written,
            NOW
        ));
        assert!(
            !override_active(&ov("operator@example.com", "seeded", NOW), written, NOW),
            "expired"
        );
        assert!(!override_active(
            &ov(" ", "seeded", NOW + DAY_MS),
            written,
            NOW
        ));
        assert!(!override_active(
            &ov("operator@example.com", "", NOW + DAY_MS),
            written,
            NOW
        ));
        assert!(
            !override_active(
                &ov("operator@example.com", "seeded", written + 8 * DAY_MS),
                written,
                NOW
            ),
            "a stored expiry past 7 days from the write is refused"
        );
        let active = |expires: i64, written: i64| {
            override_active(&ov("operator@example.com", "seeded", expires), written, NOW)
        };
        assert!(
            !active(NOW + 30 * DAY_MS, NOW + 30 * DAY_MS),
            "written past the skew"
        );
        assert!(
            !active(NOW + 7 * DAY_MS, NOW + 30 * DAY_MS),
            "written past the skew"
        );
        assert!(active(written + 7 * DAY_MS, written));
        assert!(!active(written + 7 * DAY_MS + 1, written));
        let ahead = NOW + CLOCK_SKEW_MS - 1;
        assert!(active(NOW + 7 * DAY_MS, ahead));
        assert!(!active(NOW + 7 * DAY_MS + 1, ahead));
        assert!(active(NOW + CLOCK_SKEW_MS + 60_000, NOW + CLOCK_SKEW_MS));
        assert!(!active(
            NOW + CLOCK_SKEW_MS + 60_000,
            NOW + CLOCK_SKEW_MS + 1
        ));
        assert!(
            !active(NOW + 60_000, NOW + CLOCK_SKEW_MS - 1),
            "expires before its write"
        );
        assert!(!active(ahead, ahead), "expires at its write");
        assert!(active(ahead + 1, ahead));
        assert!(override_problem(&ov("a", "b", NOW + 7 * DAY_MS), NOW).is_none());
        assert!(override_problem(&ov("a", "b", NOW + 7 * DAY_MS + 1), NOW).is_some());
        assert!(override_problem(&ov("a", "b", NOW), NOW).is_some());
        assert!(override_problem(&ov("", "b", NOW + DAY_MS), NOW).is_some());
    }
}
