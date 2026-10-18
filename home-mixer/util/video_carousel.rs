use crate::models::candidate::{CandidateHelpers, PostCandidate};
use crate::models::query::{RequestType, ScoredPostsQuery};
use crate::params::{
    EnableVideoCarousel, VIDEO_CAROUSEL_MAX_VIDEOS, VIDEO_CAROUSEL_MIN_VIDEOS,
    VideoCarouselImmersiveVqvWeight,
};
use std::collections::HashSet;

const MAX_VERTICAL_ASPECT_RATIO: f32 = 1.0;

pub(crate) fn is_enabled(query: &ScoredPostsQuery) -> bool {
    query.request_type == RequestType::ForYou
        && query.params.get(EnableVideoCarousel)
        && query.video_carousel_eligible
}

pub(crate) fn ranks_by_immersive_scores(query: &ScoredPostsQuery) -> bool {
    query.request_type == RequestType::ForYou
        && query.params.get(EnableVideoCarousel)
        && query.params.get(VideoCarouselImmersiveVqvWeight) != 0.0
}

pub(crate) fn select_videos<'a>(
    candidates: impl IntoIterator<Item = &'a PostCandidate>,
    video_open_weight: f64,
    immersive_vqv_weight: f64,
    dedupe_by_author: bool,
    semantic_id_dedupe_levels: usize,
    embedding_dedupe_threshold: Option<f64>,
) -> Vec<PostCandidate> {
    let mut videos: Vec<&PostCandidate> = candidates
        .into_iter()
        .filter(|candidate| is_carousel_video(candidate) && is_vertical(candidate))
        .collect();
    let score =
        |video: &PostCandidate| carousel_score(video, video_open_weight, immersive_vqv_weight);
    videos.sort_by(|a, b| score(b).total_cmp(&score(a)));
    let videos = take_distinct(
        videos,
        dedupe_by_author,
        semantic_id_dedupe_levels,
        embedding_dedupe_threshold,
    );
    if videos.len() < VIDEO_CAROUSEL_MIN_VIDEOS {
        return Vec::new();
    }
    videos.into_iter().cloned().collect()
}

fn take_distinct(
    ranked: Vec<&PostCandidate>,
    dedupe_by_author: bool,
    semantic_id_dedupe_levels: usize,
    embedding_dedupe_threshold: Option<f64>,
) -> Vec<&PostCandidate> {
    let mut authors = HashSet::new();
    let mut prefixes: HashSet<&[i32]> = HashSet::new();
    let mut embeddings: Vec<&[f32]> = Vec::new();
    let mut kept = Vec::with_capacity(VIDEO_CAROUSEL_MAX_VIDEOS);
    for video in ranked {
        if kept.len() == VIDEO_CAROUSEL_MAX_VIDEOS {
            break;
        }
        let author = video.get_original_author_id();
        if dedupe_by_author && authors.contains(&author) {
            continue;
        }
        let prefix = video.semantic_id_prefix(semantic_id_dedupe_levels);
        if prefix.is_some_and(|prefix| prefixes.contains(prefix)) {
            continue;
        }
        let embedding = match (embedding_dedupe_threshold, &video.video_embedding) {
            (Some(_), Some(embedding)) if video.quoted_tweet_id.is_none() => {
                Some(embedding.as_slice())
            }
            _ => None,
        };
        if let (Some(threshold), Some(embedding)) = (embedding_dedupe_threshold, embedding)
            && embeddings
                .iter()
                .any(|picked| cosine_similarity(picked, embedding) >= threshold)
        {
            continue;
        }
        authors.insert(author);
        prefixes.extend(prefix);
        embeddings.extend(embedding);
        kept.push(video);
    }
    kept
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f64 {
    let (dot, norm_a, norm_b) = a
        .iter()
        .zip(b)
        .fold((0.0, 0.0, 0.0), |(d, na, nb), (&x, &y)| {
            let (x, y) = (f64::from(x), f64::from(y));
            (d + x * y, na + x * x, nb + y * y)
        });
    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }
    dot / (norm_a.sqrt() * norm_b.sqrt())
}

pub(crate) fn take_videos_by_carousel_score(
    candidates: &mut Vec<PostCandidate>,
    count: usize,
    video_open_weight: f64,
    immersive_vqv_weight: f64,
) -> Vec<PostCandidate> {
    let (mut videos, rest): (Vec<_>, Vec<_>) = std::mem::take(candidates)
        .into_iter()
        .partition(is_carousel_video);
    let score =
        |video: &PostCandidate| carousel_score(video, video_open_weight, immersive_vqv_weight);
    videos.sort_by(|a, b| score(b).total_cmp(&score(a)));
    let overflow = videos.split_off(count.min(videos.len()));
    *candidates = rest;
    candidates.extend(overflow);
    for video in &mut videos {
        video.video_carousel_extra = true;
    }
    videos
}

pub(crate) fn is_carousel_video(candidate: &PostCandidate) -> bool {
    candidate.has_video == Some(true)
        && candidate.in_reply_to_tweet_id.is_none()
        && candidate.phoenix_scores.video_open_score.is_some()
}

fn is_vertical(candidate: &PostCandidate) -> bool {
    candidate
        .video_aspect_ratio
        .is_some_and(|ratio| ratio < MAX_VERTICAL_ASPECT_RATIO)
}

fn carousel_score(
    candidate: &PostCandidate,
    video_open_weight: f64,
    immersive_vqv_weight: f64,
) -> f64 {
    video_open_weight * video_open_score(candidate)
        + immersive_vqv_weight * immersive_vqv_score(candidate)
}

fn video_open_score(candidate: &PostCandidate) -> f64 {
    candidate.phoenix_scores.video_open_score.unwrap_or(0.0)
}

fn immersive_vqv_score(candidate: &PostCandidate) -> f64 {
    candidate
        .immersive_scores
        .as_ref()
        .and_then(|scores| scores.vqv_score)
        .unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::candidate::PhoenixScores;

    fn video(tweet_id: u64, aspect_ratio: f32, video_open: f64) -> PostCandidate {
        PostCandidate {
            tweet_id,
            has_video: Some(true),
            video_aspect_ratio: Some(aspect_ratio),
            phoenix_scores: PhoenixScores {
                video_open_score: Some(video_open),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    fn tweet_ids(candidates: &[PostCandidate]) -> Vec<u64> {
        candidates
            .iter()
            .map(|candidate| candidate.tweet_id)
            .collect()
    }

    #[test]
    fn keeps_vertical_videos_ranked_by_video_open() {
        let candidates = vec![
            video(1, 0.5625, 0.1),
            video(2, 1.7778, 0.9),
            video(3, 1.0, 0.8),
            video(4, 0.5625, 0.5),
            video(5, 0.75, 0.3),
            PostCandidate {
                in_reply_to_tweet_id: Some(9),
                ..video(6, 0.5625, 0.7)
            },
        ];

        assert_eq!(
            tweet_ids(&select_videos(&candidates, 1.0, 0.0, false, 0, None)),
            vec![4, 5, 1]
        );
    }

    fn vertical(tweet_id: u64, author_id: u64, sid: &[i32], video_open: f64) -> PostCandidate {
        PostCandidate {
            author_id,
            semantic_ids: (!sid.is_empty()).then(|| sid.to_vec()),
            ..video(tweet_id, 0.5625, video_open)
        }
    }

    #[test]
    fn author_dedupe_keeps_the_top_video_per_original_author() {
        let candidates = vec![
            vertical(1, 10, &[], 0.9),
            PostCandidate {
                retweeted_user_id: Some(10),
                ..vertical(2, 20, &[], 0.8)
            },
            vertical(3, 10, &[], 0.7),
            vertical(4, 30, &[], 0.6),
            vertical(5, 40, &[], 0.5),
        ];

        assert_eq!(
            tweet_ids(&select_videos(&candidates, 1.0, 0.0, true, 0, None)),
            vec![1, 4, 5]
        );
    }

    #[test]
    fn semantic_id_dedupe_matches_full_prefixes_only() {
        let candidates = vec![
            vertical(1, 10, &[1, 2, 3], 0.9),
            vertical(2, 20, &[1, 2, 9], 0.8),
            vertical(3, 30, &[1, 5, 3], 0.7),
            vertical(4, 40, &[], 0.6),
            vertical(5, 50, &[1], 0.5),
        ];

        assert_eq!(
            tweet_ids(&select_videos(&candidates, 1.0, 0.0, false, 2, None)),
            vec![1, 3, 4, 5]
        );
    }

    #[test]
    fn dedupe_fills_from_lower_ranks_up_to_the_max() {
        let candidates: Vec<_> = (1..=8)
            .map(|id| {
                vertical(
                    id,
                    if id <= 3 { 10 } else { id },
                    &[],
                    1.0 - id as f64 / 10.0,
                )
            })
            .collect();

        assert_eq!(
            tweet_ids(&select_videos(&candidates, 1.0, 0.0, true, 0, None)),
            vec![1, 4, 5, 6, 7]
        );
    }

    #[test]
    fn dedupe_below_the_minimum_drops_the_carousel() {
        let candidates = vec![
            vertical(1, 10, &[], 0.9),
            vertical(2, 10, &[], 0.8),
            vertical(3, 20, &[], 0.7),
        ];

        assert!(select_videos(&candidates, 1.0, 0.0, true, 0, None).is_empty());
    }

    fn with_immersive_vqv(candidate: PostCandidate, vqv: f64) -> PostCandidate {
        PostCandidate {
            immersive_scores: Some(PhoenixScores {
                vqv_score: Some(vqv),
                ..Default::default()
            }),
            ..candidate
        }
    }

    #[test]
    fn ranks_by_weighted_sum_with_missing_immersive_scores_as_zero() {
        let candidates = vec![
            with_immersive_vqv(vertical(1, 10, &[], 0.9), 0.1),
            with_immersive_vqv(vertical(2, 20, &[], 0.2), 0.9),
            vertical(3, 30, &[], 0.5),
            with_immersive_vqv(vertical(4, 40, &[], 0.3), 0.95),
        ];

        assert_eq!(
            tweet_ids(&select_videos(&candidates, 1.0, 1.0, false, 0, None)),
            vec![4, 2, 1, 3]
        );
    }

    fn with_embedding(mut candidate: PostCandidate, embedding: &[f32]) -> PostCandidate {
        candidate.video_embedding = Some(embedding.to_vec());
        candidate
    }

    #[test]
    fn embedding_dedupe_skips_near_duplicates_except_quotes() {
        let candidates = vec![
            with_embedding(vertical(1, 10, &[], 0.9), &[1.0, 0.0]),
            with_embedding(vertical(2, 20, &[], 0.8), &[0.99, 0.1]),
            with_embedding(vertical(3, 30, &[], 0.7), &[0.0, 1.0]),
            PostCandidate {
                quoted_tweet_id: Some(99),
                ..with_embedding(vertical(4, 40, &[], 0.6), &[1.0, 0.0])
            },
            vertical(5, 50, &[], 0.5),
        ];

        assert_eq!(
            tweet_ids(&select_videos(&candidates, 1.0, 0.0, false, 0, Some(0.72))),
            vec![1, 3, 4, 5]
        );
    }
}
