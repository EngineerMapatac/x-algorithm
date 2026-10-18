use super::AdsBlender;
use super::util::{
    MIN_POSTS_FOR_ADS, is_bsr_low_ad, posts_to_feed_items, should_drop_handle, should_drop_keyword,
};
use crate::params::FOLLOWING_MAX_RESULT_SIZE;
use xai_home_mixer_proto::{BrandSafetyVerdict, FeedItem, ScoredPost, feed_item};
use xai_recsys_proto::AdIndexInfo;

const DEFAULT_SPACING: usize = 3;
const MIN_REQUESTED_GAP: usize = 3;
const SPACING_PROBE_ADS: usize = 4;

pub struct FollowingAdBlender {
    pub(crate) result_size: usize,
}

impl Default for FollowingAdBlender {
    fn default() -> Self {
        Self {
            result_size: FOLLOWING_MAX_RESULT_SIZE,
        }
    }
}

impl AdsBlender for FollowingAdBlender {
    fn blend_inner(&self, scored_posts: Vec<ScoredPost>, ads: Vec<AdIndexInfo>) -> Vec<FeedItem> {
        blend_impl(scored_posts, ads, MIN_POSTS_FOR_ADS, self.result_size)
    }
}

pub(crate) fn compute_spacing(ads: &[AdIndexInfo]) -> usize {
    if ads.len() < 2 {
        return DEFAULT_SPACING;
    }

    let mut positions: Vec<i32> = ads
        .iter()
        .take(SPACING_PROBE_ADS)
        .map(|a| a.insert_position)
        .collect();
    positions.sort_unstable();

    let min_diff = positions
        .windows(2)
        .map(|w| (w[1] - w[0]) as usize)
        .filter(|&d| d > 0)
        .min();

    match min_diff {
        Some(requested) if requested >= MIN_REQUESTED_GAP => requested,
        _ => DEFAULT_SPACING,
    }
}

pub(crate) fn compute_first_position(ads: &[AdIndexInfo], spacing: usize) -> usize {
    let first = ads
        .first()
        .map(|a| a.insert_position.max(0) as usize)
        .unwrap_or(1);
    first.clamp(1, spacing)
}

fn same_conversation_unit(a: &ScoredPost, b: &ScoredPost) -> bool {
    a.ancestors.contains(&b.tweet_id)
        || b.ancestors.contains(&a.tweet_id)
        || a.ancestors.iter().any(|id| b.ancestors.contains(id))
}

pub(crate) fn blend_units(posts: &[ScoredPost]) -> Vec<std::ops::Range<usize>> {
    if posts.is_empty() {
        return Vec::new();
    }
    let mut units = Vec::new();
    let mut start = 0;
    for i in 1..posts.len() {
        if !same_conversation_unit(&posts[i - 1], &posts[i]) {
            units.push(start..i);
            start = i;
        }
    }
    units.push(start..posts.len());
    units
}

fn is_medium_risk(post: &ScoredPost) -> bool {
    !matches!(
        post.brand_safety_verdict(),
        BrandSafetyVerdict::Safe | BrandSafetyVerdict::LowRisk
    )
}

fn is_low_risk(post: &ScoredPost) -> bool {
    post.brand_safety_verdict() == BrandSafetyVerdict::LowRisk
}

fn unit_medium_risk(unit: &[ScoredPost]) -> bool {
    unit.iter().any(is_medium_risk)
}

fn unit_low_risk(unit: &[ScoredPost]) -> bool {
    !unit_medium_risk(unit) && unit.iter().any(is_low_risk)
}

fn unit_drops_handle(ad: &AdIndexInfo, unit: &[ScoredPost]) -> bool {
    unit.iter().any(|p| should_drop_handle(ad, Some(p), None))
}

fn unit_drops_keyword(ad: &AdIndexInfo, unit: &[ScoredPost]) -> bool {
    unit.iter().any(|p| should_drop_keyword(ad, Some(p), None))
}

pub(crate) fn blend_impl(
    scored_posts: Vec<ScoredPost>,
    ads: Vec<AdIndexInfo>,
    min_posts: usize,
    result_size: usize,
) -> Vec<FeedItem> {
    let spacing = compute_spacing(&ads);
    blend_with_spacing(scored_posts, ads, min_posts, result_size, spacing)
}

pub(crate) fn blend_with_spacing(
    scored_posts: Vec<ScoredPost>,
    ads: Vec<AdIndexInfo>,
    min_posts: usize,
    result_size: usize,
    spacing: usize,
) -> Vec<FeedItem> {
    let units = blend_units(&scored_posts);
    let n_units = units.len();

    if ads.is_empty() || n_units < min_posts || spacing < 1 {
        return posts_to_feed_items(scored_posts);
    }

    let first_position = compute_first_position(&ads, spacing);
    let mut pool = ads;
    let mut ad_after: Vec<Option<AdIndexInfo>> = (0..scored_posts.len()).map(|_| None).collect();
    let mut gap = spacing - first_position;

    for (unit_idx, unit) in units.iter().enumerate() {
        gap += 1;
        if gap >= spacing && !pool.is_empty() && unit_idx + 1 < n_units {
            let above = &scored_posts[unit.start..unit.end];
            let next = &units[unit_idx + 1];
            let below = &scored_posts[next.start..next.end];
            let group_safe = !unit_medium_risk(above) && !unit_medium_risk(below);
            let group_low_risk = group_safe && (unit_low_risk(above) || unit_low_risk(below));

            if group_safe
                && let Some(placed_idx) = pool.iter().position(|ad| {
                    !(unit_drops_handle(ad, above)
                        || unit_drops_handle(ad, below)
                        || unit_drops_keyword(ad, above)
                        || unit_drops_keyword(ad, below)
                        || (group_low_risk && is_bsr_low_ad(ad)))
                })
            {
                let last = unit.end - 1;
                ad_after[last] = Some(pool.remove(placed_idx));
                gap = 0;
            }
        }
    }

    let mut items: Vec<FeedItem> = Vec::with_capacity(scored_posts.len() + ad_after.len());
    for (index, post) in scored_posts.into_iter().enumerate() {
        items.push(FeedItem {
            position: 0,
            item: Some(feed_item::Item::Post(post)),
        });
        if let Some(ad) = ad_after[index].take() {
            items.push(FeedItem {
                position: 0,
                item: Some(feed_item::Item::Ad(ad)),
            });
        }
    }

    items.truncate(result_size);
    if matches!(items.last(), Some(item) if matches!(item.item, Some(feed_item::Item::Ad(_)))) {
        items.pop();
    }
    for (i, item) in items.iter_mut().enumerate() {
        item.position = i as i32;
    }
    items
}
