use crate::ads::partition_organic_blender::*;
use crate::ads::util::should_drop_keyword;
use xai_home_mixer_proto::{feed_item, BrandSafetyVerdict, FeedItem, ScoredPost};
use xai_recsys_proto::{AdAdjacencyControl, AdIndexInfo, BrandSafetyRiskLevel};

fn make_post(tweet_id: u64) -> ScoredPost {
    ScoredPost {
        tweet_id,
        score: 1.0 - (tweet_id as f32 * 0.01),
        ..Default::default()
    }
}

fn make_low_risk_post(tweet_id: u64) -> ScoredPost {
    ScoredPost {
        tweet_id,
        brand_safety_verdict: BrandSafetyVerdict::LowRisk.into(),
        score: 1.0 - (tweet_id as f32 * 0.01),
        ..Default::default()
    }
}

fn make_avoid_post(tweet_id: u64) -> ScoredPost {
    ScoredPost {
        tweet_id,
        brand_safety_verdict: BrandSafetyVerdict::MediumRisk.into(),
        score: 1.0 - (tweet_id as f32 * 0.01),
        ..Default::default()
    }
}

fn make_normal_ad(post_id: i64) -> AdIndexInfo {
    AdIndexInfo {
        post_id,
        ad_adjacency_control: Some(AdAdjacencyControl {
            brand_safety_risk: BrandSafetyRiskLevel::BsrNormal.into(),
            ..Default::default()
        }),
        ..Default::default()
    }
}

fn make_sensitive_ad(post_id: i64) -> AdIndexInfo {
    AdIndexInfo {
        post_id,
        ad_adjacency_control: Some(AdAdjacencyControl {
            brand_safety_risk: BrandSafetyRiskLevel::BsrLow.into(),
            ..Default::default()
        }),
        ..Default::default()
    }
}

fn make_bsr_high_ad(post_id: i64) -> AdIndexInfo {
    AdIndexInfo {
        post_id,
        ad_adjacency_control: Some(AdAdjacencyControl {
            brand_safety_risk: BrandSafetyRiskLevel::BsrHigh.into(),
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

fn ad_neighbour_verdicts(items: &[FeedItem]) -> Vec<(BrandSafetyVerdict, BrandSafetyVerdict)> {
    let mut result = Vec::new();
    for (i, item) in items.iter().enumerate() {
        if matches!(item.item, Some(feed_item::Item::Ad(_))) {
            let above = if i > 0 {
                match &items[i - 1].item {
                    Some(feed_item::Item::Post(p)) => p.brand_safety_verdict(),
                    _ => BrandSafetyVerdict::VerdictUnspecified,
                }
            } else {
                BrandSafetyVerdict::VerdictUnspecified
            };
            let below = if i + 1 < items.len() {
                match &items[i + 1].item {
                    Some(feed_item::Item::Post(p)) => p.brand_safety_verdict(),
                    _ => BrandSafetyVerdict::VerdictUnspecified,
                }
            } else {
                BrandSafetyVerdict::VerdictUnspecified
            };
            result.push((above, below));
        }
    }
    result
}

fn ad_bsr_levels(items: &[FeedItem]) -> Vec<BrandSafetyRiskLevel> {
    items
        .iter()
        .filter_map(|i| match &i.item {
            Some(feed_item::Item::Ad(ad)) => Some(
                ad.ad_adjacency_control
                    .as_ref()
                    .map(|c| c.brand_safety_risk())
                    .unwrap_or(BrandSafetyRiskLevel::BsrUnknown),
            ),
            _ => None,
        })
        .collect()
}

#[test]
fn test_no_ads() {
    let posts = vec![make_post(1), make_post(2), make_post(3)];
    let result = blend_impl(posts, vec![], 5);
    assert_eq!(result.len(), 3);
    assert_eq!(ad_count(&result), 0);
}

#[test]
fn test_no_posts() {
    let result = blend_impl(vec![], vec![make_normal_ad(100)], 5);
    assert_eq!(result.len(), 0);
}

#[test]
fn test_too_few_posts() {
    let posts: Vec<_> = (1..=4).map(make_post).collect();
    let result = blend_impl(posts, vec![make_normal_ad(100)], 5);
    assert_eq!(ad_count(&result), 0);
}

#[test]
fn test_basic_blending_all_safe() {
    let posts: Vec<_> = (1..=10).map(make_post).collect();
    let ads = vec![make_normal_ad(100), make_normal_ad(200)];
    let result = blend_impl(posts, ads, 5);
    assert!(ad_count(&result) > 0);

    for (above, below) in ad_neighbour_verdicts(&result) {
        assert_ne!(above, BrandSafetyVerdict::MediumRisk);
        assert_ne!(below, BrandSafetyVerdict::MediumRisk);
    }
}

#[test]
fn test_bsr_low_ad_not_adjacent_to_low_risk_post() {
    let posts = vec![
        make_post(1),
        make_post(2),
        make_post(3),
        make_low_risk_post(4),
        make_post(5),
        make_post(6),
        make_post(7),
        make_post(8),
    ];
    let ads = vec![make_sensitive_ad(100)];
    let result = blend_impl(posts, ads, 5);

    for (above, below) in ad_neighbour_verdicts(&result) {
        assert_ne!(
            above,
            BrandSafetyVerdict::LowRisk,
            "BSR_LOW ad adjacent to LowRisk above"
        );
        assert_ne!(
            below,
            BrandSafetyVerdict::LowRisk,
            "BSR_LOW ad adjacent to LowRisk below"
        );
    }
}

#[test]
fn test_bsr_normal_ad_can_be_adjacent_to_low_risk_post() {
    let posts = vec![
        make_low_risk_post(1),
        make_low_risk_post(2),
        make_post(3),
        make_post(4),
        make_post(5),
        make_post(6),
    ];
    let ads = vec![make_normal_ad(100)];
    let result = blend_impl(posts, ads, 5);
    assert_eq!(ad_count(&result), 1);
}

#[test]
fn test_bsr_low_ad_dropped_when_group_has_low_risk() {
    let posts = vec![
        make_low_risk_post(1),
        make_post(2),
        make_post(3),
        make_post(4),
        make_post(5),
        make_post(6),
    ];
    let ads = vec![make_sensitive_ad(100)];
    let result = blend_impl(posts, ads, 5);
    assert_eq!(
        ad_count(&result),
        0,
        "BSR_LOW ad should be dropped when group has LowRisk"
    );
}

#[test]
fn test_bsr_low_ad_dropped_when_no_safe_swap() {
    let posts = vec![
        make_low_risk_post(1),
        make_low_risk_post(2),
        make_low_risk_post(3),
        make_low_risk_post(4),
        make_low_risk_post(5),
        make_low_risk_post(6),
    ];
    let ads = vec![make_sensitive_ad(100)];
    let result = blend_impl(posts, ads, 5);
    assert_eq!(
        ad_count(&result),
        0,
        "BSR_LOW ad should be dropped when no Safe posts"
    );
}

#[test]
fn test_mixed_bsr_ads() {
    let posts = vec![
        make_post(1),
        make_low_risk_post(2),
        make_post(3),
        make_post(4),
        make_post(5),
        make_post(6),
        make_post(7),
        make_post(8),
        make_post(9),
        make_post(10),
    ];
    let ads = vec![make_sensitive_ad(100), make_normal_ad(200)];
    let result = blend_impl(posts, ads, 5);
    assert!(ad_count(&result) >= 1);

    let bsr_levels = ad_bsr_levels(&result);
    let neighbours = ad_neighbour_verdicts(&result);

    for (i, bsr) in bsr_levels.iter().enumerate() {
        if *bsr == BrandSafetyRiskLevel::BsrLow {
            let (above, below) = neighbours[i];
            assert_ne!(above, BrandSafetyVerdict::LowRisk);
            assert_ne!(below, BrandSafetyVerdict::LowRisk);
        }
        let (above, below) = neighbours[i];
        assert_ne!(above, BrandSafetyVerdict::MediumRisk);
        assert_ne!(below, BrandSafetyVerdict::MediumRisk);
    }
}

#[test]
fn test_all_low_risk_posts_bsr_low_ads_dropped_normal_placed() {
    let posts: Vec<_> = (1..=8).map(make_low_risk_post).collect();
    let ads = vec![make_sensitive_ad(100), make_normal_ad(200)];
    let result = blend_impl(posts, ads, 5);

    let bsr_levels = ad_bsr_levels(&result);
    assert!(
        !bsr_levels.contains(&BrandSafetyRiskLevel::BsrLow),
        "BSR_LOW ad should not appear when all posts are LowRisk"
    );
    assert!(
        bsr_levels.contains(&BrandSafetyRiskLevel::BsrNormal),
        "BSR_NORMAL ad should still be placed"
    );
}

#[test]
fn test_no_medium_risk_posts() {
    let posts = vec![
        make_post(1),
        make_post(2),
        make_low_risk_post(3),
        make_post(4),
        make_post(5),
        make_low_risk_post(6),
        make_post(7),
        make_post(8),
    ];
    let ads = vec![make_sensitive_ad(100), make_normal_ad(200)];
    let result = blend_impl(posts, ads, 5);
    assert_eq!(ad_count(&result), 2);

    let bsr_levels = ad_bsr_levels(&result);
    let neighbours = ad_neighbour_verdicts(&result);
    for (i, bsr) in bsr_levels.iter().enumerate() {
        if *bsr == BrandSafetyRiskLevel::BsrLow {
            let (above, below) = neighbours[i];
            assert_ne!(above, BrandSafetyVerdict::LowRisk);
            assert_ne!(below, BrandSafetyVerdict::LowRisk);
        }
    }
}

#[test]
fn test_ad_never_first_or_last() {
    let posts: Vec<_> = (1..=10).map(make_post).collect();
    let ads = vec![make_sensitive_ad(100), make_normal_ad(200)];
    let result = blend_impl(posts, ads, 5);

    assert!(matches!(
        result.first().unwrap().item,
        Some(feed_item::Item::Post(_))
    ));
    assert!(matches!(
        result.last().unwrap().item,
        Some(feed_item::Item::Post(_))
    ));
}

#[test]
fn test_result_size_truncation() {
    let posts: Vec<_> = (1..=40).map(make_post).collect();
    let ads: Vec<_> = (100..=110).map(make_normal_ad).collect();
    let result = blend_impl(posts, ads, 5);

    assert!(
        result.len() <= 35,
        "Result should be truncated to RESULT_SIZE"
    );
    assert!(matches!(
        result.last().unwrap().item,
        Some(feed_item::Item::Post(_))
    ));

    for (above, below) in ad_neighbour_verdicts(&result) {
        assert_ne!(above, BrandSafetyVerdict::MediumRisk);
        assert_ne!(below, BrandSafetyVerdict::MediumRisk);
    }
}

#[test]
fn test_drop_single_sensitive_ad_both_neighbours_low_risk_no_swap() {
    let posts: Vec<_> = (1..=6).map(make_low_risk_post).collect();
    let ads = vec![make_sensitive_ad(100)];
    let result = blend_impl(posts, ads, 5);

    assert_eq!(ad_count(&result), 0);
    assert_eq!(result.len(), 6);
}

#[test]
fn test_drop_sensitive_keep_normal_all_low_risk_posts() {
    let posts: Vec<_> = (1..=10).map(make_low_risk_post).collect();
    let ads = vec![
        make_sensitive_ad(100),
        make_normal_ad(200),
        make_sensitive_ad(300),
    ];
    let result = blend_impl(posts, ads, 5);

    let bsr_levels = ad_bsr_levels(&result);
    let sensitive_count = bsr_levels
        .iter()
        .filter(|b| **b == BrandSafetyRiskLevel::BsrLow)
        .count();
    let normal_count = bsr_levels
        .iter()
        .filter(|b| **b == BrandSafetyRiskLevel::BsrNormal)
        .count();

    assert_eq!(sensitive_count, 0, "Both BSR_LOW ads should be dropped");
    assert_eq!(normal_count, 1, "BSR_NORMAL ad should be placed");
}

#[test]
fn test_drop_sensitive_ad_when_only_one_safe_post_available() {
    let mut posts = vec![make_post(1)];
    posts.extend((2..=6).map(make_low_risk_post));
    let ads = vec![make_sensitive_ad(100)];
    let result = blend_impl(posts, ads, 5);

    let neighbours = ad_neighbour_verdicts(&result);
    for (above, below) in &neighbours {
        assert_ne!(
            *above,
            BrandSafetyVerdict::LowRisk,
            "BSR_LOW adjacent to LowRisk"
        );
        assert_ne!(
            *below,
            BrandSafetyVerdict::LowRisk,
            "BSR_LOW adjacent to LowRisk"
        );
    }
}

#[test]
fn test_multiple_sensitive_ads_compete_for_safe_posts() {
    let mut posts: Vec<_> = (1..=4).map(make_post).collect();
    posts.extend((5..=12).map(make_low_risk_post));
    let ads = vec![
        make_sensitive_ad(100),
        make_sensitive_ad(200),
        make_sensitive_ad(300),
    ];
    let result = blend_impl(posts, ads, 5);

    let bsr_levels = ad_bsr_levels(&result);
    let sensitive_placed = bsr_levels
        .iter()
        .filter(|b| **b == BrandSafetyRiskLevel::BsrLow)
        .count();

    assert!(
        sensitive_placed <= 2,
        "At most 2 BSR_LOW ads should be placed with 4 Safe posts"
    );

    let neighbours = ad_neighbour_verdicts(&result);
    for (i, bsr) in bsr_levels.iter().enumerate() {
        if *bsr == BrandSafetyRiskLevel::BsrLow {
            let (above, below) = neighbours[i];
            assert_ne!(above, BrandSafetyVerdict::LowRisk);
            assert_ne!(below, BrandSafetyVerdict::LowRisk);
        }
    }
}

#[test]
fn test_sensitive_ad_dropped_normal_ad_still_placed() {
    let posts = vec![
        make_low_risk_post(1),
        make_post(2),
        make_post(3),
        make_post(4),
        make_low_risk_post(5),
        make_post(6),
        make_post(7),
        make_post(8),
        make_post(9),
        make_post(10),
    ];
    let ads = vec![make_sensitive_ad(100), make_normal_ad(200)];
    let result = blend_impl(posts, ads, 5);

    let bsr_levels = ad_bsr_levels(&result);
    assert_eq!(ad_count(&result), 1, "Only BSR_NORMAL should be placed");
    assert!(
        bsr_levels.contains(&BrandSafetyRiskLevel::BsrNormal),
        "BSR_NORMAL ad should be placed"
    );
    assert!(
        !bsr_levels.contains(&BrandSafetyRiskLevel::BsrLow),
        "BSR_LOW ad should be dropped"
    );
}

#[test]
fn test_all_ads_dropped_returns_posts_only() {
    let posts: Vec<_> = (1..=8).map(make_low_risk_post).collect();
    let ads = vec![
        make_sensitive_ad(100),
        make_sensitive_ad(200),
        make_sensitive_ad(300),
    ];
    let result = blend_impl(posts, ads, 5);

    assert_eq!(ad_count(&result), 0, "All ads should be dropped");
    assert_eq!(result.len(), 8, "All posts should be returned");

    for item in &result {
        assert!(matches!(item.item, Some(feed_item::Item::Post(_))));
    }
}

#[test]
fn test_drop_preserves_sequential_positions() {
    let posts: Vec<_> = (1..=8).map(make_low_risk_post).collect();
    let ads = vec![make_sensitive_ad(100)];
    let result = blend_impl(posts, ads, 5);

    for (i, item) in result.iter().enumerate() {
        assert_eq!(
            item.position, i as i32,
            "Position {i} should be sequential after drop"
        );
    }
}

#[test]
fn test_mixed_verdicts_sensitive_ad_swap_chain() {
    let posts = vec![
        make_post(1),
        make_low_risk_post(2),
        make_avoid_post(3),
        make_post(4),
        make_post(5),
        make_low_risk_post(6),
        make_post(7),
        make_avoid_post(8),
        make_post(9),
        make_post(10),
        make_low_risk_post(11),
        make_post(12),
        make_post(13),
        make_avoid_post(14),
        make_post(15),
    ];
    let ads = vec![
        make_sensitive_ad(100),
        make_normal_ad(200),
        make_sensitive_ad(300),
        make_normal_ad(400),
    ];
    let result = blend_impl(posts, ads, 5);

    let output: Vec<(&str, i64)> = result
        .iter()
        .map(|item| match &item.item {
            Some(feed_item::Item::Post(p)) => ("post", p.tweet_id as i64),
            Some(feed_item::Item::Ad(a)) => ("ad", a.post_id),
            _ => ("unknown", 0),
        })
        .collect();

    let expected: Vec<(&str, i64)> = vec![
        ("post", 1),
        ("ad", 200),
        ("post", 2),
        ("post", 3),
        ("post", 4),
        ("post", 7),
        ("post", 8),
        ("post", 9),
        ("post", 5),
        ("ad", 400),
        ("post", 6),
        ("post", 10),
        ("post", 11),
        ("post", 12),
        ("post", 13),
        ("post", 14),
        ("post", 15),
    ];

    assert_eq!(
        output, expected,
        "BSR_LOW ads dropped, NORMAL ads fill same group slots.\n\
         Got:      {output:?}\n\
         Expected: {expected:?}"
    );
}

#[test]
fn test_large_timeline_stress() {
    let mut posts: Vec<ScoredPost> = Vec::new();
    for i in 1..=20 {
        posts.push(make_post(i));
    }
    for i in 21..=25 {
        posts.push(make_low_risk_post(i));
    }
    for i in 26..=35 {
        posts.push(make_avoid_post(i));
    }

    let mut ads = Vec::new();
    for i in 0..4 {
        ads.push(make_sensitive_ad(100 + i));
    }
    for i in 0..4 {
        ads.push(make_normal_ad(200 + i));
    }

    let result = blend_impl(posts, ads, 5);

    assert!(result.len() <= 35);
    assert!(matches!(
        result.first().unwrap().item,
        Some(feed_item::Item::Post(_))
    ));
    assert!(matches!(
        result.last().unwrap().item,
        Some(feed_item::Item::Post(_))
    ));

    let bsr_levels = ad_bsr_levels(&result);
    let neighbours = ad_neighbour_verdicts(&result);

    for (i, bsr) in bsr_levels.iter().enumerate() {
        let (above, below) = neighbours[i];
        assert_ne!(
            above,
            BrandSafetyVerdict::MediumRisk,
            "Ad {i} MedRisk above"
        );
        assert_ne!(
            below,
            BrandSafetyVerdict::MediumRisk,
            "Ad {i} MedRisk below"
        );
        if *bsr == BrandSafetyRiskLevel::BsrLow {
            assert_ne!(
                above,
                BrandSafetyVerdict::LowRisk,
                "BSR_LOW ad {i} LowRisk above"
            );
            assert_ne!(
                below,
                BrandSafetyVerdict::LowRisk,
                "BSR_LOW ad {i} LowRisk below"
            );
        }
    }

    for (i, item) in result.iter().enumerate() {
        assert_eq!(item.position, i as i32);
    }
}

fn make_post_with_text(tweet_id: u64, text: &str) -> ScoredPost {
    ScoredPost {
        tweet_id,
        tweet_text: text.to_string(),
        score: 1.0 - (tweet_id as f32 * 0.01),
        ..Default::default()
    }
}

fn kw_drop_above(ad: &AdIndexInfo, post: &ScoredPost) -> bool {
    should_drop_keyword(ad, Some(post), None)
}

fn kw_drop_both(ad: &AdIndexInfo, above: &ScoredPost, below: &ScoredPost) -> bool {
    should_drop_keyword(ad, Some(above), Some(below))
}

fn make_ad_with_keywords(post_id: i64, keywords: &[&str]) -> AdIndexInfo {
    AdIndexInfo {
        post_id,
        ad_adjacency_control: Some(AdAdjacencyControl {
            brand_safety_risk: BrandSafetyRiskLevel::BsrNormal.into(),
            keywords: keywords.iter().map(|s| s.to_string()).collect(),
            ..Default::default()
        }),
        ..Default::default()
    }
}

#[test]
fn test_keyword_exact_word_match() {
    let ad = make_ad_with_keywords(100, &["acme"]);
    let post = make_post_with_text(1, "Acme Corp releases new product today");
    assert!(
        kw_drop_above(&ad, &post),
        "exact word 'acme' should match 'Acme'"
    );
}

#[test]
fn test_keyword_no_sub_word_match() {
    let ad = make_ad_with_keywords(100, &["ho"]);
    let post = make_post_with_text(1, "How do we feel about who does this?");
    assert!(
        !kw_drop_above(&ad, &post),
        "'ho' should NOT match inside 'How' or 'who'"
    );
}

#[test]
fn test_keyword_no_sub_word_match_pot() {
    let ad = make_ad_with_keywords(100, &["pot"]);
    let post = make_post_with_text(1, "This shows potential for growth");
    assert!(
        !kw_drop_above(&ad, &post),
        "'pot' should NOT match inside 'potential'"
    );
}

#[test]
fn test_keyword_no_sub_word_match_ass() {
    let ad = make_ad_with_keywords(100, &["ass"]);
    let post = make_post_with_text(1, "The Passiflora plant is a beautiful class of flowers");
    assert!(
        !kw_drop_above(&ad, &post),
        "'ass' should NOT match inside 'Passiflora' or 'class'"
    );
}

#[test]
fn test_keyword_no_sub_word_match_meth() {
    let ad = make_ad_with_keywords(100, &["meth"]);
    let post = make_post_with_text(1, "Something is occurring metaphysically beyond our realm");
    assert!(
        !kw_drop_above(&ad, &post),
        "'meth' should NOT match inside 'metaphysically'"
    );
}

#[test]
fn test_keyword_no_sub_word_match_cop() {
    let ad = make_ad_with_keywords(100, &["cop"]);
    let post = make_post_with_text(1, "You can't copy the link for videos anymore");
    assert!(
        !kw_drop_above(&ad, &post),
        "'cop' should NOT match inside 'copy'"
    );
}

#[test]
fn test_keyword_empty_string_never_matches() {
    let ad = make_ad_with_keywords(100, &[""]);
    let post = make_post_with_text(1, "Any content here should not be matched");
    assert!(
        !kw_drop_above(&ad, &post),
        "empty keyword '' should NEVER match"
    );
}

#[test]
fn test_keyword_all_empty_keywords_never_match() {
    let ad = make_ad_with_keywords(100, &["", "", ""]);
    let post = make_post_with_text(1, "Some random tweet text");
    assert!(
        !kw_drop_above(&ad, &post),
        "all-empty keyword list should NEVER match"
    );
}

#[test]
fn test_keyword_empty_tweet_text_never_matches() {
    let ad = make_ad_with_keywords(100, &["acme"]);
    let post = make_post_with_text(1, "");
    assert!(
        !kw_drop_above(&ad, &post),
        "empty tweet text should never match any keyword"
    );
}

#[test]
fn test_keyword_case_insensitive() {
    let ad = make_ad_with_keywords(100, &["ACME"]);
    let post = make_post_with_text(1, "acme corp launches new service");
    assert!(
        kw_drop_above(&ad, &post),
        "keyword matching should be case insensitive"
    );
}

#[test]
fn test_keyword_with_punctuation() {
    let ad = make_ad_with_keywords(100, &["beer"]);
    let post = make_post_with_text(1, "He wasn't asking for money to buy beer.");
    assert!(
        kw_drop_above(&ad, &post),
        "'beer' should match even with trailing period"
    );
}

#[test]
fn test_keyword_multi_word_phrase_match() {
    let ad = make_ad_with_keywords(100, &["san francisco"]);
    let post = make_post_with_text(1, "I visited San Francisco last week");
    assert!(
        kw_drop_above(&ad, &post),
        "'san francisco' should match consecutive words"
    );
}

#[test]
fn test_keyword_multi_word_phrase_no_match_non_consecutive() {
    let ad = make_ad_with_keywords(100, &["san francisco"]);
    let post = make_post_with_text(1, "San Diego and Francisco are different places");
    assert!(
        !kw_drop_above(&ad, &post),
        "'san francisco' should NOT match when words are not consecutive"
    );
}

#[test]
fn test_keyword_multi_word_phrase_match_names() {
    let ad = make_ad_with_keywords(100, &["jane doe"]);
    let post = make_post_with_text(1, "Jane Doe has officially announced something");
    assert!(
        kw_drop_above(&ad, &post),
        "'jane doe' should match consecutive words"
    );
}

#[test]
fn test_keyword_multi_word_no_partial() {
    let ad = make_ad_with_keywords(100, &["jane doe"]);
    let post = make_post_with_text(1, "Jane tweeted something interesting today");
    assert!(
        !kw_drop_above(&ad, &post),
        "'jane doe' should NOT match just 'jane' alone"
    );
}

#[test]
fn test_keyword_with_hashtag() {
    let ad = make_ad_with_keywords(100, &["acme"]);
    let post = make_post_with_text(1, "#acme 2024 product launch event");
    assert!(
        kw_drop_above(&ad, &post),
        "'acme' should match '#acme' in tweet"
    );
}

#[test]
fn test_keyword_checks_both_neighbours() {
    let ad = make_ad_with_keywords(100, &["bitcoin"]);
    let above = make_post_with_text(1, "Just a normal day at the park");
    let below = make_post_with_text(2, "Buy bitcoin now while it's cheap!");
    assert!(
        kw_drop_both(&ad, &above, &below),
        "should match keyword in below post"
    );
}

#[test]
fn test_keyword_no_match_returns_false() {
    let ad = make_ad_with_keywords(100, &["bitcoin"]);
    let post = make_post_with_text(1, "Beautiful sunset over the mountains");
    assert!(!kw_drop_above(&ad, &post));
}

#[test]
fn test_keyword_short_word_18_no_match_in_2018() {
    let ad = make_ad_with_keywords(100, &["18"]);
    let post = make_post_with_text(1, "In 2018 the world changed significantly");
    assert!(
        !kw_drop_above(&ad, &post),
        "'18' should NOT match inside '2018'"
    );
}

#[test]
fn test_keyword_accent_normalization() {
    let ad = make_ad_with_keywords(100, &["cafe"]);
    let post = make_post_with_text(1, "I visited a lovely café downtown");
    assert!(
        kw_drop_above(&ad, &post),
        "'cafe' should match 'café' via accent normalization"
    );
}

#[test]
fn test_keyword_multi_word_two_word_match() {
    let ad = make_ad_with_keywords(100, &["ice cream"]);
    let post = make_post_with_text(1, "We stopped for ice cream on the way home");
    assert!(
        kw_drop_above(&ad, &post),
        "'ice cream' should match consecutive 2-word phrase"
    );
}

#[test]
fn test_keyword_multi_word_three_word_match() {
    let ad = make_ad_with_keywords(100, &["new york city"]);
    let post = make_post_with_text(1, "Visiting New York City this weekend for the first time");
    assert!(
        kw_drop_above(&ad, &post),
        "'new york city' should match consecutive 3-word phrase"
    );
}

#[test]
fn test_keyword_multi_word_broken_by_intervening_word() {
    let ad = make_ad_with_keywords(100, &["ice cream"]);
    let post = make_post_with_text(1, "The ice cold cream soda was refreshing");
    assert!(
        !kw_drop_above(&ad, &post),
        "'ice cream' should NOT match when 'cold' breaks the sequence"
    );
}

#[test]
fn test_dropped_ad_does_not_lose_posts() {
    let posts = vec![
        make_low_risk_post(1),
        make_post(2),
        make_post(3),
        make_post(4),
        make_post(5),
        make_post(6),
        make_post(7),
        make_post(8),
        make_post(9),
        make_post(10),
    ];
    let ads = vec![make_sensitive_ad(100), make_normal_ad(200)];
    let num_input_posts = posts.len();
    let result = blend_impl(posts, ads, 5);

    let output_post_count = result
        .iter()
        .filter(|i| matches!(i.item, Some(feed_item::Item::Post(_))))
        .count();

    assert_eq!(
        output_post_count, num_input_posts,
        "All {num_input_posts} input posts must appear in output even when an ad is dropped (got {output_post_count})"
    );
}

#[test]
fn test_all_ads_dropped_preserves_all_posts() {
    let posts: Vec<_> = (1..=8).map(make_low_risk_post).collect();
    let num_input_posts = posts.len();
    let ads = vec![
        make_sensitive_ad(100),
        make_sensitive_ad(200),
        make_sensitive_ad(300),
    ];
    let result = blend_impl(posts, ads, 5);

    let output_post_count = result
        .iter()
        .filter(|i| matches!(i.item, Some(feed_item::Item::Post(_))))
        .count();

    assert_eq!(
        output_post_count, num_input_posts,
        "All {num_input_posts} posts must appear when all ads are dropped (got {output_post_count})"
    );
    assert_eq!(ad_count(&result), 0);
}

#[test]
fn test_dropped_ads_do_not_waste_group_slot() {
    let posts = vec![
        make_post(1),
        make_low_risk_post(2),
        make_post(3),
        make_low_risk_post(4),
        make_post(5),
        make_low_risk_post(6),
        make_post(7),
        make_low_risk_post(8),
        make_post(9),
        make_low_risk_post(10),
    ];
    let ads = vec![
        make_sensitive_ad(100),
        make_sensitive_ad(200),
        make_sensitive_ad(300),
        make_sensitive_ad(400),
        make_normal_ad(500),
        make_normal_ad(600),
    ];
    let result = blend_impl(posts, ads, 5);

    let output: Vec<(&str, i64)> = result
        .iter()
        .map(|item| match &item.item {
            Some(feed_item::Item::Post(p)) => ("post", p.tweet_id as i64),
            Some(feed_item::Item::Ad(a)) => ("ad", a.post_id),
            _ => ("unknown", 0),
        })
        .collect();

    let expected: Vec<(&str, i64)> = vec![
        ("post", 1),
        ("ad", 500),
        ("post", 2),
        ("post", 3),
        ("post", 6),
        ("post", 7),
        ("post", 4),
        ("ad", 600),
        ("post", 5),
        ("post", 8),
        ("post", 9),
        ("post", 10),
    ];

    assert_eq!(
        output, expected,
        "output sequence must match expected: dropped BSR_LOW ads should not \
         waste group slots, and post order must be preserved.\n\
         Got:      {output:?}\n\
         Expected: {expected:?}"
    );
}

fn make_ad_with_handles(post_id: i64, handles: &[i64]) -> AdIndexInfo {
    AdIndexInfo {
        post_id,
        ad_adjacency_control: Some(AdAdjacencyControl {
            brand_safety_risk: BrandSafetyRiskLevel::BsrNormal.into(),
            handles: handles.to_vec(),
            ..Default::default()
        }),
        ..Default::default()
    }
}

#[test]
fn test_keyword_and_handle_drops_do_not_waste_group_slot() {
    let mut post1 = make_post_with_text(1, "this post contains badword in the text");
    post1.author_id = 9999;
    let posts = vec![
        post1,
        make_post_with_text(2, "just a normal safe post"),
        make_post_with_text(3, "another safe post here"),
        make_post_with_text(4, "nothing controversial"),
        make_post_with_text(5, "great weather today"),
        make_post_with_text(6, "loving this new recipe"),
        make_post_with_text(7, "happy birthday friend"),
        make_post_with_text(8, "what a beautiful sunset"),
        make_post_with_text(9, "just finished a workout"),
        make_post_with_text(10, "anyone watching the game"),
    ];
    let ads = vec![
        make_ad_with_keywords(100, &["badword"]),
        make_ad_with_keywords(200, &["badword"]),
        make_ad_with_keywords(300, &["badword"]),
        make_ad_with_handles(400, &[9999]),
        make_normal_ad(500),
        make_normal_ad(600),
    ];
    let result = blend_impl(posts, ads, 5);

    let output: Vec<(&str, i64)> = result
        .iter()
        .map(|item| match &item.item {
            Some(feed_item::Item::Post(p)) => ("post", p.tweet_id as i64),
            Some(feed_item::Item::Ad(a)) => ("ad", a.post_id),
            _ => ("unknown", 0),
        })
        .collect();

    let expected: Vec<(&str, i64)> = vec![
        ("post", 1),
        ("ad", 500),
        ("post", 2),
        ("post", 3),
        ("post", 6),
        ("post", 7),
        ("post", 4),
        ("ad", 600),
        ("post", 5),
        ("post", 8),
        ("post", 9),
        ("post", 10),
    ];

    assert_eq!(
        output, expected,
        "keyword + handle dropped ads should not waste group slots.\n\
         Got:      {output:?}\n\
         Expected: {expected:?}"
    );
}

#[test]
fn stuck_outcome_no_rejections_is_no_ads() {
    let rejections = SlotRejections::default();
    assert_eq!(rejections.stuck_outcome(), "unfilled_no_ads");
}

#[test]
fn stuck_outcome_picks_dominant_cause() {
    let rejections = SlotRejections {
        bsr_low: 1,
        handle: 0,
        keyword: 3,
    };
    assert_eq!(rejections.stuck_outcome(), "unfilled_keyword");

    let rejections = SlotRejections {
        bsr_low: 5,
        handle: 2,
        keyword: 3,
    };
    assert_eq!(rejections.stuck_outcome(), "unfilled_bsr_low");

    let rejections = SlotRejections {
        bsr_low: 0,
        handle: 4,
        keyword: 1,
    };
    assert_eq!(rejections.stuck_outcome(), "unfilled_handle");
}

#[test]
fn stuck_outcome_ties_break_bsr_then_keyword() {
    let rejections = SlotRejections {
        bsr_low: 2,
        handle: 2,
        keyword: 2,
    };
    assert_eq!(rejections.stuck_outcome(), "unfilled_bsr_low");

    let rejections = SlotRejections {
        bsr_low: 0,
        handle: 2,
        keyword: 2,
    };
    assert_eq!(rejections.stuck_outcome(), "unfilled_keyword");
}

#[test]
fn serving_limitation_picks_binding_constraint() {
    assert_eq!(serving_limitation(2, 11, 9), "ads_supply");
    assert_eq!(serving_limitation(25, 11, 9), "safe_posts");
    assert_eq!(serving_limitation(25, 8, 9), "spacing");
    assert_eq!(serving_limitation(25, 11, 0), "safe_posts");
}

#[test]
fn serving_limitation_ties_break_supply_then_spacing() {
    assert_eq!(serving_limitation(5, 5, 5), "ads_supply");
    assert_eq!(serving_limitation(9, 5, 5), "spacing");
    assert_eq!(serving_limitation(5, 9, 5), "ads_supply");
}

#[test]
fn bsr_high_ad_sits_next_to_safe_not_medium() {
    let mut posts: Vec<_> = (1..=6).map(make_avoid_post).collect();
    posts.extend((7..=12).map(make_post));
    let result = blend_impl(posts, vec![make_bsr_high_ad(100)], 5);
    assert_eq!(ad_count(&result), 1);
    let (above, below) = ad_neighbour_verdicts(&result)[0];
    assert_ne!(above, BrandSafetyVerdict::MediumRisk);
    assert_ne!(below, BrandSafetyVerdict::MediumRisk);
    assert_eq!(ad_bsr_levels(&result)[0], BrandSafetyRiskLevel::BsrHigh);
}

#[test]
fn bsr_high_ad_dropped_when_only_medium_posts() {
    let posts: Vec<_> = (1..=8).map(make_avoid_post).collect();
    let result = blend_impl(posts, vec![make_bsr_high_ad(100)], 5);
    assert_eq!(ad_count(&result), 0);
}
