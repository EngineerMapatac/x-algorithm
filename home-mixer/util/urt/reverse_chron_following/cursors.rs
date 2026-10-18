use crate::models::query::FollowingPaginationMeta;
use xai_candidate_pipeline::component_library::utils::current_time_to_id;
use xai_home_mixer_proto::feed_item::Item as FeedItemKind;
use xai_home_mixer_proto::FeedItem;
use xai_urt_thrift::cursor::{RequestCursor, UrtOrderedCursor};
use xai_urt_thrift::entry::{TimelineEntry, TimelineEntryContent};
use xai_urt_thrift::operation::{CursorType, TimelineCursor, TimelineOperation};

const TOP_CURSOR_OFFSET: i64 = 10000;
const CURSOR_TOP_ENTRY_ID: &str = "cursor-top";
const CURSOR_BOTTOM_ENTRY_ID: &str = "cursor-bottom";
const CURSOR_GAP_ENTRY_ID: &str = "cursor-gap";

pub(super) fn initial_sort_index(cursor: Option<&UrtOrderedCursor>) -> i64 {
    let is_top = cursor
        .and_then(|cursor| cursor.cursor_type.as_ref())
        .is_some_and(|cursor_type| *cursor_type == CursorType::TOP);

    if is_top {
        current_time_to_id()
    } else {
        cursor
            .map(|cursor| cursor.initial_sort_index)
            .unwrap_or_else(current_time_to_id)
    }
}

pub(super) fn build_cursors(
    entries: &[TimelineEntry],
    initial_sort_index: i64,
    top_post_id: Option<i64>,
    bottom_post_id: Option<i64>,
    request_cursor: Option<&UrtOrderedCursor>,
    items: &[FeedItem],
    pagination_meta: &FollowingPaginationMeta,
) -> (TimelineEntry, TimelineEntry) {
    let top_sort_index = entries
        .first()
        .map(|entry| entry.sort_index + 1)
        .unwrap_or(initial_sort_index + 1);
    let bottom_sort_index = entries
        .last()
        .map(|entry| entry.sort_index - 1)
        .unwrap_or(initial_sort_index);
    let inbound_cursor_id = request_cursor.and_then(|cursor| cursor.id.filter(|id| *id > 0));

    let top_cursor = build_cursor_entry(
        CURSOR_TOP_ENTRY_ID,
        top_sort_index,
        CursorType::TOP,
        &UrtOrderedCursor::new(
            top_sort_index + TOP_CURSOR_OFFSET,
            top_post_id.or(inbound_cursor_id),
            Some(CursorType::TOP),
            None::<i64>,
        ),
    );

    let bottom_cursor_id = pagination_meta
        .bottom_tweet_id
        .map(|id| id as i64)
        .or(bottom_post_id);
    let emit_gap = should_emit_gap_cursor(request_cursor, items, pagination_meta);

    let bottom_or_gap_cursor = if emit_gap {
        let gap_boundary_id = request_cursor
            .and_then(|cursor| cursor.gap_boundary_id.filter(|id| *id > 0))
            .or_else(|| request_cursor.and_then(|cursor| cursor.id.filter(|id| *id > 0)));
        build_cursor_entry(
            CURSOR_GAP_ENTRY_ID,
            bottom_sort_index,
            CursorType::GAP,
            &UrtOrderedCursor::new(
                bottom_sort_index - 2,
                bottom_cursor_id.or(gap_boundary_id),
                Some(CursorType::GAP),
                gap_boundary_id,
            ),
        )
    } else {
        build_cursor_entry(
            CURSOR_BOTTOM_ENTRY_ID,
            bottom_sort_index,
            CursorType::BOTTOM,
            &UrtOrderedCursor::new(
                bottom_sort_index - 2,
                bottom_cursor_id.or(inbound_cursor_id),
                Some(CursorType::BOTTOM),
                None::<i64>,
            ),
        )
    };

    (top_cursor, bottom_or_gap_cursor)
}

pub(super) fn should_emit_gap_cursor(
    cursor: Option<&UrtOrderedCursor>,
    items: &[FeedItem],
    pagination_meta: &FollowingPaginationMeta,
) -> bool {
    let is_top_or_gap = cursor
        .and_then(|cursor| cursor.cursor_type.as_ref())
        .is_some_and(|cursor_type| {
            *cursor_type == CursorType::TOP || *cursor_type == CursorType::GAP
        });
    if !is_top_or_gap {
        return false;
    }

    let has_post = items.iter().any(|item| post_id(item).is_some());
    pagination_meta.response_truncated && has_post
}

fn post_id(item: &FeedItem) -> Option<u64> {
    match &item.item {
        Some(FeedItemKind::Post(post)) if post.tweet_id > 0 => Some(post.tweet_id),
        _ => None,
    }
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
        entry_id: format!("{entry_id}-{sort_index}"),
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

#[cfg(test)]
mod tests {
    use super::*;
    use xai_home_mixer_proto::ScoredPost;
    use xai_urt_thrift::operation::CursorType;

    fn post_item(tweet_id: u64) -> FeedItem {
        FeedItem {
            position: 0,
            item: Some(FeedItemKind::Post(ScoredPost {
                tweet_id,
                ..Default::default()
            })),
        }
    }

    fn top_cursor() -> UrtOrderedCursor {
        UrtOrderedCursor::new(1, Some(1i64), Some(CursorType::TOP), None::<i64>)
    }

    fn cursor_id(entry: &TimelineEntry) -> Option<i64> {
        let TimelineEntryContent::Operation(TimelineOperation::Cursor(c)) = &entry.content else {
            panic!("expected cursor entry");
        };
        match xai_urt_thrift::cursor_from_string::<RequestCursor>(&c.value).unwrap() {
            RequestCursor::UrtOrderedCursor(ordered) => ordered.id,
            _ => None,
        }
    }

    #[test]
    fn gap_not_emitted_without_source_truncation() {
        let meta = FollowingPaginationMeta {
            response_truncated: false,
            bottom_tweet_id: Some(10),
        };
        let items = vec![post_item(30), post_item(20)];
        assert!(!should_emit_gap_cursor(Some(&top_cursor()), &items, &meta));
    }

    #[test]
    fn gap_emitted_on_source_truncation() {
        let meta = FollowingPaginationMeta {
            response_truncated: true,
            bottom_tweet_id: Some(10),
        };
        let items = vec![post_item(30), post_item(20)];
        assert!(should_emit_gap_cursor(Some(&top_cursor()), &items, &meta));
    }

    #[test]
    fn source_truncation_gap_uses_pre_groupby_bottom() {
        let meta = FollowingPaginationMeta {
            response_truncated: true,
            bottom_tweet_id: Some(10),
        };
        let items = vec![post_item(30), post_item(20)];
        let (_top, gap) = build_cursors(
            &[],
            100,
            Some(30),
            Some(20),
            Some(&top_cursor()),
            &items,
            &meta,
        );
        assert!(gap.entry_id.starts_with("cursor-gap"));
        assert_eq!(cursor_id(&gap), Some(10));
    }

    #[test]
    fn bottom_cursor_uses_pre_groupby_bottom() {
        let meta = FollowingPaginationMeta {
            response_truncated: false,
            bottom_tweet_id: Some(10),
        };
        let items = vec![post_item(30), post_item(20)];
        let (_top, bottom) = build_cursors(
            &[],
            100,
            Some(30),
            Some(20),
            Some(&top_cursor()),
            &items,
            &meta,
        );
        assert!(bottom.entry_id.starts_with("cursor-bottom"));
        assert_eq!(cursor_id(&bottom), Some(10));
    }
}
