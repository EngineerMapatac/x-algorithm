use crate::hydration::Hydrators;
use crate::models::{
    AuthorLabel, DropReason, FosnrReason, LimitedEngagementReason, MediaInterstitial,
    MediaRestriction, Notice, NsfwViewerDropReason, SafetyLabelMap, SafetyLabelType,
    TombstoneReason, ViewerProfile, Withholding,
};
use crate::params::{CountryList, LimitedActionType};
use crate::rules::RuleContext;
use features::{In, TweetSafetyLabels};
use inputs::Interpreter;
use rule::{NamedCond, RuleCond};
use std::fmt::{self, Debug, Formatter};
use std::ops::Not;
use xai_safety_label_store::types::label_meta;
use xai_visibility_filtering::models::{FilteredReason, SafetyResult, SafetyResultReason};
use xai_x_thrift::action::{AppealablePolicy, InterstitialAction, InterstitialReason};

#[derive(Clone, Copy, Debug, PartialEq, Eq, strum::IntoStaticStr)]
#[strum(serialize_all = "snake_case")]
pub(super) enum RuleId {
    FilterAll,

    SuspendedAuthor,
    DeactivatedAuthor,
    ErasedAuthor,
    OffboardedAuthor,
    ProtectedAuthor,
    ViewerBlocksAuthor,
    ViewerMutesAuthor,
    ViewerMutesRetweets,

    Pdna,
    Bounce,
    Spam,
    ForEmergencyUseOnly,
    FosnrHatefulConduct,
    FosnrViolentSpeech,
    FosnrAbuse,
    FosnrCivicIntegrity,
    FosnrAbuseInsults,
    FosnrAbuseInsultsNonFollower,
    FosnrAbuseInsultsFollower,
    FosnrAuthor,
    FosnrFallback,
    NullcastedTweet,
    StaleTweet,
    LegalTakedown,
    LocalLawsTakedown,
    ArticleTweetContent,
    ProtectedCommunityTweet,
    HiddenCommunityTweet,
    AuthorRemovedCommunityTweet,

    SensitiveViewerLoggedOut,
    SensitiveViewerUnderage,
    SensitiveViewerNoStatedAge,
    NsfwSensitiveViewerTweet,
    NsfwSensitiveViewerUser,

    ExclusiveTweet,
    AuthorBlocksViewerExclusiveContent,
    CreatorTweetNsfw,
    TrustedFriendsTweet,

    NsfwHighPrecision,
    GoreAndViolenceHighPrecision,
    GoreAndViolenceIgnoringSettings,
    NsfwCardImage,
    NsfwAdmin,
    NsfwUser,
    NsfwAccount,
    NsfwReportedHeuristics,
    GoreAndViolenceReportedHeuristics,

    DmcaMedia,
    GeoRestrictedMedia,
    NsfwUserAuthor,
    NsfwAdminAuthor,
    NsfwUserTweetFlag,
    NsfwAdminTweetFlag,
    NsfwHighRecall,
    DoNotAmplify,
    MaliciousUrl,
    SpamHighRecall,
    BrazilElectionLegal,

    NsfwHighRecallUserLabel,
    NsfwHighPrecisionUserLabel,
    NsfwAvatarImageUserLabel,
    NsfwBannerImageUserLabel,
    NsfwNearPerfectUserLabel,
    SpamHighRecallUserLabel,
    CompromisedUserLabel,
    ReadOnlyUserLabel,
    ImpersonationHighPrecisionUserLabel,
    AbusiveHighRecallUserLabel,
    DoNotAmplifyUserLabel,

    BlockedViewer,
    LimitRepliesByInvitation,
    LimitRepliesCommunity,
    LimitRepliesSubscribers,
    LimitRepliesVerified,
    LimitRepliesMyNetwork,
    LimitRepliesCo,
    CommunityTweetViewerRemoved,
    LocalTweet,
    ReadOnlyViewer,
}

pub(super) mod conditions;
pub(super) mod features;
pub(super) mod inputs;
#[cfg(test)]
mod laws;
pub(super) mod observed;
mod predicates;
pub(super) mod rule;

pub(in crate::rules) use features::leaves;
pub use features::{
    ArticleStageLeaf, AuthorLabelsLeaf, AuthorSafetyLeaf, CommunityModeratorLeaf,
    CommunityViewerRemovedLeaf, CompoundLeaf, EdgeLeaf, PostFactsLeaf, PostModerationLeaf,
    ReplyControlLeaf, RequestLeaf, SourceTweetIdLeaf, TweetLeaf, TweetSafetyLabelsLeaf,
    ViewerAccountLeaf,
};
pub use predicates::Predicate;
pub(in crate::rules) use predicates::*;

pub(super) fn rule_name(id: RuleId, action: &RuleAction) -> String {
    let id: &str = id.into();
    let (kind, reason, prompt) = match action.spec() {
        ActionSpec::Drop(DropReason::Legacy(reason)) => {
            let reason = match reason {
                FilteredReason::UnspecifiedReason => "unspecified".to_owned(),
                FilteredReason::PossiblyUndesirable => "undesirable".to_owned(),
                FilteredReason::ContainNsfwMedia => "nsfw_media".to_owned(),
                FilteredReason::AuthorAccountIsInactive => "inactive".to_owned(),
                reason @ (FilteredReason::AuthorBlockViewer
                | FilteredReason::AuthorIsProtected
                | FilteredReason::AuthorIsUnsafe
                | FilteredReason::ReportedTweet
                | FilteredReason::TweetMatchesViewerMutedKeyword(_)
                | FilteredReason::TweetIsBounced
                | FilteredReason::SafetyResult(_)
                | FilteredReason::AuthorIsDeactivated
                | FilteredReason::AuthorIsSuspended
                | FilteredReason::ViewerMutesAuthor
                | FilteredReason::TweetIsNullcast
                | FilteredReason::ExclusiveTweet
                | FilteredReason::ViewerBlocksAuthor) => snake(reason),
            };
            ("drop", Some(reason), None)
        }
        ActionSpec::Drop(DropReason::NsfwViewer(reason)) => {
            ("drop", Some(<&str>::from(reason).to_owned()), None)
        }
        ActionSpec::Tombstone(TombstoneReason::SensitiveViewerAgeVerification) => {
            ("tombstone", Some("age_verification".to_owned()), None)
        }
        ActionSpec::Tombstone(reason) => ("tombstone", Some(<&str>::from(reason).to_owned()), None),
        ActionSpec::MediaRestriction(MediaRestriction::MediaInterstitial(blur)) => {
            #[expect(
                clippy::wildcard_enum_match_arm,
                reason = "InterstitialReason is a generated Thrift union; every variant but PossiblyUndesirable takes its snake-cased name, including any the IDL adds"
            )]
            let reason = match &blur.reason {
                InterstitialReason::PossiblyUndesirable(_) => "undesirable".to_owned(),
                reason => snake(reason),
            };
            let prompt = blur.prompt.map(|prompt| match prompt {
                InterstitialAction::AGE_VERIFICATION_PROMPT => "age_prompt".to_owned(),
                prompt => prompt.0.to_string(),
            });
            ("blur", Some(reason), prompt)
        }
        ActionSpec::MediaRestriction(MediaRestriction::NsfwInterstitial) => {
            ("legacy_interstitial", None, None)
        }
        ActionSpec::LimitedEngagement(reason) => (
            "limited_engagement",
            Some(<&str>::from(reason).to_owned()),
            None,
        ),
        ActionSpec::SoftIntervention { policy, .. } => {
            let policy = match *policy {
                AppealablePolicy::ABUSE => "abuse".to_owned(),
                policy => policy.0.to_string(),
            };
            ("soft_intervention", Some(policy), None)
        }
        ActionSpec::Appealable(_) => ("appealable", None, None),
    };
    let keeps_reason = matches!(action.spec(), ActionSpec::SoftIntervention { .. });
    let squashed = |name: &str| name.replace('_', "");
    let reason = reason
        .map(|reason| {
            ["author_is_", "tweet_is_", "is_", "has_"]
                .iter()
                .find_map(|prefix| reason.strip_prefix(prefix))
                .unwrap_or(&reason)
                .to_owned()
        })
        .filter(|reason| keeps_reason || !squashed(id).contains(&squashed(reason)));
    let mut name = format!("{id}/{kind}");
    for part in [reason, prompt].into_iter().flatten() {
        name.push('/');
        name.push_str(&part);
    }
    name
}

fn snake(value: &impl std::fmt::Debug) -> String {
    snake_case(format!("{value:?}").split('(').next().unwrap_or_default())
}

fn snake_case(variant: &str) -> String {
    let mut snake = String::new();
    for (index, char) in variant.chars().enumerate() {
        if char.is_uppercase() && index > 0 {
            snake.push('_');
        }
        snake.push(char.to_ascii_lowercase());
    }
    snake
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct FosnrViolation {
    pub(super) label: SafetyLabelType,
    pub(super) policy: AppealablePolicy,
    pub(super) level: i8,
    pub(super) limited_actions: &'static [LimitedActionType],
}

#[derive(Clone, Debug, PartialEq)]
pub(super) enum ActionSpec {
    Drop(DropReason),
    Tombstone(TombstoneReason),
    MediaRestriction(MediaRestriction),
    SoftIntervention {
        label: SafetyLabelType,
        policy: AppealablePolicy,
        level: i8,
    },
    LimitedEngagement(LimitedEngagementReason),
    Appealable(&'static [FosnrViolation]),
}

#[derive(Clone, PartialEq)]
pub(super) struct RuleAction {
    spec: ActionSpec,
}

impl RuleAction {
    pub(super) const fn spec(&self) -> &ActionSpec {
        &self.spec
    }
}

impl Debug for RuleAction {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.spec.fmt(f)
    }
}

impl Eq for RuleAction {}

#[cfg(test)]
impl From<ActionSpec> for RuleAction {
    fn from(spec: ActionSpec) -> Self {
        RuleAction { spec }
    }
}

#[derive(Clone, Copy)]
pub(super) enum DropAs {
    UnspecifiedReason,
    PossiblyUndesirable,
    ContainNsfwMedia,
    AuthorAccountIsInactive,
    AuthorIsSuspended,
    AuthorIsDeactivated,
    AuthorIsProtected,
    ViewerBlocksAuthor,
    ViewerMutesAuthor,
    TweetIsBounced,
    TweetIsNullcast,
    ExclusiveTweet,
    NsfwHighPrecisionSafetyResult,
    NsfwViewerLoggedOut,
    NsfwViewerIsUnderage,
    NsfwViewerHasNoStatedAge,
}

impl DropAs {
    const fn reason(self) -> DropReason {
        let legacy = match self {
            DropAs::UnspecifiedReason => FilteredReason::UnspecifiedReason,
            DropAs::PossiblyUndesirable => FilteredReason::PossiblyUndesirable,
            DropAs::ContainNsfwMedia => FilteredReason::ContainNsfwMedia,
            DropAs::AuthorAccountIsInactive => FilteredReason::AuthorAccountIsInactive,
            DropAs::AuthorIsSuspended => FilteredReason::AuthorIsSuspended,
            DropAs::AuthorIsDeactivated => FilteredReason::AuthorIsDeactivated,
            DropAs::AuthorIsProtected => FilteredReason::AuthorIsProtected,
            DropAs::ViewerBlocksAuthor => FilteredReason::ViewerBlocksAuthor,
            DropAs::ViewerMutesAuthor => FilteredReason::ViewerMutesAuthor,
            DropAs::TweetIsBounced => FilteredReason::TweetIsBounced,
            DropAs::TweetIsNullcast => FilteredReason::TweetIsNullcast,
            DropAs::ExclusiveTweet => FilteredReason::ExclusiveTweet,
            DropAs::NsfwHighPrecisionSafetyResult => FilteredReason::SafetyResult(SafetyResult {
                reason: Some(SafetyResultReason::NsfwHighPrecision),
                action: xai_visibility_filtering::models::Action::Drop(
                    xai_visibility_filtering::models::DropReason {},
                ),
            }),
            DropAs::NsfwViewerLoggedOut => {
                return DropReason::NsfwViewer(NsfwViewerDropReason::LoggedOut);
            }
            DropAs::NsfwViewerIsUnderage => {
                return DropReason::NsfwViewer(NsfwViewerDropReason::IsUnderage);
            }
            DropAs::NsfwViewerHasNoStatedAge => {
                return DropReason::NsfwViewer(NsfwViewerDropReason::HasNoStatedAge);
            }
        };
        DropReason::Legacy(legacy)
    }
}

pub(super) const fn drop_post(reason: DropAs) -> RuleAction {
    RuleAction {
        spec: ActionSpec::Drop(reason.reason()),
    }
}

pub(super) const fn tombstone(reason: TombstoneReason) -> RuleAction {
    RuleAction {
        spec: ActionSpec::Tombstone(reason),
    }
}

pub(super) const fn limit(reason: LimitedEngagementReason) -> RuleAction {
    RuleAction {
        spec: ActionSpec::LimitedEngagement(reason),
    }
}

pub(super) const fn soft_intervention(
    label: SafetyLabelType,
    policy: AppealablePolicy,
    level: i8,
) -> RuleAction {
    RuleAction {
        spec: ActionSpec::SoftIntervention {
            label,
            policy,
            level,
        },
    }
}

pub(super) const fn appealable(violations: &'static [FosnrViolation]) -> RuleAction {
    RuleAction {
        spec: ActionSpec::Appealable(violations),
    }
}

pub(super) const fn blur(reason: InterstitialReason) -> RuleAction {
    media_interstitial(reason, None)
}

pub(super) const fn blur_with_age_prompt(reason: InterstitialReason) -> RuleAction {
    media_interstitial(reason, Some(InterstitialAction::AGE_VERIFICATION_PROMPT))
}

const fn media_interstitial(
    reason: InterstitialReason,
    prompt: Option<InterstitialAction>,
) -> RuleAction {
    RuleAction {
        spec: ActionSpec::MediaRestriction(MediaRestriction::MediaInterstitial(
            MediaInterstitial {
                legacy: FilteredReason::ContainNsfwMedia,
                reason,
                prompt,
            },
        )),
    }
}

pub(super) const fn legacy_nsfw_interstitial() -> RuleAction {
    RuleAction {
        spec: ActionSpec::MediaRestriction(MediaRestriction::NsfwInterstitial),
    }
}

#[cfg(test)]
impl ActionSpec {
    pub(super) const fn kind_and_slot(&self) -> (&'static str, &'static str) {
        match self {
            ActionSpec::Appealable(_) => ("appealable", "notice"),
            ActionSpec::Drop(_) => ("drop", "withholding"),
            ActionSpec::Tombstone(_) => ("tombstone", "withholding"),
            ActionSpec::MediaRestriction(_) => ("media restriction", "media"),
            ActionSpec::SoftIntervention { .. } => ("soft intervention", "notice"),
            ActionSpec::LimitedEngagement(_) => ("limited engagement", "engagement"),
        }
    }

    pub(super) fn spelled(&self) -> String {
        match self {
            ActionSpec::Drop(reason) => format!("{reason:?}"),
            ActionSpec::Tombstone(reason) => format!("{reason:?}"),
            ActionSpec::MediaRestriction(restriction) => format!("{restriction:?}"),
            ActionSpec::SoftIntervention {
                label,
                policy,
                level,
            } => format!("{} {policy:?} level {level}", spelled_label(*label)),
            ActionSpec::LimitedEngagement(reason) => format!("{reason:?}"),
            ActionSpec::Appealable(violations) => violations
                .iter()
                .map(
                    |&FosnrViolation {
                         label,
                         policy,
                         level,
                         limited_actions,
                     }| {
                        format!(
                            "{} {policy:?} level {level} limits {limited_actions:?}",
                            spelled_label(label)
                        )
                    },
                )
                .collect::<Vec<_>>()
                .join(", "),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Contribution {
    Withholding(Withholding),
    Media(MediaRestriction),
    Engagement(LimitedEngagementReason),
    Notice(Notice),
}

impl ActionSpec {
    #[inline]
    pub(super) fn interpret<'a, I: Interpreter<'a>>(
        &self,
        interpreter: &I,
    ) -> I::Output<Option<Contribution>> {
        match self {
            Self::Drop(reason) => interpreter
                .run(|()| Some(Contribution::Withholding(Withholding::Drop(reason.clone())))),
            Self::Tombstone(reason) => interpreter
                .run(|()| Some(Contribution::Withholding(Withholding::Tombstone(*reason)))),
            Self::MediaRestriction(restriction) => {
                interpreter.run(|()| Some(Contribution::Media(restriction.clone())))
            }
            Self::LimitedEngagement(reason) => {
                interpreter.run(|()| Some(Contribution::Engagement(*reason)))
            }
            Self::SoftIntervention {
                label,
                policy,
                level,
            } => interpreter.run(|In::<TweetSafetyLabels>(labels)| {
                Some(Contribution::Notice(Notice::SoftIntervention(
                    fosnr_reason(*label, *policy, *level, labels),
                )))
            }),
            Self::Appealable(violations) => interpreter.run(|In::<TweetSafetyLabels>(labels)| {
                appeal(violations, labels).map(Contribution::Notice)
            }),
        }
    }

    pub(super) fn requirements(&self) -> Hydrators {
        self.interpret(&inputs::Required)
    }

    pub(super) const fn severity(&self) -> u8 {
        match self {
            Self::Appealable(_) => 18,
            Self::Drop(_) => 17,
            Self::Tombstone(_) => 16,
            Self::MediaRestriction(_) => 10,
            Self::SoftIntervention { .. } => 8,
            Self::LimitedEngagement(_) => 6,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Truth {
    True,
    False,
    Unknown { default: bool, failed: Hydrators },
}

impl Truth {
    #[inline]
    pub(crate) fn resolves_true(self) -> bool {
        matches!(self, Truth::True | Truth::Unknown { default: true, .. })
    }

    #[inline]
    fn and(self, other: Self) -> Self {
        match (self, other) {
            (Truth::False, _) | (_, Truth::False) => Truth::False,
            (Truth::True, other) | (other, Truth::True) => other,
            (
                Truth::Unknown {
                    default: a,
                    failed: x,
                },
                Truth::Unknown {
                    default: b,
                    failed: y,
                },
            ) => Truth::Unknown {
                default: a && b,
                failed: x.union(y),
            },
        }
    }

    #[inline]
    fn or(self, other: Self) -> Self {
        match (self, other) {
            (Truth::True, _) | (_, Truth::True) => Truth::True,
            (Truth::False, other) | (other, Truth::False) => other,
            (
                Truth::Unknown {
                    default: a,
                    failed: x,
                },
                Truth::Unknown {
                    default: b,
                    failed: y,
                },
            ) => Truth::Unknown {
                default: a || b,
                failed: x.union(y),
            },
        }
    }
}

impl Not for Truth {
    type Output = Self;

    #[inline]
    fn not(self) -> Self {
        match self {
            Truth::True => Truth::False,
            Truth::False => Truth::True,
            Truth::Unknown { default, failed } => Truth::Unknown {
                default: !default,
                failed,
            },
        }
    }
}

fn appeal(violations: &[FosnrViolation], labels: &SafetyLabelMap) -> Option<Notice> {
    let violation = violations
        .iter()
        .find(|violation| labels.has_label(violation.label))?;
    Some(Notice::Appealable {
        reason: fosnr_reason(violation.label, violation.policy, violation.level, labels),
        limited_actions: violation.limited_actions,
    })
}

fn fosnr_reason(
    label: SafetyLabelType,
    policy: AppealablePolicy,
    level: i8,
    labels: &SafetyLabelMap,
) -> FosnrReason {
    FosnrReason {
        policy,
        level,
        proactive: !labels.is_by_agent(label),
        appeal_submitted: labels.has_label(SafetyLabelType::FOSNR_APPEAL_SUBMITTED),
    }
}

fn account_or_request_country_in(
    profile: Option<&ViewerProfile>,
    request_country: Option<&str>,
    in_list: impl FnOnce(&str) -> bool,
) -> bool {
    profile
        .and_then(|profile| profile.account_country_code.as_deref())
        .or(request_country)
        .is_some_and(in_list)
}

#[cfg(test)]
mod tests {
    use super::leaves::*;
    use super::*;
    use crate::hydration::Hydrator;
    use crate::models::{
        ClientCapability, PostCandidate, VerifyBlurSupport, Viewer, ViewerFeatures,
    };
    use crate::rules::fixtures::{
        candidate, logged_out_viewer, viewer, viewer_with_profile, VIEWER_ID,
    };
    use crate::rules::test_context;

    impl Predicate {
        fn holds_for(self, viewer: &ViewerFeatures, candidate: &PostCandidate) -> bool {
            self.holds(&test_context(viewer, candidate).hydrated_by(self.requirements()))
        }
    }

    #[test]
    fn created_after_is_strict_at_the_unix_millisecond_boundary() {
        const CUTOFF_MS: u64 = 1705536000000;
        const AT_CUTOFF: u64 = (CUTOFF_MS - 1288834974657) << 22;
        let viewer = ViewerFeatures::default();
        for (tweet_id, expected) in [
            (AT_CUTOFF - 1, false),
            (AT_CUTOFF, false),
            (AT_CUTOFF + (1 << 22) - 1, false),
            (AT_CUTOFF + (1 << 22), true),
            (0, false),
        ] {
            let mut candidate = PostCandidate::default();
            candidate.post.tweet_id = tweet_id;
            let context = test_context(&viewer, &candidate);
            assert_eq!(
                Predicate::PostFacts(TweetCreatedAfter(CUTOFF_MS)).holds(&context),
                expected,
                "{tweet_id}"
            );
        }
    }

    #[test]
    fn age_verified_is_the_label_or_idv_premium_and_the_request_country_is_never_unknown() {
        let in_country = |country: Option<&str>, profile: ViewerProfile| ViewerFeatures {
            country_code: country.map(str::to_string),
            ..viewer_with_profile(profile)
        };
        let labeled = ViewerFeatures {
            viewer: Viewer::LoggedIn {
                id: VIEWER_ID,
                profile: ViewerProfile::default(),
                has_age_verified_18_label: true,
            },
            ..ViewerFeatures::default()
        };
        let idv_premium = ViewerProfile {
            has_idv_premium: true,
            ..ViewerProfile::default()
        };
        let account_in_fr = ViewerProfile {
            account_country_code: Some("fr".into()),
            ..ViewerProfile::default()
        };
        let request_country_in = Predicate::Request(RequestCountryIn(CountryList::AgeVerification));
        let age_verified = Predicate::Compound(ViewerAgeVerified);
        let candidate = candidate().build();
        for (predicate, viewer, expected) in [
            (age_verified, labeled, true),
            (age_verified, in_country(None, idv_premium), true),
            (
                age_verified,
                in_country(None, ViewerProfile::default()),
                false,
            ),
            (age_verified, logged_out_viewer(), false),
            (
                request_country_in,
                in_country(Some("fr"), ViewerProfile::default()),
                true,
            ),
            (
                request_country_in,
                in_country(Some("us"), account_in_fr),
                false,
            ),
            (
                request_country_in,
                in_country(None, ViewerProfile::default()),
                false,
            ),
        ] {
            assert_eq!(predicate.holds_for(&viewer, &candidate), expected);
        }

        let mut failed = candidate;
        failed.failed = Hydrators::all();
        let viewer = in_country(Some("fr"), ViewerProfile::default());
        let context = test_context(&viewer, &failed);
        assert_eq!(
            age_verified.truth(&context),
            Truth::Unknown {
                default: false,
                failed: Hydrators::of(Hydrator::ViewerProfile).with(Hydrator::ViewerLabels),
            }
        );
        assert_eq!(request_country_in.truth(&context), Truth::True);
    }

    #[test]
    fn the_client_checks_read_the_resolved_capability_and_are_never_unknown() {
        use VerifyBlurSupport::{IosNeedsUpdate, Supported};
        let viewer = ViewerFeatures {
            client_capability: ClientCapability {
                verify_blur_support: Some(IosNeedsUpdate),
                modern_blur: true,
                stale_tweet_limits: true,
                community_viewer_removed_limits: true,
                gore_blur_ignores_settings: true,
                fosnr_rules: true,
                fosnr_fallback_drops: true,
            },
            ..viewer(VIEWER_ID)
        };
        let mut failed = candidate().build();
        failed.failed = Hydrators::all();
        let context = test_context(&viewer, &failed);
        for (support, expected) in [(IosNeedsUpdate, Truth::True), (Supported, Truth::False)] {
            let check = Predicate::Request(ClientVerifyBlurSupportIs(support));
            assert_eq!(check.truth(&context), expected, "{support:?}");
        }
        for check in
            [ClientHasModernBlur, ClientHasCommunityViewerRemovedLimits].map(Predicate::Request)
        {
            assert_eq!(check.truth(&context), Truth::True, "{check:?}");
        }
    }
}
