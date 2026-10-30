use super::{Role, Row, Subject};
use crate::models::{PostCandidate, TweetFeatures, ViewerFeatures};
use crate::rules::fixtures::{
    allow, author_viewer, candidate, dropped, tweet_candidate, viewer_in_country,
};
use crate::rules::SafetyLevel::{
    ImmersiveExpandedRecommendations, TimelineHome, TimelineHomeHydration,
    TimelineHomeRecommendations,
};
use xai_core_entities::entities::TakedownReason;
use xai_visibility_filtering::models::FilteredReason;

pub(super) fn rows() -> Vec<Row> {
    vec![
        Row {
            name: "legal_takedown_us",
            subject: Subject::Post(takedown_candidate(TakedownReason::LegalRequest {
                country_code: "us".to_string(),
            })),
            expect: vec![
                (
                    TimelineHome,
                    Role::As("in_us", viewer_in_country("us")),
                    dropped(
                        FilteredReason::UnspecifiedReason,
                        "legal_takedown/drop/unspecified",
                    ),
                ),
                (TimelineHome, Role::NonFollower, allow()),
                (
                    TimelineHome,
                    Role::As("in_fr", viewer_in_country("fr")),
                    allow(),
                ),
                (
                    TimelineHomeHydration,
                    Role::As("in_us", viewer_in_country("us")),
                    dropped(
                        FilteredReason::UnspecifiedReason,
                        "legal_takedown/drop/unspecified",
                    ),
                ),
                (
                    TimelineHomeHydration,
                    Role::As("in_fr", viewer_in_country("fr")),
                    allow(),
                ),
            ],
        },
        Row {
            name: "legal_takedown_worldwide",
            subject: Subject::Post(takedown_candidate(TakedownReason::LegalRequest {
                country_code: "xx".to_string(),
            })),
            expect: vec![
                (
                    TimelineHome,
                    Role::As("in_us", viewer_in_country("us")),
                    dropped(
                        FilteredReason::UnspecifiedReason,
                        "legal_takedown/drop/unspecified",
                    ),
                ),
                (
                    TimelineHome,
                    Role::NonFollower,
                    dropped(
                        FilteredReason::UnspecifiedReason,
                        "legal_takedown/drop/unspecified",
                    ),
                ),
                (
                    TimelineHomeRecommendations,
                    Role::NonFollower,
                    dropped(
                        FilteredReason::UnspecifiedReason,
                        "legal_takedown/drop/unspecified",
                    ),
                ),
                (
                    ImmersiveExpandedRecommendations,
                    Role::NonFollower,
                    dropped(
                        FilteredReason::UnspecifiedReason,
                        "legal_takedown/drop/unspecified",
                    ),
                ),
            ],
        },
        Row {
            name: "legal_takedown_copyright",
            subject: Subject::Post(takedown_candidate(TakedownReason::LegalRequest {
                country_code: "xy".to_string(),
            })),
            expect: vec![(TimelineHome, Role::NonFollower, allow())],
        },
        Row {
            name: "unspecified_takedown_worldwide",
            subject: Subject::Post(takedown_candidate(TakedownReason::UnspecifiedReason {
                country_code: "xx".to_string(),
            })),
            expect: vec![(
                TimelineHome,
                Role::NonFollower,
                dropped(
                    FilteredReason::UnspecifiedReason,
                    "legal_takedown/drop/unspecified",
                ),
            )],
        },
        Row {
            name: "unspecified_takedown_copyright",
            subject: Subject::Post(takedown_candidate(TakedownReason::UnspecifiedReason {
                country_code: "xy".to_string(),
            })),
            expect: vec![(
                TimelineHome,
                Role::NonFollower,
                dropped(
                    FilteredReason::UnspecifiedReason,
                    "legal_takedown/drop/unspecified",
                ),
            )],
        },
        Row {
            name: "local_laws_takedown_worldwide",
            subject: Subject::Post(takedown_candidate(TakedownReason::BystanderReport {
                country_code: "xx".to_string(),
            })),
            expect: vec![(
                TimelineHome,
                Role::As("in_us", viewer_in_country("us")),
                allow(),
            )],
        },
        Row {
            name: "dmca_takedown",
            subject: Subject::Post(takedown_candidate(TakedownReason::Dmca)),
            expect: vec![
                (
                    TimelineHome,
                    Role::NonFollower,
                    dropped(
                        FilteredReason::UnspecifiedReason,
                        "legal_takedown/drop/unspecified",
                    ),
                ),
                (TimelineHome, Role::Author, allow()),
            ],
        },
        Row {
            name: "local_laws_takedown_de",
            subject: Subject::Post(takedown_candidate(TakedownReason::BystanderReport {
                country_code: "de".to_string(),
            })),
            expect: vec![
                (
                    TimelineHome,
                    Role::As("in_de", viewer_in_country("de")),
                    dropped(
                        FilteredReason::UnspecifiedReason,
                        "local_laws_takedown/drop/unspecified",
                    ),
                ),
                (
                    TimelineHome,
                    Role::As("in_us", viewer_in_country("us")),
                    allow(),
                ),
                (
                    TimelineHomeHydration,
                    Role::As("in_de", viewer_in_country("de")),
                    dropped(
                        FilteredReason::UnspecifiedReason,
                        "local_laws_takedown/drop/unspecified",
                    ),
                ),
                (
                    TimelineHomeHydration,
                    Role::As("in_us", viewer_in_country("us")),
                    allow(),
                ),
                (TimelineHome, Role::NonFollower, allow()),
                (
                    TimelineHome,
                    Role::As(
                        "author_in_de",
                        ViewerFeatures {
                            country_code: Some("de".to_string()),
                            ..author_viewer()
                        },
                    ),
                    allow(),
                ),
                (
                    TimelineHomeRecommendations,
                    Role::As("in_de", viewer_in_country("de")),
                    dropped(
                        FilteredReason::UnspecifiedReason,
                        "local_laws_takedown/drop/unspecified",
                    ),
                ),
                (
                    ImmersiveExpandedRecommendations,
                    Role::As("in_de", viewer_in_country("de")),
                    dropped(
                        FilteredReason::UnspecifiedReason,
                        "local_laws_takedown/drop/unspecified",
                    ),
                ),
            ],
        },
        Row {
            name: "non_country_takedown",
            subject: Subject::Post(tweet_candidate(|t| {
                t.takedown_reasons = vec![TakedownReason::HatefulImagery, TakedownReason::Unknown]
            })),
            expect: vec![(
                TimelineHome,
                Role::As("in_de", viewer_in_country("de")),
                allow(),
            )],
        },
        Row {
            name: "legal_takedown_uppercase_worldwide",
            subject: Subject::Post(takedown_candidate(TakedownReason::LegalRequest {
                country_code: "XX".to_string(),
            })),
            expect: vec![
                (
                    TimelineHome,
                    Role::As("in_us", viewer_in_country("us")),
                    dropped(
                        FilteredReason::UnspecifiedReason,
                        "legal_takedown/drop/unspecified",
                    ),
                ),
                (
                    TimelineHome,
                    Role::NonFollower,
                    dropped(
                        FilteredReason::UnspecifiedReason,
                        "legal_takedown/drop/unspecified",
                    ),
                ),
            ],
        },
        Row {
            name: "legal_takedown_uppercase_copyright",
            subject: Subject::Post(takedown_candidate(TakedownReason::LegalRequest {
                country_code: "XY".to_string(),
            })),
            expect: vec![
                (
                    TimelineHome,
                    Role::As("in_us", viewer_in_country("us")),
                    allow(),
                ),
                (TimelineHome, Role::NonFollower, allow()),
            ],
        },
        Row {
            name: "local_laws_takedown_uppercase_copyright",
            subject: Subject::Post(takedown_candidate(TakedownReason::BystanderReport {
                country_code: "XY".to_string(),
            })),
            expect: vec![
                (
                    TimelineHome,
                    Role::As("in_us", viewer_in_country("us")),
                    allow(),
                ),
                (TimelineHome, Role::NonFollower, allow()),
            ],
        },
    ]
}

fn takedown_candidate(reason: TakedownReason) -> PostCandidate {
    candidate()
        .with_tweet_features(TweetFeatures {
            takedown_reasons: vec![reason],
            ..Default::default()
        })
        .build()
}
