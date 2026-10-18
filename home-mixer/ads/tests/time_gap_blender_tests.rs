use crate::ads::time_gap_blender::{blend_impl, TimeGapConfig};
use xai_home_mixer_proto::{feed_item, FeedItem, ScoredPost};
use xai_recsys_proto::{AdAdjacencyControl, AdIndexInfo, BrandSafetyRiskLevel};

fn make_post(tweet_id: u64) -> ScoredPost {
    ScoredPost {
        tweet_id,
        score: 1.0 - (tweet_id as f32 * 0.01),
        ..Default::default()
    }
}

fn make_post_dwell(tweet_id: u64, dwell: f32) -> ScoredPost {
    ScoredPost {
        tweet_id,
        score: 1.0 - (tweet_id as f32 * 0.01),
        predicted_dwell_sec: Some(dwell),
        ..Default::default()
    }
}

fn make_ad_at(post_id: i64, insert_position: i32) -> AdIndexInfo {
    AdIndexInfo {
        post_id,
        insert_position,
        ad_adjacency_control: Some(AdAdjacencyControl {
            brand_safety_risk: BrandSafetyRiskLevel::BsrNormal.into(),
            ..Default::default()
        }),
        ..Default::default()
    }
}

fn ad_count(items: &[FeedItem]) -> usize {
    items
        .iter()
        .filter(|i| matches!(i.item, Some(feed_item::Item::Ad(_))))
        .count()
}

fn default_cfg() -> TimeGapConfig {
    TimeGapConfig {
        t_sec: 4.0,
        clamp_lo: 0.5,
        clamp_hi: 2.0,
        min_organic_gap: 3,
    }
}

fn min_organics_between_ads(items: &[FeedItem]) -> Option<usize> {
    let mut ad_idxs = Vec::new();
    for (i, item) in items.iter().enumerate() {
        if matches!(item.item, Some(feed_item::Item::Ad(_))) {
            ad_idxs.push(i);
        }
    }
    if ad_idxs.len() < 2 {
        return None;
    }
    ad_idxs.windows(2).map(|w| w[1] - w[0] - 1).min()
}

#[test]
fn test_time_gap_places_second_ad_when_need_met() {
    let cfg = default_cfg();
    let posts: Vec<_> = (1..=8).map(|id| make_post_dwell(id, 4.0)).collect();
    let ads = vec![make_ad_at(100, 1), make_ad_at(200, 4), make_ad_at(300, 7)];
    let result = blend_impl(posts, ads, 0, cfg);
    assert_eq!(ad_count(&result), 2);
    assert!(min_organics_between_ads(&result).unwrap_or(0) >= 3);
}

#[test]
fn test_time_gap_drops_later_ads_when_need_unmet() {
    let cfg = TimeGapConfig {
        t_sec: 4.0,
        clamp_lo: 0.2,
        clamp_hi: 2.0,
        min_organic_gap: 3,
    };
    let posts: Vec<_> = (1..=8).map(|id| make_post_dwell(id, 0.5)).collect();
    let ads = vec![make_ad_at(100, 1), make_ad_at(200, 4), make_ad_at(300, 7)];
    let result = blend_impl(posts, ads, 0, cfg);
    assert_eq!(ad_count(&result), 1);
    assert_eq!(
        result
            .iter()
            .filter(|i| matches!(i.item, Some(feed_item::Item::Post(_))))
            .count(),
        8
    );
}

#[test]
fn test_time_gap_missing_dwell_uses_t() {
    let cfg = default_cfg();
    let posts: Vec<_> = (1..=8).map(make_post).collect();
    let ads = vec![make_ad_at(100, 1), make_ad_at(200, 4), make_ad_at(300, 7)];
    let result = blend_impl(posts, ads, 0, cfg);
    assert_eq!(ad_count(&result), 2);
    assert!(min_organics_between_ads(&result).unwrap_or(0) >= 3);
}

#[test]
fn test_normalized_dwell_clamp() {
    let cfg = default_cfg();
    assert!((cfg.normalized_dwell(&make_post_dwell(1, 0.1)) - 2.0).abs() < 1e-9);
    assert!((cfg.normalized_dwell(&make_post_dwell(1, 100.0)) - 8.0).abs() < 1e-9);
    assert!((cfg.normalized_dwell(&make_post(1)) - 4.0).abs() < 1e-9);
    assert!((cfg.normalized_dwell(&make_post_dwell(1, 5.0)) - 5.0).abs() < 1e-9);
}

#[test]
fn test_min_organic_gap_forces_filler_when_time_already_met() {
    let cfg = TimeGapConfig {
        t_sec: 4.0,
        clamp_lo: 0.5,
        clamp_hi: 2.0,
        min_organic_gap: 3,
    };
    let posts: Vec<_> = (1..=10).map(|id| make_post_dwell(id, 100.0)).collect();
    let ads = vec![make_ad_at(100, 1), make_ad_at(200, 4), make_ad_at(300, 7)];
    let result = blend_impl(posts, ads, 0, cfg);
    assert!(ad_count(&result) >= 2, "ads={}", ad_count(&result));
    assert_eq!(min_organics_between_ads(&result), Some(3));
}

#[test]
fn test_min_organic_gap_zero_allows_two_organics() {
    let cfg = TimeGapConfig {
        t_sec: 4.0,
        clamp_lo: 0.5,
        clamp_hi: 2.0,
        min_organic_gap: 0,
    };
    let posts: Vec<_> = (1..=10).map(|id| make_post_dwell(id, 100.0)).collect();
    let ads = vec![make_ad_at(100, 1), make_ad_at(200, 4), make_ad_at(300, 7)];
    let result = blend_impl(posts, ads, 0, cfg);
    assert!(ad_count(&result) >= 2, "ads={}", ad_count(&result));
    assert_eq!(min_organics_between_ads(&result), Some(2));
}
