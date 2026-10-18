#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Decision {
    Skip(String),
    Act(Vec<ActionSpec>),
    ActRequestedActions,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActionSpec {
    SuspendUser {
        perm: bool,
        policy: String,
    },
    AddLabelsV2 {
        labels: Vec<String>,
        ttl_msec: Option<i64>,
    },
    AddPostLabelsV2 {
        labels: Vec<String>,
        ttl_msec: Option<i64>,
    },
    Arkose,
    Captcha,
    SpamLivenessCheck,
}

impl ActionSpec {
    pub fn dedup_action_class(&self) -> u8 {
        match self {
            ActionSpec::SuspendUser { .. } => crate::dedup_cache::CLASS_SUSPEND,
            ActionSpec::Arkose | ActionSpec::Captcha | ActionSpec::SpamLivenessCheck => {
                crate::dedup_cache::CLASS_CHALLENGE
            }
            ActionSpec::AddLabelsV2 { .. } | ActionSpec::AddPostLabelsV2 { .. } => {
                crate::dedup_cache::CLASS_LABEL
            }
        }
    }
}
