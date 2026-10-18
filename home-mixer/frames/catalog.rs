use super::placement::Placement;
use crate::models::query::ScoredPostsQuery;
use crate::params::{
    EnableJetfuelFrameMlbCarousel, EnableJetfuelFrameNfl, EnableJetfuelFrameNflTimelinePinned,
    EnableJetfuelFrameNflTimelineRotation, EnableJetfuelFrameSoccer, EnableJetfuelFrames,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cadence {
    FirstResponseOnly,
    EveryResponse,
}

impl Cadence {
    fn should_emit(self, query: &ScoredPostsQuery) -> bool {
        match self {
            Cadence::FirstResponseOnly => !query.is_bottom_request,
            Cadence::EveryResponse => true,
        }
    }
}

pub struct FrameDef {
    pub route: &'static str,
    pub is_dismissible: bool,
    pub placement_id: i64,
    pub placement: Placement,
    pub cadence: Cadence,
    pub enabled: fn(&ScoredPostsQuery) -> bool,
}

pub struct TopicTimeline {
    pub topic_id: i64,
    pub enabled: fn(&ScoredPostsQuery) -> bool,
    pub frames: &'static [FrameDef],
}

pub const SOCCER_TOPIC_ID: i64 = 1925950498420469760;
pub const NFL_TOPIC_ID: i64 = 1925951614067650560;
pub const MLB_TOPIC_ID: i64 = 1925951672561319936;
pub const BASEBALL_TOPIC_ID: i64 = 1925950576740675584;

const NFL_ROTATION_INTERVAL: usize = 7;

pub const TIMELINES: &[TopicTimeline] = &[
    TopicTimeline {
        topic_id: SOCCER_TOPIC_ID,
        enabled: |q| q.params.get(EnableJetfuelFrameSoccer),
        frames: &[FrameDef {
            route: "/cards/soccer/timelineCarousel",
            is_dismissible: false,
            placement_id: 0,
            placement: Placement::Pinned { rank: 0 },
            cadence: Cadence::FirstResponseOnly,
            enabled: |_| true,
        }],
    },
    TopicTimeline {
        topic_id: NFL_TOPIC_ID,
        enabled: |q| q.params.get(EnableJetfuelFrameNfl),
        frames: &[
            FrameDef {
                route: "/cards/nfl/timeline/pinned",
                is_dismissible: false,
                placement_id: 0,
                placement: Placement::Pinned { rank: 0 },
                cadence: Cadence::FirstResponseOnly,
                enabled: |q| q.params.get(EnableJetfuelFrameNflTimelinePinned),
            },
            FrameDef {
                route: "/cards/nfl/timeline/card",
                is_dismissible: false,
                placement_id: 0,
                placement: Placement::EveryNItems {
                    first_slot: NFL_ROTATION_INTERVAL,
                    interval: NFL_ROTATION_INTERVAL,
                    rotating: true,
                },
                cadence: Cadence::EveryResponse,
                enabled: |q| q.params.get(EnableJetfuelFrameNflTimelineRotation),
            },
        ],
    },
    TopicTimeline {
        topic_id: MLB_TOPIC_ID,
        enabled: |_| true,
        frames: &[FrameDef {
            route: "/cards/mlb/timelineCarousel",
            is_dismissible: false,
            placement_id: 0,
            placement: Placement::Pinned { rank: 0 },
            cadence: Cadence::FirstResponseOnly,
            enabled: |q| q.params.get(EnableJetfuelFrameMlbCarousel),
        }],
    },
    TopicTimeline {
        topic_id: BASEBALL_TOPIC_ID,
        enabled: |_| true,
        frames: &[FrameDef {
            route: "/cards/mlb/timelineCarousel",
            is_dismissible: false,
            placement_id: 0,
            placement: Placement::Pinned { rank: 0 },
            cadence: Cadence::FirstResponseOnly,
            enabled: |q| q.params.get(EnableJetfuelFrameMlbCarousel),
        }],
    },
];

pub fn eligible(query: &ScoredPostsQuery) -> impl Iterator<Item = &'static FrameDef> + '_ {
    eligible_in(TIMELINES, query)
}

pub fn placement_for_route(route: &str) -> Option<Placement> {
    placement_in(TIMELINES, route)
}

pub(super) fn eligible_in<'a>(
    timelines: &'static [TopicTimeline],
    query: &'a ScoredPostsQuery,
) -> impl Iterator<Item = &'static FrameDef> + 'a {
    let jetfuel_enabled = query.params.get(EnableJetfuelFrames);
    timelines
        .iter()
        .filter(move |timeline| {
            jetfuel_enabled
                && query.topic_ids.contains(&timeline.topic_id)
                && (timeline.enabled)(query)
        })
        .flat_map(|timeline| timeline.frames.iter())
        .filter(move |frame| frame.cadence.should_emit(query) && (frame.enabled)(query))
}

pub(super) fn placement_in(timelines: &'static [TopicTimeline], route: &str) -> Option<Placement> {
    timelines
        .iter()
        .flat_map(|timeline| timeline.frames.iter())
        .find(|frame| frame.route == route)
        .map(|frame| frame.placement)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cadence_controls_scroll_down_eligibility() {
        let first_response = ScoredPostsQuery::default();
        let scroll_down = ScoredPostsQuery {
            is_bottom_request: true,
            ..Default::default()
        };
        assert!(Cadence::FirstResponseOnly.should_emit(&first_response));
        assert!(!Cadence::FirstResponseOnly.should_emit(&scroll_down));
        assert!(Cadence::EveryResponse.should_emit(&first_response));
        assert!(Cadence::EveryResponse.should_emit(&scroll_down));
    }
}
