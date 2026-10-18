use xai_candidate_pipeline::component_library::utils::client_utils::{
    ClientPlatform, RequestContext, RequestContext::*,
};
use xai_home_mixer_proto::feed_item::Item as FeedItemKind;
use xai_home_mixer_proto::FeedItem;
use xai_urt_thrift::instruction::{ClearCache, Navigation, TimelineInstruction};

const MIN_ENTRIES_TO_CLEAR_CACHE: usize = 10;

pub(super) fn build_navigation_instructions(
    request_context: &str,
    client_app_id: i32,
    items: &[FeedItem],
) -> Vec<TimelineInstruction> {
    let ctx = RequestContext::parse(request_context);
    let clear_cache = build_clear_cache(ctx, client_app_id, items);
    let navigation = build_navigation(ctx, client_app_id);
    let mut instructions = Vec::new();
    instructions.extend(clear_cache);
    instructions.extend(navigation);
    instructions
}

fn build_clear_cache(
    ctx: RequestContext,
    client_app_id: i32,
    items: &[FeedItem],
) -> Option<TimelineInstruction> {
    let enabled = if ClientPlatform::is_ios(client_app_id) {
        matches!(
            ctx,
            PullToRefresh | Launch | Navigate | ManualRefresh | ViewportAwareRefresh
        )
    } else if ClientPlatform::is_android(client_app_id) {
        matches!(ctx, PullToRefresh | Launch)
    } else {
        false
    };

    let has_push_to_home = items
        .iter()
        .any(|fi| matches!(&fi.item, Some(FeedItemKind::PushToHome(_))));

    if !enabled && !has_push_to_home {
        return None;
    }

    let posts_count = items
        .iter()
        .filter(|fi| matches!(&fi.item, Some(FeedItemKind::Post(_))))
        .count();

    if posts_count < MIN_ENTRIES_TO_CLEAR_CACHE {
        return None;
    }

    Some(TimelineInstruction::ClearCache(ClearCache {
        retain_viewport_items: Some(true),
    }))
}

fn build_navigation(ctx: RequestContext, client_app_id: i32) -> Option<TimelineInstruction> {
    let enabled = if ClientPlatform::is_ios(client_app_id) {
        matches!(ctx, Launch | Navigate | ManualRefresh)
    } else if ClientPlatform::is_android(client_app_id) {
        matches!(ctx, Launch)
    } else {
        false
    };

    if !enabled {
        return None;
    }

    Some(TimelineInstruction::Navigation(Navigation {
        start_at_top: Some(true),
        get_newer: None,
    }))
}
