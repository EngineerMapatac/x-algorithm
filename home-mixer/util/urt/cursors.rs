use xai_candidate_pipeline::component_library::utils::current_time_to_id;
use xai_urt_thrift::cursor::{RequestCursor, UrtOrderedCursor};
use xai_urt_thrift::entry::{TimelineEntry, TimelineEntryContent};
use xai_urt_thrift::operation::{CursorType, TimelineCursor, TimelineOperation};

const TOP_CURSOR_OFFSET: i64 = 10000;
const CURSOR_TOP_ENTRY_ID: &str = "cursor-top";
const CURSOR_BOTTOM_ENTRY_ID: &str = "cursor-bottom";

pub(super) fn initial_sort_index(cursor: Option<&UrtOrderedCursor>) -> i64 {
    let is_top = cursor
        .and_then(|c| c.cursor_type.as_ref())
        .map(|ct| *ct == CursorType::TOP)
        .unwrap_or(false);

    if is_top {
        current_time_to_id()
    } else {
        cursor
            .map(|c| c.initial_sort_index)
            .unwrap_or_else(current_time_to_id)
    }
}

pub(super) fn build_cursors(
    entries: &[TimelineEntry],
    initial_sort_index: i64,
    top_post_id: Option<i64>,
    bottom_post_id: Option<i64>,
) -> (TimelineEntry, TimelineEntry) {
    let top_sort_index = entries
        .first()
        .map(|e| e.sort_index + 1)
        .unwrap_or(initial_sort_index + 1);

    let bottom_sort_index = entries
        .last()
        .map(|e| e.sort_index - 1)
        .unwrap_or(initial_sort_index);

    let top_cursor = build_cursor_entry(
        CURSOR_TOP_ENTRY_ID,
        top_sort_index,
        CursorType::TOP,
        &UrtOrderedCursor::new(
            top_sort_index + TOP_CURSOR_OFFSET,
            top_post_id.or(Some(0)),
            Some(CursorType::TOP),
            None::<i64>,
        ),
    );

    let bottom_cursor = build_cursor_entry(
        CURSOR_BOTTOM_ENTRY_ID,
        bottom_sort_index,
        CursorType::BOTTOM,
        &UrtOrderedCursor::new(
            bottom_sort_index - 2,
            bottom_post_id.or(Some(0)),
            Some(CursorType::BOTTOM),
            None::<i64>,
        ),
    );

    (top_cursor, bottom_cursor)
}

fn build_cursor_entry(
    entry_id: &str,
    sort_index: i64,
    cursor_type: CursorType,
    cursor: &UrtOrderedCursor,
) -> TimelineEntry {
    let cursor_value =
        xai_urt_thrift::cursor_to_string(&RequestCursor::UrtOrderedCursor(cursor.clone()))
            .expect("cursor serialization should not fail");

    TimelineEntry {
        entry_id: format!("{}-{}", entry_id, sort_index),
        sort_index,
        content: TimelineEntryContent::Operation(TimelineOperation::Cursor(TimelineCursor {
            value: cursor_value,
            cursor_type,
            display_treatment: None,
            stop_on_empty_response: None,
            cursor_info: None,
            url: None,
        })),
        expiry_time: None,
    }
}
