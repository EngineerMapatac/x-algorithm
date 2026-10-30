use super::{Role, Row, Subject};
use crate::models::{AuthorFeatures, PostCandidate, SafetyLabelType, Verdict};
use crate::rules::fixtures::{allow, appealed, author_candidate, candidate, dropped};
use crate::rules::SafetyLevel;
use crate::rules::SafetyLevel::{
    ImmersiveExpandedRecommendations, TimelineHome, TimelineHomeHydration,
    TimelineHomeRecommendations,
};
use xai_visibility_filtering::models::FilteredReason;
use xai_x_thrift::action::AppealablePolicy;

fn fosnr_post(set: fn(&mut AuthorFeatures)) -> PostCandidate {
    let mut features = AuthorFeatures::default();
    set(&mut features);
    candidate()
        .with_author_features(features)
        .with_label(SafetyLabelType::FOSNR_HATEFUL_CONDUCT)
        .build()
}

fn author_appeals() -> Vec<(SafetyLevel, Role, Verdict)> {
    vec![(
        TimelineHomeHydration,
        Role::Author,
        appealed(
            AppealablePolicy::HATEFUL_CONDUCT,
            3,
            true,
            false,
            "fosnr_author/appealable",
        ),
    )]
}

pub(super) fn rows() -> Vec<Row> {
    vec![
        Row {
            name: "suspended_author",
            subject: Subject::Post(author_candidate(|a| a.is_suspended = true)),
            expect: vec![
                (
                    TimelineHome,
                    Role::NonFollower,
                    dropped(FilteredReason::AuthorIsSuspended, "suspended_author/drop"),
                ),
                (
                    TimelineHome,
                    Role::Follower,
                    dropped(FilteredReason::AuthorIsSuspended, "suspended_author/drop"),
                ),
                (TimelineHome, Role::Author, allow()),
                (
                    TimelineHomeHydration,
                    Role::Follower,
                    dropped(FilteredReason::AuthorIsSuspended, "suspended_author/drop"),
                ),
                (TimelineHomeHydration, Role::Author, allow()),
                (
                    TimelineHomeRecommendations,
                    Role::NonFollower,
                    dropped(FilteredReason::AuthorIsSuspended, "suspended_author/drop"),
                ),
                (
                    ImmersiveExpandedRecommendations,
                    Role::NonFollower,
                    dropped(FilteredReason::AuthorIsSuspended, "suspended_author/drop"),
                ),
            ],
        },
        Row {
            name: "suspended_author_nsfw_media",
            subject: Subject::Post(
                candidate()
                    .with_author_features(AuthorFeatures {
                        is_suspended: true,
                        ..Default::default()
                    })
                    .with_label(SafetyLabelType::NSFW_HIGH_PRECISION)
                    .with_media()
                    .build(),
            ),
            expect: vec![
                (
                    TimelineHome,
                    Role::NonFollower,
                    dropped(FilteredReason::AuthorIsSuspended, "suspended_author/drop"),
                ),
                (
                    TimelineHomeHydration,
                    Role::NonFollower,
                    dropped(FilteredReason::AuthorIsSuspended, "suspended_author/drop"),
                ),
            ],
        },
        Row {
            name: "suspended_author_bounced_post",
            subject: Subject::Post(
                candidate()
                    .with_author_features(AuthorFeatures {
                        is_suspended: true,
                        ..Default::default()
                    })
                    .with_label(SafetyLabelType::BOUNCE)
                    .build(),
            ),
            expect: vec![(
                TimelineHomeHydration,
                Role::NonFollower,
                dropped(FilteredReason::AuthorIsSuspended, "suspended_author/drop"),
            )],
        },
        Row {
            name: "deactivated_author",
            subject: Subject::Post(author_candidate(|a| a.is_deactivated = true)),
            expect: vec![
                (
                    TimelineHome,
                    Role::NonFollower,
                    dropped(
                        FilteredReason::AuthorIsDeactivated,
                        "deactivated_author/drop",
                    ),
                ),
                (
                    TimelineHome,
                    Role::Follower,
                    dropped(
                        FilteredReason::AuthorIsDeactivated,
                        "deactivated_author/drop",
                    ),
                ),
                (TimelineHome, Role::Author, allow()),
                (
                    TimelineHomeHydration,
                    Role::Follower,
                    dropped(
                        FilteredReason::AuthorIsDeactivated,
                        "deactivated_author/drop",
                    ),
                ),
                (
                    TimelineHomeRecommendations,
                    Role::NonFollower,
                    dropped(
                        FilteredReason::AuthorIsDeactivated,
                        "deactivated_author/drop",
                    ),
                ),
                (
                    ImmersiveExpandedRecommendations,
                    Role::NonFollower,
                    dropped(
                        FilteredReason::AuthorIsDeactivated,
                        "deactivated_author/drop",
                    ),
                ),
            ],
        },
        Row {
            name: "suspended_and_deactivated_author",
            subject: Subject::Post(author_candidate(|a| {
                a.is_suspended = true;
                a.is_deactivated = true;
            })),
            expect: vec![
                (
                    TimelineHome,
                    Role::NonFollower,
                    dropped(FilteredReason::AuthorIsSuspended, "suspended_author/drop"),
                ),
                (
                    TimelineHomeHydration,
                    Role::NonFollower,
                    dropped(
                        FilteredReason::AuthorIsDeactivated,
                        "deactivated_author/drop",
                    ),
                ),
            ],
        },
        Row {
            name: "erased_author",
            subject: Subject::Post(author_candidate(|a| a.is_erased = true)),
            expect: vec![
                (
                    TimelineHome,
                    Role::NonFollower,
                    dropped(
                        FilteredReason::AuthorAccountIsInactive,
                        "erased_author/drop/inactive",
                    ),
                ),
                (
                    TimelineHome,
                    Role::Follower,
                    dropped(
                        FilteredReason::AuthorAccountIsInactive,
                        "erased_author/drop/inactive",
                    ),
                ),
                (TimelineHome, Role::Author, allow()),
                (
                    TimelineHomeHydration,
                    Role::Follower,
                    dropped(
                        FilteredReason::AuthorAccountIsInactive,
                        "erased_author/drop/inactive",
                    ),
                ),
                (
                    TimelineHomeRecommendations,
                    Role::NonFollower,
                    dropped(
                        FilteredReason::AuthorAccountIsInactive,
                        "erased_author/drop/inactive",
                    ),
                ),
                (
                    ImmersiveExpandedRecommendations,
                    Role::NonFollower,
                    dropped(
                        FilteredReason::AuthorAccountIsInactive,
                        "erased_author/drop/inactive",
                    ),
                ),
            ],
        },
        Row {
            name: "offboarded_author",
            subject: Subject::Post(author_candidate(|a| a.is_offboarded = true)),
            expect: vec![
                (
                    TimelineHome,
                    Role::NonFollower,
                    dropped(
                        FilteredReason::AuthorAccountIsInactive,
                        "offboarded_author/drop/inactive",
                    ),
                ),
                (
                    TimelineHome,
                    Role::Follower,
                    dropped(
                        FilteredReason::AuthorAccountIsInactive,
                        "offboarded_author/drop/inactive",
                    ),
                ),
                (TimelineHome, Role::Author, allow()),
                (
                    TimelineHomeHydration,
                    Role::Follower,
                    dropped(
                        FilteredReason::AuthorAccountIsInactive,
                        "offboarded_author/drop/inactive",
                    ),
                ),
                (
                    TimelineHomeRecommendations,
                    Role::NonFollower,
                    dropped(
                        FilteredReason::AuthorAccountIsInactive,
                        "offboarded_author/drop/inactive",
                    ),
                ),
                (
                    ImmersiveExpandedRecommendations,
                    Role::NonFollower,
                    dropped(
                        FilteredReason::AuthorAccountIsInactive,
                        "offboarded_author/drop/inactive",
                    ),
                ),
            ],
        },
        Row {
            name: "protected_author",
            subject: Subject::Post(author_candidate(|a| a.is_protected = true)),
            expect: vec![
                (
                    TimelineHome,
                    Role::NonFollower,
                    dropped(FilteredReason::AuthorIsProtected, "protected_author/drop"),
                ),
                (TimelineHome, Role::Follower, allow()),
                (TimelineHome, Role::Author, allow()),
                (
                    TimelineHome,
                    Role::LoggedOut,
                    dropped(FilteredReason::AuthorIsProtected, "protected_author/drop"),
                ),
                (
                    TimelineHomeHydration,
                    Role::NonFollower,
                    dropped(FilteredReason::AuthorIsProtected, "protected_author/drop"),
                ),
                (TimelineHomeHydration, Role::Follower, allow()),
                (TimelineHomeHydration, Role::Author, allow()),
                (
                    TimelineHomeHydration,
                    Role::LoggedOut,
                    dropped(FilteredReason::AuthorIsProtected, "protected_author/drop"),
                ),
                (
                    TimelineHomeRecommendations,
                    Role::NonFollower,
                    dropped(FilteredReason::AuthorIsProtected, "protected_author/drop"),
                ),
                (
                    ImmersiveExpandedRecommendations,
                    Role::NonFollower,
                    dropped(FilteredReason::AuthorIsProtected, "protected_author/drop"),
                ),
            ],
        },
        Row {
            name: "suspended_author_fosnr_label",
            subject: Subject::Post(fosnr_post(|a| a.is_suspended = true)),
            expect: author_appeals(),
        },
        Row {
            name: "deactivated_author_fosnr_label",
            subject: Subject::Post(fosnr_post(|a| a.is_deactivated = true)),
            expect: author_appeals(),
        },
        Row {
            name: "erased_author_fosnr_label",
            subject: Subject::Post(fosnr_post(|a| a.is_erased = true)),
            expect: author_appeals(),
        },
        Row {
            name: "offboarded_author_fosnr_label",
            subject: Subject::Post(fosnr_post(|a| a.is_offboarded = true)),
            expect: author_appeals(),
        },
        Row {
            name: "protected_author_fosnr_label",
            subject: Subject::Post(fosnr_post(|a| a.is_protected = true)),
            expect: author_appeals(),
        },
    ]
}
