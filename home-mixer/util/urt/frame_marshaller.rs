use super::client_event::empty_details;
use xai_home_mixer_proto::Frame;
use xai_urt_thrift::entry::{TimelineEntry, TimelineEntryContent};
use xai_urt_thrift::frame;
use xai_urt_thrift::item::{TimelineItem, TimelineItemContent};
use xai_urt_thrift::metadata::{ClientEventInfo, TimelinesDetails};

const ENTRY_NAMESPACE_FRAME: &str = "frame";
const ELEMENT_FRAME: &str = "frame";
const INJECTION_TYPE: &str = "JetfuelFrame";
const COMPONENT_FRAME: &str = "for_you_jetfuel_frame";

pub(super) fn marshal_frame(f: &Frame, sort_index: i64, page: i64) -> TimelineEntry {
    let item = TimelineItem {
        content: TimelineItemContent::Frame(frame::Frame::new(
            f.payload.clone(),
            f.height as i16,
            f.is_dismissible,
            f.placement_id,
        )),
        client_event_info: Some(build_frame_client_event_info(COMPONENT_FRAME)),
        feedback_info: None,
        prompt: None,
        reactive_triggers: None,
    };

    TimelineEntry {
        entry_id: frame_entry_id(f, page),
        sort_index,
        content: TimelineEntryContent::Item(item),
        expiry_time: None,
    }
}

fn frame_entry_id(f: &Frame, page: i64) -> String {
    let slug = f.route.trim_start_matches('/').replace('/', "-");
    format!("{ENTRY_NAMESPACE_FRAME}-{slug}-{page}-{}", f.occurrence)
}

fn build_frame_client_event_info(component: &str) -> ClientEventInfo {
    let mut details = empty_details();
    details.timelines_details = Some(TimelinesDetails {
        injection_type: Some(INJECTION_TYPE.to_string()),
        controller_data: None,
        source_data: None,
    });
    ClientEventInfo {
        component: Some(component.to_string()),
        element: Some(ELEMENT_FRAME.to_string()),
        details: Some(details),
        action: None,
        entity_token: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frame(occurrence: u32) -> Frame {
        Frame {
            payload: "RENDERED_BINARY".to_string(),
            height: 320,
            is_dismissible: true,
            placement_id: 0,
            route: "/cards/soccer/timelineCarousel".to_string(),
            occurrence,
            fetch_route: "/cards/soccer/timelineCarousel".to_string(),
        }
    }

    #[test]
    fn marshals_frame_entry() {
        let entry = marshal_frame(&frame(0), 0, 7000);
        assert_eq!(entry.entry_id, "frame-cards-soccer-timelineCarousel-7000-0");
        match entry.content {
            TimelineEntryContent::Item(item) => {
                assert!(
                    matches!(item.content, TimelineItemContent::Frame(ref fr) if fr.payload == "RENDERED_BINARY" && fr.height == 320)
                );
            }
            other => panic!("expected Item entry, got {other:?}"),
        }
    }

    #[test]
    fn repeated_occurrences_get_distinct_entry_ids() {
        let ids: Vec<String> = (0..3)
            .map(|n| marshal_frame(&frame(n), 0, 7000).entry_id)
            .collect();
        assert_eq!(
            ids,
            vec![
                "frame-cards-soccer-timelineCarousel-7000-0",
                "frame-cards-soccer-timelineCarousel-7000-1",
                "frame-cards-soccer-timelineCarousel-7000-2",
            ]
        );
    }

    #[test]
    fn the_same_frame_gets_distinct_entry_ids_across_pages() {
        let first_page = marshal_frame(&frame(0), 0, 7000).entry_id;
        let next_page = marshal_frame(&frame(0), 0, 6960).entry_id;
        assert_ne!(first_page, next_page);
    }
}
