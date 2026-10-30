use super::{Role, Row, Subject};
use crate::hydration::Hydrator;
use crate::models::{
    ClientCapability, ClientSwitch, SafetyLabelType, ViewerFeatures, ViewerProfile,
};
use crate::rules::fixtures::{
    allow, appealed, blurred, candidate, controlled_root, dropped, labeled, noticed, on_client,
    viewer, viewer_in_country, viewer_with_profile, AUTHOR_ID, VIEWER_ID,
};
use crate::rules::SafetyLevel::{
    ImmersiveExpandedRecommendations, TimelineHome, TimelineHomeHydration,
    TimelineHomeRecommendations,
};
use xai_core_entities::entities::ConversationControlArm;
use xai_visibility_filtering::models::FilteredReason;
use xai_x_thrift::action::{AppealablePolicy, InterstitialReason};

pub(super) fn rows() -> Vec<Row> {
    vec![
        Row {
            name: "malicious_url_label",
            subject: Subject::Post(labeled(SafetyLabelType::MALICIOUS_URL)),
            expect: vec![
                (
                    TimelineHomeRecommendations,
                    Role::each([Role::NonFollower, Role::Follower]),
                    dropped(
                        FilteredReason::PossiblyUndesirable,
                        "malicious_url/drop/undesirable",
                    ),
                ),
                (TimelineHomeRecommendations, Role::Author, allow()),
                (TimelineHome, Role::NonFollower, allow()),
                (
                    ImmersiveExpandedRecommendations,
                    Role::NonFollower,
                    dropped(
                        FilteredReason::PossiblyUndesirable,
                        "malicious_url/drop/undesirable",
                    ),
                ),
            ],
        },
        Row {
            name: "nsfw_high_recall_label",
            subject: Subject::Post(labeled(SafetyLabelType::NSFW_HIGH_RECALL)),
            expect: vec![
                (
                    TimelineHomeRecommendations,
                    Role::each([Role::NonFollower, Role::Follower]),
                    dropped(
                        FilteredReason::ContainNsfwMedia,
                        "nsfw_high_recall/drop/nsfw_media",
                    ),
                ),
                (TimelineHomeRecommendations, Role::Author, allow()),
            ],
        },
        Row {
            name: "do_not_amplify_label",
            subject: Subject::Post(labeled(SafetyLabelType::DO_NOT_AMPLIFY)),
            expect: vec![
                (
                    TimelineHomeRecommendations,
                    Role::each([Role::NonFollower, Role::Follower]),
                    dropped(
                        FilteredReason::PossiblyUndesirable,
                        "do_not_amplify/drop/undesirable",
                    ),
                ),
                (TimelineHomeRecommendations, Role::Author, allow()),
                (
                    ImmersiveExpandedRecommendations,
                    Role::NonFollower,
                    dropped(
                        FilteredReason::PossiblyUndesirable,
                        "do_not_amplify/drop/undesirable",
                    ),
                ),
            ],
        },
        Row {
            name: "spam_high_recall_label",
            subject: Subject::Post(labeled(SafetyLabelType::SPAM_HIGH_RECALL)),
            expect: vec![
                (
                    TimelineHomeRecommendations,
                    Role::each([Role::NonFollower, Role::Follower]),
                    dropped(
                        FilteredReason::PossiblyUndesirable,
                        "spam_high_recall/drop/undesirable",
                    ),
                ),
                (TimelineHomeRecommendations, Role::Author, allow()),
                (TimelineHome, Role::NonFollower, allow()),
                (
                    ImmersiveExpandedRecommendations,
                    Role::NonFollower,
                    dropped(
                        FilteredReason::PossiblyUndesirable,
                        "spam_high_recall/drop/undesirable",
                    ),
                ),
            ],
        },
        Row {
            name: "brazil_election_legal_label",
            subject: Subject::Post(labeled(SafetyLabelType::BRAZIL_ELECTION_LEGAL)),
            expect: vec![
                (
                    TimelineHomeRecommendations,
                    Role::As("in_br", viewer_in_country("br")),
                    dropped(
                        FilteredReason::PossiblyUndesirable,
                        "brazil_election_legal/drop/undesirable",
                    ),
                ),
                (
                    TimelineHomeRecommendations,
                    Role::As("in_us", viewer_in_country("us")),
                    allow(),
                ),
                (
                    TimelineHomeRecommendations,
                    Role::As(
                        "account_br_no_request_country",
                        viewer_with_profile(ViewerProfile {
                            account_country_code: Some("br".to_string()),
                            ..ViewerProfile::default()
                        }),
                    ),
                    allow(),
                ),
                (
                    TimelineHomeRecommendations,
                    Role::As(
                        "author_in_br",
                        ViewerFeatures {
                            country_code: Some("br".to_string()),
                            ..viewer(AUTHOR_ID)
                        },
                    ),
                    allow(),
                ),
                (TimelineHomeRecommendations, Role::NonFollower, allow()),
                (
                    TimelineHome,
                    Role::As("in_br", viewer_in_country("br")),
                    allow(),
                ),
                (
                    ImmersiveExpandedRecommendations,
                    Role::As("in_br", viewer_in_country("br")),
                    dropped(
                        FilteredReason::PossiblyUndesirable,
                        "brazil_election_legal/drop/undesirable",
                    ),
                ),
            ],
        },
        Row {
            name: "brazil_election_legal_label_followed",
            subject: Subject::Post(
                candidate()
                    .with_label(SafetyLabelType::BRAZIL_ELECTION_LEGAL)
                    .with_edge(Hydrator::Follows)
                    .build(),
            ),
            expect: vec![(
                TimelineHomeRecommendations,
                Role::As("follower_in_br", viewer_in_country("br")),
                dropped(
                    FilteredReason::PossiblyUndesirable,
                    "brazil_election_legal/drop/undesirable",
                ),
            )],
        },
        Row {
            name: "fosnr_abuse_insults_label",
            subject: Subject::Post(labeled(SafetyLabelType::FOSNR_ABUSE_INSULTS)),
            expect: vec![
                (
                    TimelineHomeRecommendations,
                    Role::each([Role::NonFollower, Role::Follower]),
                    dropped(
                        FilteredReason::PossiblyUndesirable,
                        "fosnr_abuse_insults/drop/undesirable",
                    ),
                ),
                (TimelineHomeRecommendations, Role::Author, allow()),
                (TimelineHome, Role::NonFollower, allow()),
                (
                    TimelineHomeHydration,
                    Role::each([Role::NonFollower, Role::LoggedOut]),
                    dropped(
                        FilteredReason::PossiblyUndesirable,
                        "fosnr_abuse_insults_non_follower/drop/undesirable",
                    ),
                ),
                (
                    TimelineHomeHydration,
                    Role::Follower,
                    noticed(
                        true,
                        false,
                        "fosnr_abuse_insults_follower/soft_intervention/abuse",
                    ),
                ),
                (
                    TimelineHomeHydration,
                    Role::Author,
                    appealed(
                        AppealablePolicy::ABUSE,
                        1,
                        true,
                        false,
                        "fosnr_author/appealable",
                    ),
                ),
                (
                    TimelineHomeHydration,
                    Role::As("client_without_fosnr", client_without_fosnr(VIEWER_ID)),
                    dropped(
                        FilteredReason::PossiblyUndesirable,
                        "fosnr_fallback/drop/undesirable",
                    ),
                ),
                (
                    TimelineHomeHydration,
                    Role::As(
                        "author_on_client_without_fosnr",
                        client_without_fosnr(AUTHOR_ID),
                    ),
                    dropped(
                        FilteredReason::PossiblyUndesirable,
                        "fosnr_fallback/drop/undesirable",
                    ),
                ),
            ],
        },
        Row {
            name: "fosnr_abuse_insults_by_agent_under_appeal",
            subject: Subject::Post(
                candidate()
                    .with_agent_label(SafetyLabelType::FOSNR_ABUSE_INSULTS)
                    .with_label(SafetyLabelType::FOSNR_APPEAL_SUBMITTED)
                    .build(),
            ),
            expect: vec![
                (
                    TimelineHomeHydration,
                    Role::Follower,
                    noticed(
                        false,
                        true,
                        "fosnr_abuse_insults_follower/soft_intervention/abuse",
                    ),
                ),
                (
                    TimelineHomeHydration,
                    Role::Author,
                    appealed(
                        AppealablePolicy::ABUSE,
                        1,
                        false,
                        true,
                        "fosnr_author/appealable",
                    ),
                ),
            ],
        },
        Row {
            name: "fosnr_abuse_insults_on_nsfw_media",
            subject: Subject::Post(
                candidate()
                    .with_label(SafetyLabelType::FOSNR_ABUSE_INSULTS)
                    .with_label(SafetyLabelType::NSFW_HIGH_PRECISION)
                    .with_media()
                    .with_edge(Hydrator::Follows)
                    .build(),
            ),
            expect: vec![(
                TimelineHomeHydration,
                Role::As(
                    "web_follower_in_us",
                    on_client("web", "us", viewer(VIEWER_ID)),
                ),
                blurred(
                    InterstitialReason::Sensitive(true),
                    "nsfw_high_precision/blur/sensitive",
                ),
            )],
        },
        Row {
            name: "fosnr_abuse_insults_in_by_invitation_conversation",
            subject: Subject::Post(
                candidate()
                    .with_label(SafetyLabelType::FOSNR_ABUSE_INSULTS)
                    .with_conversation_control(controlled_root(
                        ConversationControlArm::ByInvitation,
                    ))
                    .build(),
            ),
            expect: vec![(
                TimelineHomeHydration,
                Role::Follower,
                noticed(
                    true,
                    false,
                    "fosnr_abuse_insults_follower/soft_intervention/abuse",
                ),
            )],
        },
    ]
}

fn client_without_fosnr(viewer_id: u64) -> ViewerFeatures {
    ViewerFeatures {
        client_capability: ClientCapability::DEFAULT
            .without(ClientSwitch::FosnrRules)
            .with(ClientSwitch::FosnrFallbackDrops),
        ..viewer(viewer_id)
    }
}
