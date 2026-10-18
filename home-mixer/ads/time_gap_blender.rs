use super::util::*;
use super::AdsBlender;
use crate::params::RESULT_SIZE;
use xai_home_mixer_proto::{feed_item, FeedItem, ScoredPost};
use xai_post_text::TokenSequence;
use xai_recsys_proto::AdIndexInfo;
use xai_stats_receiver::global_stats_receiver;

const ENFORCEMENT_METRIC: &str = "TimeGapAds.enforcement";

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TimeGapConfig {
    pub t_sec: f64,
    pub clamp_lo: f64,
    pub clamp_hi: f64,
    pub min_organic_gap: usize,
}

impl TimeGapConfig {
    pub(crate) fn normalized_dwell(&self, post: &ScoredPost) -> f64 {
        let t = self.t_sec;
        let lo = self.clamp_lo * t;
        let hi = self.clamp_hi * t;
        let raw = post
            .predicted_dwell_sec
            .filter(|d| d.is_finite() && *d > 0.0)
            .map(|d| d as f64)
            .unwrap_or(t);
        raw.clamp(lo, hi)
    }

    fn need_sec(&self, spacing: &AdSpacing) -> f64 {
        self.t_sec * spacing.requested as f64
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TimeGapAdsBlender {
    pub config: TimeGapConfig,
}

impl AdsBlender for TimeGapAdsBlender {
    fn blend_inner(&self, scored_posts: Vec<ScoredPost>, ads: Vec<AdIndexInfo>) -> Vec<FeedItem> {
        blend_impl(scored_posts, ads, MIN_POSTS_FOR_ADS, self.config)
    }
}

pub(crate) fn blend_impl(
    scored_posts: Vec<ScoredPost>,
    ads: Vec<AdIndexInfo>,
    min_posts: usize,
    cfg: TimeGapConfig,
) -> Vec<FeedItem> {
    let n = scored_posts.len();

    if ads.is_empty() || n < min_posts {
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
            continue;
        }
        if is_bsr_low_ad(&ad) {
            bsr_ok += 1;
        }

        if should_drop_handle(&ad, above_ref, below_ref) {
            handle_drop += 1;
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
                continue;
            }
        }

        let above = safe_opts[group_start].take().unwrap();
        let below = safe_opts[group_start + 1].take().unwrap();
        triples.push((ad, above, below));
        group_idx += 1;
        slot_tokens = None;
    }

    let placed_ads = triples.len();
    emit_enforcement_metrics(bsr_drop, bsr_ok, handle_drop, keyword_drop);

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

    let mut items: Vec<FeedItem> = Vec::with_capacity(n + placed_ads);
    assemble_time_gap(&mut items, triples, filler, &spacing, cfg);

    items.truncate(RESULT_SIZE);
    if matches!(items.last(), Some(item) if matches!(item.item, Some(feed_item::Item::Ad(_)))) {
        items.pop();
    }
    for (i, item) in items.iter_mut().enumerate() {
        item.position = i as i32;
    }

    items
}

fn push_post(items: &mut Vec<FeedItem>, post: ScoredPost) {
    items.push(FeedItem {
        position: 0,
        item: Some(feed_item::Item::Post(post)),
    });
}

fn push_ad(items: &mut Vec<FeedItem>, ad: AdIndexInfo) {
    items.push(FeedItem {
        position: 0,
        item: Some(feed_item::Item::Ad(ad)),
    });
}

fn assemble_time_gap(
    items: &mut Vec<FeedItem>,
    triples: Vec<(AdIndexInfo, ScoredPost, ScoredPost)>,
    filler: Vec<ScoredPost>,
    spacing: &AdSpacing,
    cfg: TimeGapConfig,
) {
    let need = cfg.need_sec(spacing);
    let min_gap = cfg.min_organic_gap;
    let mut filler = filler.into_iter();
    let mut triples = triples.into_iter();

    let Some((ad, above, below)) = triples.next() else {
        for post in filler {
            push_post(items, post);
        }
        return;
    };
    let mut prev_below_dwell = cfg.normalized_dwell(&below);
    push_post(items, above);
    push_ad(items, ad);
    push_post(items, below);

    while let Some((ad, above, below)) = triples.next() {
        let mut organics_between = 1usize;
        let mut filler_need = need - prev_below_dwell - cfg.normalized_dwell(&above);
        while filler_need > 0.0 || (min_gap > 0 && organics_between + 1 < min_gap) {
            let Some(post) = filler.next() else {
                push_post(items, above);
                push_post(items, below);
                for (_dropped_ad, a, b) in triples {
                    push_post(items, a);
                    push_post(items, b);
                }
                for post in filler {
                    push_post(items, post);
                }
                return;
            };
            filler_need -= cfg.normalized_dwell(&post);
            organics_between += 1;
            push_post(items, post);
        }
        prev_below_dwell = cfg.normalized_dwell(&below);
        push_post(items, above);
        push_ad(items, ad);
        push_post(items, below);
    }

    for post in filler {
        push_post(items, post);
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
