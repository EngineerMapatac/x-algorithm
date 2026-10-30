use super::{Role, Row, Subject};
use crate::models::{
    ClientCapability, SafetyLabelType, TombstoneReason, ViewerFeatures, ViewerProfile,
};
use crate::rules::fixtures::{
    allow, appealed, candidate, dropped, labeled, tombstoned, viewer_with_profile,
};
use crate::rules::SafetyLevel::{
    ImmersiveExpandedRecommendations, TimelineHome, TimelineHomeHydration,
    TimelineHomeRecommendations,
};
use xai_visibility_filtering::models::{
    Action, DropReason, FilteredReason, SafetyResult, SafetyResultReason,
};
use xai_x_thrift::action::AppealablePolicy;

fn soft_nsfw_viewer(name: &'static str, is_new_account: bool, switch_on: bool) -> Role {
    let base = viewer_with_profile(ViewerProfile {
        is_new_account,
        ..ViewerProfile::default()
    });
    Role::As(
        name,
        ViewerFeatures {
            client_capability: ClientCapability {
                soft_nsfw_new_user_drop: switch_on,
                ..ClientCapability::default()
            },
            ..base
        },
    )
}

pub(super) fn rows() -> Vec<Row> {
    vec![
        Row {
            name: "soft_nsfw_label",
            subject: Subject::Post(labeled(SafetyLabelType::SOFT_NSFW)),
            expect: vec![
                (
                    TimelineHomeRecommendations,
                    soft_nsfw_viewer("new_account_switch_on", true, true),
                    dropped(
                        FilteredReason::UnspecifiedReason,
                        "soft_nsfw_new_user/drop/unspecified",
                    ),
                ),
                (
                    TimelineHomeRecommendations,
                    soft_nsfw_viewer("new_account_switch_off", true, false),
                    allow(),
                ),
                (
                    TimelineHomeRecommendations,
                    soft_nsfw_viewer("old_account_switch_on", false, true),
                    allow(),
                ),
                (
                    TimelineHomeHydration,
                    soft_nsfw_viewer("new_account_switch_on", true, true),
                    allow(),
                ),
            ],
        },
        Row {
            name: "for_emergency_use_only_label",
            subject: Subject::Post(labeled(SafetyLabelType::FOR_EMERGENCY_USE_ONLY)),
            expect: vec![
                (
                    TimelineHome,
                    Role::each([Role::NonFollower, Role::Follower, Role::Author]),
                    dropped(
                        FilteredReason::UnspecifiedReason,
                        "for_emergency_use_only/drop/unspecified",
                    ),
                ),
                (TimelineHomeHydration, Role::Author, allow()),
                (
                    TimelineHomeRecommendations,
                    Role::NonFollower,
                    dropped(
                        FilteredReason::UnspecifiedReason,
                        "for_emergency_use_only/drop/unspecified",
                    ),
                ),
                (
                    ImmersiveExpandedRecommendations,
                    Role::NonFollower,
                    dropped(
                        FilteredReason::UnspecifiedReason,
                        "for_emergency_use_only/drop/unspecified",
                    ),
                ),
            ],
        },
        Row {
            name: "pdna_label",
            subject: Subject::Post(labeled(SafetyLabelType::PDNA)),
            expect: vec![
                (
                    TimelineHome,
                    Role::each([Role::NonFollower, Role::Follower]),
                    dropped(nsfw_high_precision_reason(), "pdna/drop/safety_result"),
                ),
                (TimelineHome, Role::Author, allow()),
                (
                    TimelineHomeHydration,
                    Role::NonFollower,
                    dropped(nsfw_high_precision_reason(), "pdna/drop/safety_result"),
                ),
                (TimelineHomeHydration, Role::Author, allow()),
                (
                    TimelineHomeRecommendations,
                    Role::NonFollower,
                    dropped(nsfw_high_precision_reason(), "pdna/drop/safety_result"),
                ),
                (
                    ImmersiveExpandedRecommendations,
                    Role::NonFollower,
                    dropped(nsfw_high_precision_reason(), "pdna/drop/safety_result"),
                ),
            ],
        },
        Row {
            name: "bounce_label",
            subject: Subject::Post(labeled(SafetyLabelType::BOUNCE)),
            expect: vec![
                (
                    TimelineHome,
                    Role::each([Role::NonFollower, Role::Follower]),
                    dropped(FilteredReason::TweetIsBounced, "bounce/drop/bounced"),
                ),
                (TimelineHome, Role::Author, allow()),
                (
                    TimelineHomeHydration,
                    Role::NonFollower,
                    dropped(FilteredReason::TweetIsBounced, "bounce/drop/bounced"),
                ),
                (TimelineHomeHydration, Role::Author, allow()),
                (
                    TimelineHomeHydration,
                    Role::InnerQuoted(Box::new(Role::NonFollower)),
                    dropped(FilteredReason::TweetIsBounced, "bounce/drop/bounced"),
                ),
                (
                    TimelineHomeHydration,
                    Role::InnerQuoted(Box::new(Role::LoggedOut)),
                    dropped(FilteredReason::TweetIsBounced, "bounce/drop/bounced"),
                ),
                (
                    TimelineHomeHydration,
                    Role::InnerQuoted(Box::new(Role::Author)),
                    tombstoned(TombstoneReason::Bounced, "bounce_quoted/tombstone/bounced"),
                ),
                (
                    TimelineHomeRecommendations,
                    Role::NonFollower,
                    dropped(FilteredReason::TweetIsBounced, "bounce/drop/bounced"),
                ),
                (
                    ImmersiveExpandedRecommendations,
                    Role::NonFollower,
                    dropped(FilteredReason::TweetIsBounced, "bounce/drop/bounced"),
                ),
            ],
        },
        Row {
            name: "spam_label",
            subject: Subject::Post(labeled(SafetyLabelType::SPAM)),
            expect: vec![
                (
                    TimelineHome,
                    Role::each([Role::NonFollower, Role::Follower]),
                    dropped(FilteredReason::PossiblyUndesirable, "spam/drop/undesirable"),
                ),
                (TimelineHome, Role::Author, allow()),
                (
                    TimelineHomeRecommendations,
                    Role::NonFollower,
                    dropped(FilteredReason::PossiblyUndesirable, "spam/drop/undesirable"),
                ),
                (
                    ImmersiveExpandedRecommendations,
                    Role::NonFollower,
                    dropped(FilteredReason::PossiblyUndesirable, "spam/drop/undesirable"),
                ),
            ],
        },
        Row {
            name: "spam_and_bounce_labels",
            subject: Subject::Post(
                candidate()
                    .with_label(SafetyLabelType::SPAM)
                    .with_label(SafetyLabelType::BOUNCE)
                    .build(),
            ),
            expect: vec![
                (
                    TimelineHome,
                    Role::NonFollower,
                    dropped(FilteredReason::TweetIsBounced, "bounce/drop/bounced"),
                ),
                (
                    TimelineHomeHydration,
                    Role::NonFollower,
                    dropped(FilteredReason::PossiblyUndesirable, "spam/drop/undesirable"),
                ),
            ],
        },
        Row {
            name: "fosnr_hateful_conduct_label",
            subject: Subject::Post(labeled(SafetyLabelType::FOSNR_HATEFUL_CONDUCT)),
            expect: vec![
                (
                    TimelineHome,
                    Role::each([Role::NonFollower, Role::Follower]),
                    dropped(
                        FilteredReason::PossiblyUndesirable,
                        "fosnr_hateful_conduct/drop/undesirable",
                    ),
                ),
                (TimelineHome, Role::Author, allow()),
                (
                    TimelineHomeHydration,
                    Role::Author,
                    appealed(
                        AppealablePolicy::HATEFUL_CONDUCT,
                        3,
                        true,
                        false,
                        "fosnr_author/appealable",
                    ),
                ),
                (
                    TimelineHomeRecommendations,
                    Role::NonFollower,
                    dropped(
                        FilteredReason::PossiblyUndesirable,
                        "fosnr_hateful_conduct/drop/undesirable",
                    ),
                ),
                (
                    ImmersiveExpandedRecommendations,
                    Role::NonFollower,
                    dropped(
                        FilteredReason::PossiblyUndesirable,
                        "fosnr_hateful_conduct/drop/undesirable",
                    ),
                ),
            ],
        },
        Row {
            name: "fosnr_violent_speech_label",
            subject: Subject::Post(labeled(SafetyLabelType::FOSNR_VIOLENT_SPEECH)),
            expect: vec![
                (
                    TimelineHome,
                    Role::each([Role::NonFollower, Role::Follower]),
                    dropped(
                        FilteredReason::PossiblyUndesirable,
                        "fosnr_violent_speech/drop/undesirable",
                    ),
                ),
                (TimelineHome, Role::Author, allow()),
                (
                    TimelineHomeHydration,
                    Role::Author,
                    appealed(
                        AppealablePolicy::VIOLENT_SPEECH,
                        3,
                        true,
                        false,
                        "fosnr_author/appealable",
                    ),
                ),
                (
                    TimelineHomeRecommendations,
                    Role::NonFollower,
                    dropped(
                        FilteredReason::PossiblyUndesirable,
                        "fosnr_violent_speech/drop/undesirable",
                    ),
                ),
                (
                    ImmersiveExpandedRecommendations,
                    Role::NonFollower,
                    dropped(
                        FilteredReason::PossiblyUndesirable,
                        "fosnr_violent_speech/drop/undesirable",
                    ),
                ),
            ],
        },
        Row {
            name: "fosnr_abuse_label",
            subject: Subject::Post(labeled(SafetyLabelType::FOSNR_ABUSE)),
            expect: vec![
                (
                    TimelineHome,
                    Role::each([Role::NonFollower, Role::Follower]),
                    dropped(
                        FilteredReason::PossiblyUndesirable,
                        "fosnr_abuse/drop/undesirable",
                    ),
                ),
                (TimelineHome, Role::Author, allow()),
                (
                    TimelineHomeHydration,
                    Role::NonFollower,
                    dropped(
                        FilteredReason::PossiblyUndesirable,
                        "fosnr_abuse/drop/undesirable",
                    ),
                ),
                (
                    TimelineHomeHydration,
                    Role::Author,
                    appealed(
                        AppealablePolicy::ABUSE,
                        3,
                        true,
                        false,
                        "fosnr_author/appealable",
                    ),
                ),
                (
                    TimelineHomeRecommendations,
                    Role::NonFollower,
                    dropped(
                        FilteredReason::PossiblyUndesirable,
                        "fosnr_abuse/drop/undesirable",
                    ),
                ),
                (
                    ImmersiveExpandedRecommendations,
                    Role::NonFollower,
                    dropped(
                        FilteredReason::PossiblyUndesirable,
                        "fosnr_abuse/drop/undesirable",
                    ),
                ),
            ],
        },
        Row {
            name: "fosnr_civic_integrity_label",
            subject: Subject::Post(labeled(SafetyLabelType::FOSNR_CIVIC_INTEGRITY)),
            expect: vec![
                (
                    TimelineHome,
                    Role::each([Role::NonFollower, Role::Follower]),
                    dropped(
                        FilteredReason::PossiblyUndesirable,
                        "fosnr_civic_integrity/drop/undesirable",
                    ),
                ),
                (TimelineHome, Role::Author, allow()),
                (
                    TimelineHomeHydration,
                    Role::NonFollower,
                    dropped(
                        FilteredReason::PossiblyUndesirable,
                        "fosnr_civic_integrity/drop/undesirable",
                    ),
                ),
                (
                    TimelineHomeHydration,
                    Role::Author,
                    appealed(
                        AppealablePolicy::CIVIC_INTEGRITY,
                        3,
                        true,
                        false,
                        "fosnr_author/appealable",
                    ),
                ),
                (
                    TimelineHomeRecommendations,
                    Role::NonFollower,
                    dropped(
                        FilteredReason::PossiblyUndesirable,
                        "fosnr_civic_integrity/drop/undesirable",
                    ),
                ),
                (
                    ImmersiveExpandedRecommendations,
                    Role::NonFollower,
                    dropped(
                        FilteredReason::PossiblyUndesirable,
                        "fosnr_civic_integrity/drop/undesirable",
                    ),
                ),
            ],
        },
        Row {
            name: "fosnr_level_3_and_level_1_labels",
            subject: Subject::Post(
                candidate()
                    .with_label(SafetyLabelType::FOSNR_VIOLENT_SPEECH)
                    .with_label(SafetyLabelType::FOSNR_ABUSE_INSULTS)
                    .with_agent_label(SafetyLabelType::FOSNR_CIVIC_INTEGRITY)
                    .build(),
            ),
            expect: vec![
                (
                    TimelineHomeHydration,
                    Role::Author,
                    appealed(
                        AppealablePolicy::CIVIC_INTEGRITY,
                        3,
                        false,
                        false,
                        "fosnr_author/appealable",
                    ),
                ),
                (
                    TimelineHomeHydration,
                    Role::NonFollower,
                    dropped(
                        FilteredReason::PossiblyUndesirable,
                        "fosnr_violent_speech/drop/undesirable",
                    ),
                ),
            ],
        },
        Row {
            name: "fosnr_hateful_conduct_and_bounce_labels",
            subject: Subject::Post(
                candidate()
                    .with_label(SafetyLabelType::FOSNR_HATEFUL_CONDUCT)
                    .with_label(SafetyLabelType::BOUNCE)
                    .build(),
            ),
            expect: vec![
                (
                    TimelineHome,
                    Role::NonFollower,
                    dropped(FilteredReason::TweetIsBounced, "bounce/drop/bounced"),
                ),
                (
                    TimelineHomeHydration,
                    Role::NonFollower,
                    dropped(
                        FilteredReason::PossiblyUndesirable,
                        "fosnr_hateful_conduct/drop/undesirable",
                    ),
                ),
            ],
        },
        Row {
            name: "fosnr_abuse_insults_and_bounce_labels",
            subject: Subject::Post(
                candidate()
                    .with_label(SafetyLabelType::FOSNR_ABUSE_INSULTS)
                    .with_label(SafetyLabelType::BOUNCE)
                    .build(),
            ),
            expect: vec![
                (
                    TimelineHomeHydration,
                    Role::NonFollower,
                    dropped(
                        FilteredReason::PossiblyUndesirable,
                        "fosnr_abuse_insults_non_follower/drop/undesirable",
                    ),
                ),
                (
                    TimelineHomeHydration,
                    Role::Follower,
                    dropped(FilteredReason::TweetIsBounced, "bounce/drop/bounced"),
                ),
            ],
        },
    ]
}

fn nsfw_high_precision_reason() -> FilteredReason {
    FilteredReason::SafetyResult(SafetyResult {
        reason: Some(SafetyResultReason::NsfwHighPrecision),
        action: Action::Drop(DropReason {}),
    })
}
