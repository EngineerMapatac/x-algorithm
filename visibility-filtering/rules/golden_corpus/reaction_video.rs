use super::{Role, Row, Subject};
use crate::models::SafetyLabelType;
use crate::rules::fixtures::{allow, blurred, candidate, dropped, sensitive_opt_in_viewer};
use crate::rules::SafetyLevel::{TimelineHome, TimelineHomeHydration};
use xai_visibility_filtering::models::FilteredReason;
use xai_x_thrift::action::InterstitialReason;

pub(super) fn rows() -> Vec<Row> {
    let adult = || {
        blurred(
            InterstitialReason::Sensitive(true),
            "original_tweet_adult_media_propagation/blur/sensitive",
        )
    };
    let gore = || {
        blurred(
            InterstitialReason::Violence(true),
            "original_tweet_gore_media_propagation/blur/violence",
        )
    };
    let protected = || {
        dropped(
            FilteredReason::AuthorIsProtected,
            "protected_original_author/drop",
        )
    };
    let nsfw_view = || Role::As("nsfw_view", sensitive_opt_in_viewer());
    vec![
        Row {
            name: "reaction_to_adult_post",
            subject: Subject::Post(
                candidate()
                    .with_reacted_to_post_labels(&[SafetyLabelType::NSFW_HIGH_PRECISION])
                    .build(),
            ),
            expect: vec![
                (TimelineHomeHydration, Role::NonFollower, adult()),
                (TimelineHomeHydration, Role::Follower, adult()),
                (TimelineHomeHydration, Role::LoggedOut, adult()),
                (TimelineHomeHydration, Role::Author, adult()),
                (TimelineHomeHydration, nsfw_view(), adult()),
                (TimelineHome, Role::NonFollower, allow()),
            ],
        },
        Row {
            name: "reaction_to_adult_and_gore_post",
            subject: Subject::Post(
                candidate()
                    .with_reacted_to_post_labels(&[
                        SafetyLabelType::NSFW_HIGH_PRECISION,
                        SafetyLabelType::GORE_AND_VIOLENCE_HIGH_PRECISION,
                    ])
                    .build(),
            ),
            expect: vec![(TimelineHomeHydration, Role::NonFollower, adult())],
        },
        Row {
            name: "gore_post_reacting_to_adult_post",
            subject: Subject::Post(
                candidate()
                    .with_label(SafetyLabelType::GORE_AND_VIOLENCE_HIGH_PRECISION)
                    .with_reacted_to_post_labels(&[SafetyLabelType::NSFW_HIGH_PRECISION])
                    .build(),
            ),
            expect: vec![(TimelineHomeHydration, Role::NonFollower, adult())],
        },
        Row {
            name: "reaction_to_protected_author_post",
            subject: Subject::Post(
                candidate()
                    .with_reacted_to_post_labels(&[SafetyLabelType::NSFW_HIGH_PRECISION])
                    .with_protected_reacted_to_author()
                    .build(),
            ),
            expect: vec![
                (TimelineHomeHydration, Role::NonFollower, protected()),
                (TimelineHomeHydration, Role::Follower, protected()),
                (TimelineHomeHydration, Role::Author, protected()),
                (TimelineHomeHydration, nsfw_view(), protected()),
                (TimelineHome, Role::NonFollower, allow()),
            ],
        },
        Row {
            name: "reaction_to_gore_post",
            subject: Subject::Post(
                candidate()
                    .with_reacted_to_post_labels(&[
                        SafetyLabelType::GORE_AND_VIOLENCE_HIGH_PRECISION,
                    ])
                    .build(),
            ),
            expect: vec![
                (TimelineHomeHydration, Role::NonFollower, gore()),
                (TimelineHomeHydration, Role::Author, gore()),
                (TimelineHomeHydration, nsfw_view(), gore()),
            ],
        },
    ]
}
