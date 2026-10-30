use super::{Role, Row, Subject};
use crate::models::{LimitedEngagementReason, SafetyLabelType};
use crate::rules::fixtures::{
    allow, candidate, dropped, labeled, limited, read_only_viewer, AUTHOR_ID, VIEWER_ID,
};
use crate::rules::SafetyLevel::{
    FilterAll, TimelineHome, TimelineHomeHydration, TimelineHomeRecommendations,
};
use xai_visibility_filtering::models::FilteredReason;

pub(super) fn rows() -> Vec<Row> {
    vec![
        Row {
            name: "pristine",
            subject: Subject::Post(candidate().build()),
            expect: vec![
                (
                    FilterAll,
                    Role::NonFollower,
                    dropped(
                        FilteredReason::UnspecifiedReason,
                        "filter_all/drop/unspecified",
                    ),
                ),
                (
                    FilterAll,
                    Role::Author,
                    dropped(
                        FilteredReason::UnspecifiedReason,
                        "filter_all/drop/unspecified",
                    ),
                ),
                (TimelineHome, Role::NonFollower, allow()),
                (TimelineHome, Role::LoggedOut, allow()),
                (TimelineHomeRecommendations, Role::NonFollower, allow()),
                (
                    TimelineHomeHydration,
                    Role::As("read_only", read_only_viewer(VIEWER_ID)),
                    limited(
                        LimitedEngagementReason::ReadonlyViewer,
                        "read_only_viewer/limited_engagement",
                    ),
                ),
                (
                    TimelineHomeHydration,
                    Role::As("read_only_author", read_only_viewer(AUTHOR_ID)),
                    limited(
                        LimitedEngagementReason::ReadonlyViewer,
                        "read_only_viewer/limited_engagement",
                    ),
                ),
            ],
        },
        Row {
            name: "egregious_nsfw_label",
            subject: Subject::Post(labeled(SafetyLabelType::EGREGIOUS_NSFW)),
            expect: vec![
                (TimelineHome, Role::NonFollower, allow()),
                (TimelineHomeRecommendations, Role::NonFollower, allow()),
            ],
        },
    ]
}
