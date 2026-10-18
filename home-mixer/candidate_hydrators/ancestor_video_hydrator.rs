use crate::models::candidate::PostCandidate;
use crate::models::query::ScoredPostsQuery;
use std::collections::HashSet;
use std::sync::Arc;
use tonic::async_trait;
use xai_candidate_pipeline::component_library::clients::media_info_cache_client::MediaInfoCacheClient;
use xai_candidate_pipeline::hydrator::Hydrator;

pub struct AncestorVideoHydrator {
    pub media_info_cache_client: Arc<dyn MediaInfoCacheClient + Send + Sync>,
}

#[async_trait]
impl Hydrator<ScoredPostsQuery, PostCandidate> for AncestorVideoHydrator {
    fn enable(&self, query: &ScoredPostsQuery) -> bool {
        query.exclude_videos
    }

    async fn hydrate(
        &self,
        _query: &ScoredPostsQuery,
        candidates: &[PostCandidate],
    ) -> Vec<Result<PostCandidate, String>> {
        let ancestor_ids: Vec<u64> = candidates
            .iter()
            .flat_map(displayed_ancestors)
            .collect::<HashSet<u64>>()
            .into_iter()
            .collect();
        if ancestor_ids.is_empty() {
            return candidates
                .iter()
                .map(|_| Ok(PostCandidate::default()))
                .collect();
        }

        let video_ancestors: HashSet<u64> = self
            .media_info_cache_client
            .multi_get_media_info(&ancestor_ids)
            .await
            .into_iter()
            .filter_map(|(id, result)| match result {
                Ok(Some(info)) if !info.video_durations_ms.is_empty() => Some(id),
                _ => None,
            })
            .collect();

        candidates
            .iter()
            .map(|candidate| {
                Ok(PostCandidate {
                    ancestor_has_video: Some(
                        displayed_ancestors(candidate).any(|id| video_ancestors.contains(&id)),
                    ),
                    ..Default::default()
                })
            })
            .collect()
    }

    fn update(&self, candidate: &mut PostCandidate, hydrated: PostCandidate) {
        candidate.ancestor_has_video = hydrated.ancestor_has_video;
    }
}

fn displayed_ancestors(candidate: &PostCandidate) -> impl Iterator<Item = u64> {
    candidate
        .ancestors
        .iter()
        .copied()
        .filter(move |id| !candidate.tombstone_ancestor_ids.contains(id))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use xai_candidate_pipeline::component_library::clients::media_info_cache_client::MockMediaInfoCacheClient;
    use xai_x_thrift::tweet_media_info::TweetMediaInfo;

    fn video() -> Option<TweetMediaInfo> {
        Some(TweetMediaInfo::new(true, false, true, 1, vec![5000]))
    }

    fn photo() -> Option<TweetMediaInfo> {
        Some(TweetMediaInfo::new(true, true, false, 1, vec![]))
    }

    fn hydrator(media_info: HashMap<u64, Option<TweetMediaInfo>>) -> AncestorVideoHydrator {
        AncestorVideoHydrator {
            media_info_cache_client: Arc::new(MockMediaInfoCacheClient { media_info }),
        }
    }

    fn query(exclude_videos: bool) -> ScoredPostsQuery {
        ScoredPostsQuery {
            exclude_videos,
            ..Default::default()
        }
    }

    #[test]
    fn enabled_only_when_excluding_videos() {
        let hydrator = hydrator(HashMap::new());
        assert!(hydrator.enable(&query(true)));
        assert!(!hydrator.enable(&query(false)));
    }

    #[tokio::test]
    async fn flags_candidates_with_a_displayed_video_ancestor() {
        let hydrator = hydrator(HashMap::from([(10, video()), (20, photo()), (30, video())]));
        let candidates = vec![
            PostCandidate {
                tweet_id: 1,
                ancestors: vec![10],
                ..Default::default()
            },
            PostCandidate {
                tweet_id: 2,
                ancestors: vec![20, 21],
                ..Default::default()
            },
            PostCandidate {
                tweet_id: 3,
                ancestors: vec![30],
                tombstone_ancestor_ids: vec![30],
                ..Default::default()
            },
            PostCandidate {
                tweet_id: 4,
                ancestors: vec![21, 10],
                ..Default::default()
            },
            PostCandidate {
                tweet_id: 5,
                ..Default::default()
            },
        ];

        let flags: Vec<Option<bool>> = hydrator
            .hydrate(&query(true), &candidates)
            .await
            .into_iter()
            .map(|result| result.unwrap().ancestor_has_video)
            .collect();

        assert_eq!(
            flags,
            vec![
                Some(true),
                Some(false),
                Some(false),
                Some(true),
                Some(false)
            ]
        );
    }

    #[tokio::test]
    async fn no_ancestors_leaves_candidates_unflagged() {
        let hydrator = hydrator(HashMap::new());
        let candidates = vec![PostCandidate {
            tweet_id: 1,
            ..Default::default()
        }];

        let result = hydrator.hydrate(&query(true), &candidates).await;

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].as_ref().unwrap().ancestor_has_video, None);
    }
}
