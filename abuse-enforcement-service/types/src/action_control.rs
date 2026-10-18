use serde::{Deserialize, Serialize};

pub const ACTION_CONTROL_RECORD_VERSION: u32 = 3;
pub const BAND_RECORD_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub enum ConfidenceLevel {
    Low,
    Mid,
    High,
}

impl ConfidenceLevel {
    pub const ALL: [ConfidenceLevel; 3] = [
        ConfidenceLevel::Low,
        ConfidenceLevel::Mid,
        ConfidenceLevel::High,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            ConfidenceLevel::Low => "low",
            ConfidenceLevel::Mid => "mid",
            ConfidenceLevel::High => "high",
        }
    }

            pub fn floor(self) -> f64 {
        match self {
            ConfidenceLevel::Low => 0.70,
            ConfidenceLevel::Mid => 0.85,
            ConfidenceLevel::High => 0.95,
        }
    }

        pub fn supported_by(lower_bound: f64) -> Option<ConfidenceLevel> {
        Self::ALL
            .into_iter()
            .rev()
            .find(|l| lower_bound >= l.floor())
    }

    pub fn parse(s: &str) -> Option<ConfidenceLevel> {
        Self::ALL.into_iter().find(|l| l.as_str() == s)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub enum LevelRow {
    Low,
    Mid,
    High,
    Missing,
}

impl LevelRow {
    pub const ALL: [LevelRow; 4] = [
        LevelRow::Low,
        LevelRow::Mid,
        LevelRow::High,
        LevelRow::Missing,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            LevelRow::Low => "low",
            LevelRow::Mid => "mid",
            LevelRow::High => "high",
            LevelRow::Missing => "missing",
        }
    }

        pub fn level(self) -> Option<ConfidenceLevel> {
        match self {
            LevelRow::Low => Some(ConfidenceLevel::Low),
            LevelRow::Mid => Some(ConfidenceLevel::Mid),
            LevelRow::High => Some(ConfidenceLevel::High),
            LevelRow::Missing => None,
        }
    }
}

impl From<ConfidenceLevel> for LevelRow {
    fn from(level: ConfidenceLevel) -> Self {
        match level {
            ConfidenceLevel::Low => LevelRow::Low,
            ConfidenceLevel::Mid => LevelRow::Mid,
            ConfidenceLevel::High => LevelRow::High,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(deny_unknown_fields)]
pub struct ConfidenceEvidence {
            pub precision_lower_bound: f64,
    pub reads: u32,
    pub humans: u32,
        pub read_at_ms: i64,
        #[serde(default)]
    pub link: String,
}

impl Eq for ConfidenceEvidence {}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(deny_unknown_fields)]
pub struct EvidenceOverride {
    pub by: String,
    pub reason: String,
    pub expires_at_ms: i64,
}

pub const EVIDENCE_OVERRIDE_MAX_MS: i64 = 7 * 86_400_000;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub enum ControlMode {
        Rules,
            Recording,
        Live,
            Rows,
}

impl ControlMode {
    pub fn as_str(self) -> &'static str {
        match self {
            ControlMode::Rules => "rules",
            ControlMode::Recording => "recording",
            ControlMode::Live => "live",
            ControlMode::Rows => "rows",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub enum Lifetime {
    #[serde(rename = "7d")]
    Days7,
    #[serde(rename = "30d")]
    Days30,
    #[serde(rename = "permanent")]
    Permanent,
}

impl Lifetime {
        pub fn ttl_msec(self) -> Option<i64> {
        match self {
            Lifetime::Days7 => Some(7 * 86_400_000),
            Lifetime::Days30 => Some(30 * 86_400_000),
            Lifetime::Permanent => None,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum PageAction {
        Suspend {},
        LivenessCheck {},
        SpamHighRecall { lifetime: Lifetime },
        PostSpamHighRecall { lifetime: Lifetime },
        SuspendAuthor {},
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub enum ControlEntity {
    User,
    Post,
}

impl ControlEntity {
    pub fn as_str(self) -> &'static str {
        match self {
            ControlEntity::User => "user",
            ControlEntity::Post => "post",
        }
    }
}

impl PageAction {
    pub fn entity(self) -> ControlEntity {
        match self {
            PageAction::Suspend {}
            | PageAction::LivenessCheck {}
            | PageAction::SpamHighRecall { .. } => ControlEntity::User,
            PageAction::PostSpamHighRecall { .. } | PageAction::SuspendAuthor {} => {
                ControlEntity::Post
            }
        }
    }

            pub fn menu(entity: ControlEntity) -> Vec<PageAction> {
        match entity {
            ControlEntity::User => vec![
                PageAction::Suspend {},
                PageAction::LivenessCheck {},
                PageAction::SpamHighRecall {
                    lifetime: Lifetime::Days7,
                },
                PageAction::SpamHighRecall {
                    lifetime: Lifetime::Days30,
                },
                PageAction::SpamHighRecall {
                    lifetime: Lifetime::Permanent,
                },
            ],
            ControlEntity::Post => vec![
                PageAction::PostSpamHighRecall {
                    lifetime: Lifetime::Days30,
                },
                PageAction::PostSpamHighRecall {
                    lifetime: Lifetime::Permanent,
                },
                PageAction::SuspendAuthor {},
            ],
        }
    }

            pub fn suggested(entity: ControlEntity, row: LevelRow) -> Vec<PageAction> {
        match (entity, row) {
            (ControlEntity::User, LevelRow::Low) => vec![PageAction::LivenessCheck {}],
            (ControlEntity::User, LevelRow::Mid) => vec![
                PageAction::LivenessCheck {},
                PageAction::SpamHighRecall {
                    lifetime: Lifetime::Days7,
                },
            ],
            (ControlEntity::User, LevelRow::High) => vec![PageAction::Suspend {}],
            (ControlEntity::Post, LevelRow::Mid) => vec![PageAction::PostSpamHighRecall {
                lifetime: Lifetime::Days30,
            }],
            (ControlEntity::Post, LevelRow::High) => vec![PageAction::PostSpamHighRecall {
                lifetime: Lifetime::Permanent,
            }],
            (ControlEntity::Post, LevelRow::Low) | (_, LevelRow::Missing) => vec![],
        }
    }

            pub fn kind(self) -> &'static str {
        match self {
            PageAction::Suspend {} => "suspend",
            PageAction::LivenessCheck {} => "liveness_check",
            PageAction::SpamHighRecall { .. } => "spam_high_recall",
            PageAction::PostSpamHighRecall { .. } => "post_spam_high_recall",
            PageAction::SuspendAuthor {} => "suspend_author",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ActionControlRecord {
    pub v: u32,
    pub entity_type: ControlEntity,
    pub head: String,
    #[serde(default)]
    pub classifier: Option<String>,
    #[serde(default)]
    pub band: Option<String>,
    pub mode: ControlMode,
    #[serde(default)]
    pub actions: Vec<PageAction>,
    #[serde(default)]
    pub reason: String,
    #[serde(default)]
    pub updated_by: String,
    #[serde(default)]
    pub key_id: String,
    #[serde(default)]
    pub updated_at_ms: i64,
        pub version: u64,
        #[serde(default)]
    pub write_nonce: u64,
        #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confidence: Option<LevelRow>,
            #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence: Option<Box<ConfidenceEvidence>>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence_override: Option<Box<EvidenceOverride>>,
}

impl ActionControlRecord {
            pub const FIELDS: &'static [&'static str] = &[
        "v",
        "entity_type",
        "head",
        "classifier",
        "band",
        "mode",
        "actions",
        "reason",
        "updated_by",
        "key_id",
        "updated_at_ms",
        "version",
        "write_nonce",
        "confidence",
        "evidence",
        "evidence_override",
    ];

            pub fn uses_confidence(&self) -> bool {
        self.confidence.is_some()
            || self.evidence.is_some()
            || self.evidence_override.is_some()
            || self.mode == ControlMode::Rows
    }

                pub fn schema_version(&self) -> u32 {
        if self.uses_confidence() {
            ACTION_CONTROL_RECORD_VERSION
        } else {
            BAND_RECORD_VERSION
        }
    }

            pub fn parse_strict(bytes: &[u8]) -> Result<Self, String> {
        let value: serde_json::Value = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
        if let Some(obj) = value.as_object()
            && let Some(k) = obj.keys().find(|k| !Self::FIELDS.contains(&k.as_str()))
        {
            return Err(format!("unknown field `{k}`"));
        }
        serde_json::from_value(value).map_err(|e| e.to_string())
    }

            pub fn scope(&self) -> &'static str {
        match (
            self.classifier.is_some(),
            self.band.is_some(),
            self.confidence.is_some(),
        ) {
            (false, _, true) => "confidence",
            (true, _, true) => "classifier+confidence",
            (false, false, false) => "head",
            (true, false, false) => "classifier",
            (false, true, false) => "band",
            (true, true, false) => "classifier+band",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ActionControlView {
    #[serde(flatten)]
    pub record: ActionControlRecord,
    pub scope: String,
    pub lkey: String,
        pub effective_mode: ControlMode,
                #[serde(default)]
    pub forced_recording_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(deny_unknown_fields)]
pub struct ActionControlPutRequest {
            pub entity_type: ControlEntity,
    pub head: String,
    #[serde(default)]
    pub classifier: Option<String>,
    #[serde(default)]
    pub band: Option<String>,
    pub mode: ControlMode,
    #[serde(default)]
    pub actions: Vec<PageAction>,
    #[serde(default)]
    pub reason: String,
        #[serde(default)]
    pub expected_version: Option<u64>,
        #[serde(default)]
    pub confirm: Option<String>,
        #[serde(default)]
    pub updated_by: Option<String>,
        #[serde(default)]
    pub confidence: Option<LevelRow>,
        #[serde(default)]
    pub evidence: Option<ConfidenceEvidence>,
        #[serde(default)]
    pub evidence_override: Option<EvidenceOverride>,
            #[serde(default)]
    pub rows: Option<RowsRequest>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(deny_unknown_fields)]
pub struct RowsRequest {
    #[serde(default)]
    pub low: Option<RowSetting>,
    #[serde(default)]
    pub mid: Option<RowSetting>,
    #[serde(default)]
    pub high: Option<RowSetting>,
    #[serde(default)]
    pub missing: Option<RowSetting>,
}

impl RowsRequest {
    pub fn get(&self, row: LevelRow) -> Option<&RowSetting> {
        match row {
            LevelRow::Low => self.low.as_ref(),
            LevelRow::Mid => self.mid.as_ref(),
            LevelRow::High => self.high.as_ref(),
            LevelRow::Missing => self.missing.as_ref(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(deny_unknown_fields)]
pub struct RowSetting {
        pub mode: ControlMode,
    #[serde(default)]
    pub actions: Vec<PageAction>,
    #[serde(default)]
    pub evidence: Option<ConfidenceEvidence>,
    #[serde(default)]
    pub evidence_override: Option<EvidenceOverride>,
        #[serde(default)]
    pub expected_version: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ActionControlSeenRow {
    pub entity_type: ControlEntity,
    pub head: String,
        pub classifier: String,
    pub row: LevelRow,
    pub last_seen_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ActionControlSeen {
    pub head: String,
    pub classifier: String,
        pub band: String,
    pub entity_type: ControlEntity,
        #[serde(default)]
    pub factory: bool,
    pub last_seen_ms: i64,
        #[serde(default)]
    pub rule_id: String,
            #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(value_type = Object))]
    pub action_specs: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SafetyCheckView {
    pub id: String,
    pub reason: String,
    pub when: String,
        pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SafetyChecksStatus {
    pub ok: bool,
    pub problems: Vec<String>,
    pub checks: Vec<SafetyCheckView>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SafetyChecksByEntity {
    pub user: SafetyChecksStatus,
    pub post: SafetyChecksStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AllowedActions {
    pub user: Vec<PageAction>,
    pub post: Vec<PageAction>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CseLists {
    pub heads: Vec<String>,
    pub labels: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct UnreadableRecord {
    pub lkey: String,
    pub error: String,
            #[serde(default)]
    pub version: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ActionControlListResponse {
    pub feature_version: u32,
    pub env: String,
    pub pod: String,
                    pub enabled: bool,
        pub ignore_store: bool,
        pub source: String,
    #[serde(default)]
    pub error: Option<String>,
        pub snapshot_age_ms: i64,
    pub stale: bool,
    pub stale_after_ms: i64,
    pub records: Vec<ActionControlView>,
    pub unreadable: Vec<UnreadableRecord>,
    pub seen: Vec<ActionControlSeen>,
    #[serde(default)]
    pub seen_error: Option<String>,
        #[serde(default)]
    pub seen_rows: Vec<ActionControlSeenRow>,
    pub safety_checks: SafetyChecksByEntity,
    pub allowed_actions: AllowedActions,
    #[serde(default)]
    pub cse: CseLists,
    #[serde(default)]
    pub confidence: Option<ConfidenceView>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct RowView {
    pub row: LevelRow,
            pub floor: Option<f64>,
        pub suggested_actions: AllowedActions,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ConfidenceView {
    pub rows: Vec<RowView>,
        pub evidence_max_age_ms: i64,
            #[serde(default)]
    pub config_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ActionControlHistoryEntry {
    pub at_ms: i64,
    pub version: u64,
    pub lkey: String,
    pub scope: String,
    pub updated_by: String,
    pub key_id: String,
    pub reason: String,
    pub env: String,
    pub before: Option<ActionControlRecord>,
    pub after: ActionControlRecord,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ActionControlHistoryResponse {
    pub head: String,
    pub entries: Vec<ActionControlHistoryEntry>,
    #[serde(default)]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct PodView {
    pub pod: String,
    pub env: String,
    pub image: String,
    pub action_control: PodActionControlView,
    pub off_switch: PodOffSwitchView,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct PodActionControlView {
    pub loaded: bool,
    pub snapshot_age_ms: i64,
    pub stale: bool,
    pub records: usize,
        pub page_managed: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct PodOffSwitchView {
    pub snapshot_age_ms: i64,
        pub active: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_actions_parse_from_the_documented_json() {
        let a: Vec<PageAction> = serde_json::from_str(
            r#"[{"kind":"suspend"},{"kind":"liveness_check"},
                {"kind":"spam_high_recall","lifetime":"7d"},
                {"kind":"post_spam_high_recall","lifetime":"permanent"},
                {"kind":"suspend_author"}]"#,
        )
        .unwrap();
        assert_eq!(
            a,
            vec![
                PageAction::Suspend {},
                PageAction::LivenessCheck {},
                PageAction::SpamHighRecall {
                    lifetime: Lifetime::Days7
                },
                PageAction::PostSpamHighRecall {
                    lifetime: Lifetime::Permanent
                },
                PageAction::SuspendAuthor {},
            ]
        );
    }

    #[test]
    fn actions_outside_the_menu_do_not_parse() {
        for bad in [
            r#"{"kind":"suspend","perm":true}"#,
            r#"{"kind":"suspend","policy":"Cse"}"#,
            r#"{"kind":"spam_high_recall","lifetime":"90d"}"#,
            r#"{"kind":"label","labels":["SpamHighRecall"]}"#,
            r#"{"kind":"arkose"}"#,
        ] {
            assert!(serde_json::from_str::<PageAction>(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn record_with_an_unknown_field_is_unreadable() {
        let ok = r#"{"v":1,"entity_type":"user","head":"H","mode":"recording","version":1}"#;
        let r: ActionControlRecord = serde_json::from_str(ok).unwrap();
        assert_eq!(r.scope(), "head");
        assert!(r.actions.is_empty());
        let newer = r#"{"v":2,"entity_type":"user","head":"H","mode":"recording","version":1,"new_field":true}"#;
        assert!(ActionControlRecord::parse_strict(newer.as_bytes()).is_err());
        let retired = r#"{"v":1,"entity_type":"user","head":"H","mode":"recording","version":1,"dedup_strength":"suspend"}"#;
        assert!(ActionControlRecord::parse_strict(retired.as_bytes()).is_err());
    }

    #[test]
    fn fields_list_matches_the_record_and_the_view_round_trips() {
        let r = ActionControlRecord::parse_strict(
            br#"{"v":2,"entity_type":"post","head":"H","classifier":"c","band":"b","mode":"live",
                "actions":[{"kind":"suspend_author"}],"reason":"r","updated_by":"u","key_id":"k",
                "updated_at_ms":5,"version":2,"write_nonce":9,"confidence":"high",
                "evidence":{"precision_lower_bound":0.95,"reads":59,"humans":0,"read_at_ms":4,"link":"l"},
                "evidence_override":{"by":"operator@example.com","reason":"seeded","expires_at_ms":6}}"#,
        )
        .unwrap();
        let keys: Vec<String> = serde_json::to_value(&r)
            .unwrap()
            .as_object()
            .unwrap()
            .keys()
            .cloned()
            .collect();
        let mut want: Vec<&str> = ActionControlRecord::FIELDS.to_vec();
        want.sort_unstable();
        let mut got: Vec<&str> = keys.iter().map(String::as_str).collect();
        got.sort_unstable();
        assert_eq!(got, want);
        let view = ActionControlView {
            record: r,
            scope: "classifier+band".into(),
            lkey: "post/cb/H/c/b".into(),
            effective_mode: ControlMode::Recording,
            forced_recording_reason: Some("kill_switch".into()),
        };
        let back: ActionControlView =
            serde_json::from_str(&serde_json::to_string(&view).unwrap()).unwrap();
        assert_eq!(back, view);
    }

    #[test]
    fn record_and_put_need_the_entity() {
        let no_entity = r#"{"v":1,"head":"H","mode":"recording","version":1}"#;
        assert!(serde_json::from_str::<ActionControlRecord>(no_entity).is_err());
        let put = r#"{"head":"H","mode":"recording","reason":"r"}"#;
        assert!(serde_json::from_str::<ActionControlPutRequest>(put).is_err());
        let put = r#"{"entity_type":"post","head":"H","mode":"recording","reason":"r"}"#;
        let p: ActionControlPutRequest = serde_json::from_str(put).unwrap();
        assert_eq!(p.entity_type, ControlEntity::Post);
        let retired = r#"{"entity_type":"post","head":"H","mode":"recording","reason":"r","dedup_strength":"auto"}"#;
        assert!(serde_json::from_str::<ActionControlPutRequest>(retired).is_err());
    }

    #[test]
    fn scopes() {
        let mut r: ActionControlRecord = serde_json::from_str(
            r#"{"v":1,"entity_type":"user","head":"H","mode":"rules","version":1}"#,
        )
        .unwrap();
        r.classifier = Some("c".into());
        assert_eq!(r.scope(), "classifier");
        r.band = Some("b".into());
        assert_eq!(r.scope(), "classifier+band");
        r.classifier = None;
        assert_eq!(r.scope(), "band");
        r.band = None;
        r.confidence = Some(LevelRow::Mid);
        assert_eq!(r.scope(), "confidence");
        r.classifier = Some("c".into());
        assert_eq!(r.scope(), "classifier+confidence");
    }

    #[test]
    fn a_band_record_keeps_the_old_schema_version_and_bytes() {
        let json = r#"{"v":1,"entity_type":"user","head":"H","mode":"live","actions":[{"kind":"suspend"}],"reason":"","updated_by":"","key_id":"","updated_at_ms":0,"version":1,"write_nonce":0}"#;
        let r: ActionControlRecord = serde_json::from_str(json).unwrap();
        assert_eq!(r.schema_version(), BAND_RECORD_VERSION);
        let back = serde_json::to_string(&r).unwrap();
        assert!(!back.contains("confidence"), "{back}");
        assert!(!back.contains("evidence"), "{back}");
        let mut l = r.clone();
        l.confidence = Some(LevelRow::High);
        assert_eq!(
            l.schema_version(),
            3,
            "a row is refused by images that read v: 2 level records"
        );
        l.confidence = Some(LevelRow::Missing);
        assert_eq!(l.schema_version(), ACTION_CONTROL_RECORD_VERSION);
        let mut d = r;
        d.mode = ControlMode::Rows;
        assert_eq!(d.schema_version(), ACTION_CONTROL_RECORD_VERSION);
    }

    #[test]
    fn a_retired_mode_or_an_unknown_row_does_not_parse() {
        for bad in [
            r#"{"v":2,"entity_type":"user","head":"H","mode":"confidence_defaults","version":1}"#,
            r#"{"v":3,"entity_type":"user","head":"H","mode":"live","confidence":"very_high","version":1}"#,
        ] {
            assert!(
                ActionControlRecord::parse_strict(bad.as_bytes()).is_err(),
                "{bad}"
            );
        }
        let missing = r#"{"v":3,"entity_type":"user","head":"H","mode":"recording","confidence":"missing","version":1}"#;
        let r = ActionControlRecord::parse_strict(missing.as_bytes()).unwrap();
        assert_eq!(r.confidence, Some(LevelRow::Missing));
        assert_eq!(LevelRow::Missing.level(), None);
        assert_eq!(LevelRow::from(ConfidenceLevel::Mid), LevelRow::Mid);
    }

    #[test]
    fn floors_and_the_level_evidence_supports() {
        assert_eq!(
            ConfidenceLevel::supported_by(0.951),
            Some(ConfidenceLevel::High)
        );
        assert_eq!(
            ConfidenceLevel::supported_by(0.95),
            Some(ConfidenceLevel::High)
        );
        assert_eq!(
            ConfidenceLevel::supported_by(0.9499),
            Some(ConfidenceLevel::Mid)
        );
        assert_eq!(
            ConfidenceLevel::supported_by(0.85),
            Some(ConfidenceLevel::Mid)
        );
        assert_eq!(
            ConfidenceLevel::supported_by(0.70),
            Some(ConfidenceLevel::Low)
        );
        assert_eq!(ConfidenceLevel::supported_by(0.6999), None);
        assert_eq!(ConfidenceLevel::supported_by(f64::NAN), None);
    }

    #[test]
    fn suggested_table() {
        use LevelRow::*;
        let user = |l| PageAction::suggested(ControlEntity::User, l);
        let post = |l| PageAction::suggested(ControlEntity::Post, l);
        assert!(user(Missing).is_empty() && post(Missing).is_empty());
        assert_eq!(user(Low), vec![PageAction::LivenessCheck {}]);
        assert_eq!(
            user(Mid),
            vec![
                PageAction::LivenessCheck {},
                PageAction::SpamHighRecall {
                    lifetime: Lifetime::Days7
                }
            ]
        );
        assert_eq!(user(High), vec![PageAction::Suspend {}]);
        assert!(post(Low).is_empty());
        for l in LevelRow::ALL {
            for (e, a) in [
                (ControlEntity::User, user(l)),
                (ControlEntity::Post, post(l)),
            ] {
                assert!(a.iter().all(|x| x.entity() == e));
                assert!(a.iter().all(|x| PageAction::menu(e).contains(x)), "{l:?}");
            }
        }
        assert!(
            !post(High).contains(&PageAction::SuspendAuthor {}),
            "a post default never suspends the author"
        );
    }
}
