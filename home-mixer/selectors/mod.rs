mod blender_selector;
mod following_blender_selector;
mod passthrough_selector;
mod top_k_score_selector;

pub use blender_selector::BlenderSelector;
pub use following_blender_selector::FollowingBlenderSelector;
pub use passthrough_selector::PassthroughSelector;
pub use top_k_score_selector::TopKScoreSelector;

use xai_home_mixer_proto::{FeedItem, ScoredPost, feed_item};
use xai_recsys_proto::AdIndexInfo;

fn build_non_selected(
    dropped_posts: usize,
    mut dropped_ads: Vec<AdIndexInfo>,
    selected: &[FeedItem],
) -> Vec<FeedItem> {
    for item in selected {
        if let Some(feed_item::Item::Ad(ad)) = &item.item
            && let Some(index) = dropped_ads
                .iter()
                .position(|dropped| dropped.impression_id == ad.impression_id)
        {
            dropped_ads.remove(index);
        }
    }
    let mut non_selected = Vec::with_capacity(dropped_posts + dropped_ads.len());
    for _ in 0..dropped_posts {
        non_selected.push(FeedItem {
            position: 0,
            item: Some(feed_item::Item::Post(ScoredPost::default())),
        });
    }
    non_selected.extend(dropped_ads.into_iter().map(|ad| FeedItem {
        position: 0,
        item: Some(feed_item::Item::Ad(ad)),
    }));
    non_selected
}
