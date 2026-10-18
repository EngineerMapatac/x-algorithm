use super::util::*;
use super::AdsBlender;
use crate::params::RESULT_SIZE;
use xai_home_mixer_proto::{feed_item, FeedItem, ScoredPost};
use xai_post_text::TokenSequence;
use xai_recsys_proto::AdIndexInfo;
use xai_stats_receiver::global_stats_receiver;

const ENFORCEMENT_METRIC: &str = "PartitionOrganic.enforcement";
const SLOT_OUTCOME_METRIC: &str = "PartitionOrganic.slot_outcome";
const SERVING_LIMITATION_METRIC: &str = "PartitionOrganic.serving_limitation";

pub struct PartitionOrganicAdsBlender;

impl AdsBlender for PartitionOrganicAdsBlender {
    fn blend_inner(&self, scored_posts: Vec<ScoredPost>, ads: Vec<AdIndexInfo>) -> Vec<FeedItem> {
        blend_impl(scored_posts, ads, MIN_POSTS_FOR_ADS)
    }
}

pub(crate) fn blend_impl(
    scored_posts: Vec<ScoredPost>,
    ads: Vec<AdIndexInfo>,
    min_posts: usize,
) -> Vec<FeedItem> {
    let n = scored_posts.len();

    if ads.is_empty() || n < min_posts {
        emit_serving_limitation(if ads.is_empty() {
            "no_ads"
        } else {
            "too_few_posts"
        });
        return posts_to_feed_items(scored_posts);
    }

    let spacing = compute_spacing(&ads);

    let safe_count = scored_posts.iter().filter(|p| !has_avoid(p)).count();
    let max_from_safe = safe_count / 2;
    let expected_from_spacing = n
        .saturating_sub(1)
        .checked_div(spacing.requested)
        .unwrap_or(0);
    let actual_ads = ads.len().min(expected_from_spacing).min(max_from_safe);
    emit_serving_limitation(serving_limitation(
        ads.len(),
        expected_from_spacing,
        max_from_safe,
    ));

    if actual_ads == 0 {
        return posts_to_feed_items(scored_posts);
    }

    let mut safe: Vec<ScoredPost> = Vec::new();
    let mut unsafe_posts: Vec<ScoredPost> = Vec::new();
    for post in scored_posts {
        if has_avoid(&post) {
            unsafe_posts.push(post);
        } else {
            safe.push(post);
        }
    }

    let num_safe = safe.len();
    let group_size = num_safe / actual_ads;

    let mut safe_opts: Vec<Option<ScoredPost>> = safe.into_iter().map(Some).collect();
    let mut triples: Vec<(AdIndexInfo, ScoredPost, ScoredPost)> = Vec::new();

    let mut slot_tokens: Option<[Option<TokenSequence>; 2]> = None;

    let mut bsr_drop: u64 = 0;
    let mut bsr_ok: u64 = 0;
    let mut handle_drop: u64 = 0;
    let mut keyword_drop: u64 = 0;

    let mut slot_rejections = SlotRejections::default();

    let mut group_idx = 0;

    for ad in ads {
        if group_idx >= actual_ads {
            break;
        }
        let group_start = group_idx * group_size;
        let above_ref = safe_opts[group_start].as_ref();
        let below_ref = safe_opts[group_start + 1].as_ref();

        if should_drop_bsr_low(&ad, above_ref, below_ref) {
            bsr_drop += 1;
            slot_rejections.bsr_low += 1;
            continue;
        }
        if is_bsr_low_ad(&ad) {
            bsr_ok += 1;
        }

        if should_drop_handle(&ad, above_ref, below_ref) {
            handle_drop += 1;
            slot_rejections.handle += 1;
            continue;
        }

        if let Some(keywords) = tokenize_ad_keywords(&ad) {
            let tokens = slot_tokens.get_or_insert_with(|| {
                [above_ref, below_ref].map(|post| post.map(|p| tokenize_tweet_text(&p.tweet_text)))
            });
            let matches = |t: &Option<TokenSequence>| {
                t.as_ref()
                    .is_some_and(|t| tokens_match_any_keyword(t, &keywords))
            };
            if matches(&tokens[0]) || matches(&tokens[1]) {
                keyword_drop += 1;
                slot_rejections.keyword += 1;
                continue;
            }
        }

        let above = safe_opts[group_start].take().unwrap();
        let below = safe_opts[group_start + 1].take().unwrap();
        triples.push((ad, above, below));
        group_idx += 1;
        slot_tokens = None;
        slot_rejections = SlotRejections::default();
    }

    let placed_ads = triples.len();
    emit_enforcement_metrics(bsr_drop, bsr_ok, handle_drop, keyword_drop);
    emit_slot_outcome_metrics(
        placed_ads as u64,
        (actual_ads - placed_ads) as u64,
        &slot_rejections,
    );

    if placed_ads == 0 {
        let mut all_posts: Vec<ScoredPost> = safe_opts.into_iter().flatten().collect();
        all_posts.extend(unsafe_posts);
        all_posts.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        return posts_to_feed_items(all_posts);
    }

    let mut filler: Vec<ScoredPost> =
        Vec::with_capacity(num_safe - 2 * placed_ads + unsafe_posts.len());
    filler.extend(safe_opts.into_iter().flatten());
    filler.extend(unsafe_posts);
    filler.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let inter_ad_gaps = placed_ads;
    let filler_per_gap = filler.len() / inter_ad_gaps;
    let remainder = filler.len() % inter_ad_gaps;
    let mut filler_iter = filler.into_iter();

    let mut items: Vec<FeedItem> = Vec::with_capacity(n + placed_ads);

    for (i, (ad, above, below)) in triples.into_iter().enumerate() {
        items.push(FeedItem {
            position: 0,
            item: Some(feed_item::Item::Post(above)),
        });
        items.push(FeedItem {
            position: 0,
            item: Some(feed_item::Item::Ad(ad)),
        });
        items.push(FeedItem {
            position: 0,
            item: Some(feed_item::Item::Post(below)),
        });

        let count = filler_per_gap + if i >= inter_ad_gaps - remainder { 1 } else { 0 };
        for _ in 0..count {
            if let Some(post) = filler_iter.next() {
                items.push(FeedItem {
                    position: 0,
                    item: Some(feed_item::Item::Post(post)),
                });
            }
        }
    }

    items.truncate(RESULT_SIZE);
    if matches!(items.last(), Some(item) if matches!(item.item, Some(feed_item::Item::Ad(_)))) {
        items.pop();
    }
    for (i, item) in items.iter_mut().enumerate() {
        item.position = i as i32;
    }

    items
}

pub(crate) fn serving_limitation(
    ads_supply: usize,
    from_spacing: usize,
    from_safe: usize,
) -> &'static str {
    let budget = ads_supply.min(from_spacing).min(from_safe);
    if budget == ads_supply {
        "ads_supply"
    } else if budget == from_spacing {
        "spacing"
    } else {
        "safe_posts"
    }
}

fn emit_serving_limitation(factor: &'static str) {
    let Some(receiver) = global_stats_receiver() else {
        return;
    };
    receiver.incr(SERVING_LIMITATION_METRIC, &[("factor", factor)], 1);
}

#[derive(Default)]
pub(crate) struct SlotRejections {
    pub(crate) bsr_low: u64,
    pub(crate) handle: u64,
    pub(crate) keyword: u64,
}

impl SlotRejections {
    pub(crate) fn stuck_outcome(&self) -> &'static str {
        if self.bsr_low == 0 && self.handle == 0 && self.keyword == 0 {
            "unfilled_no_ads"
        } else if self.bsr_low >= self.keyword && self.bsr_low >= self.handle {
            "unfilled_bsr_low"
        } else if self.keyword >= self.handle {
            "unfilled_keyword"
        } else {
            "unfilled_handle"
        }
    }
}

fn emit_slot_outcome_metrics(filled: u64, unfilled: u64, stuck: &SlotRejections) {
    let Some(receiver) = global_stats_receiver() else {
        return;
    };
    if filled > 0 {
        receiver.incr(SLOT_OUTCOME_METRIC, &[("outcome", "filled")], filled);
    }
    if unfilled == 0 {
        return;
    }
    receiver.incr(
        SLOT_OUTCOME_METRIC,
        &[("outcome", stuck.stuck_outcome())],
        1,
    );
    if unfilled > 1 {
        receiver.incr(
            SLOT_OUTCOME_METRIC,
            &[("outcome", "unfilled_no_ads")],
            unfilled - 1,
        );
    }
}

fn emit_enforcement_metrics(bsr_drop: u64, bsr_ok: u64, handle_drop: u64, keyword_drop: u64) {
    let Some(receiver) = global_stats_receiver() else {
        return;
    };
    if bsr_drop > 0 {
        receiver.incr(ENFORCEMENT_METRIC, &[("action", "drop")], bsr_drop);
    }
    if bsr_ok > 0 {
        receiver.incr(ENFORCEMENT_METRIC, &[("action", "ok")], bsr_ok);
    }
    if handle_drop > 0 {
        receiver.incr(
            ENFORCEMENT_METRIC,
            &[("action", "handle_drop")],
            handle_drop,
        );
    }
    if keyword_drop > 0 {
        receiver.incr(
            ENFORCEMENT_METRIC,
            &[("action", "keyword_drop")],
            keyword_drop,
        );
    }
}
