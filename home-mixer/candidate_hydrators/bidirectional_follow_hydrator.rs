use crate::models::candidate::PostCandidate;
use crate::models::query::ScoredPostsQuery;
use crate::params::{EnableAllAuthorFollowHydration, EnableBidirectionalFollowHydration};
use std::collections::HashSet;
use std::sync::Arc;
use tonic::async_trait;
use xai_candidate_pipeline::component_library::clients::SocialGraphClientOps;
use xai_candidate_pipeline::hydrator::Hydrator;

pub struct BidirectionalFollowHydrator {
    pub socialgraph_client: Arc<dyn SocialGraphClientOps>,
}

#[async_trait]
impl Hydrator<ScoredPostsQuery, PostCandidate> for BidirectionalFollowHydrator {
    fn enable(&self, query: &ScoredPostsQuery) -> bool {
        query.params.get(EnableBidirectionalFollowHydration) && !query.has_cached_posts
    }

    async fn hydrate(
        &self,
        query: &ScoredPostsQuery,
        candidates: &[PostCandidate],
    ) -> Vec<Result<PostCandidate, String>> {
        let following: HashSet<i64> = query
            .user_features
            .followed_user_ids
            .iter()
            .copied()
            .collect();
        let check_all_authors = query.params.get(EnableAllAuthorFollowHydration);

        let authors: Vec<u64> = candidates
            .iter()
            .map(|c| c.author_id)
            .filter(|a| check_all_authors || following.contains(&(*a as i64)))
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();

        let followers = if authors.is_empty() {
            HashSet::new()
        } else {
            match self
                .socialgraph_client
                .check_followed_by(query.user_id, &authors)
                .await
            {
                Ok(m) => m,
                Err(e) => return candidates.iter().map(|_| Err(e.to_string())).collect(),
            }
        };

        candidates
            .iter()
            .map(|c| {
                let author_follows_viewer = followers.contains(&c.author_id);
                Ok(PostCandidate {
                    author_follows_viewer: check_all_authors.then_some(author_follows_viewer),
                    is_mutual_follow_author: Some(
                        author_follows_viewer && following.contains(&(c.author_id as i64)),
                    ),
                    ..Default::default()
                })
            })
            .collect()
    }

    fn update(&self, candidate: &mut PostCandidate, hydrated: PostCandidate) {
        candidate.is_mutual_follow_author = hydrated.is_mutual_follow_author;
        candidate.author_follows_viewer = hydrated.author_follows_viewer;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::user_features::UserFeatures;
    use tonic::Status;

    struct MockSocialGraph {
        followers: Vec<i64>,
    }

    #[async_trait]
    impl SocialGraphClientOps for MockSocialGraph {
        async fn get_following_list(&self, _user_id: u64) -> Result<Vec<u64>, Status> {
            Ok(vec![])
        }
        async fn check_blocked_by(
            &self,
            _viewer_id: u64,
            _author_ids: &[u64],
        ) -> Result<HashSet<u64>, Status> {
            Ok(HashSet::new())
        }
        async fn check_followed_by(
            &self,
            _viewer_id: u64,
            user_ids: &[u64],
        ) -> Result<HashSet<u64>, Status> {
            let followers: HashSet<i64> = self.followers.iter().copied().collect();
            Ok(user_ids
                .iter()
                .copied()
                .filter(|id| followers.contains(&(*id as i64)))
                .collect())
        }
        async fn get_blocked_user_ids(&self, _viewer_id: u64) -> Result<Vec<i64>, Status> {
            Ok(vec![])
        }
        async fn get_muted_user_ids(&self, _viewer_id: u64) -> Result<Vec<i64>, Status> {
            Ok(vec![])
        }
        async fn get_followed_user_ids(&self, _viewer_id: u64) -> Result<Vec<i64>, Status> {
            Ok(vec![])
        }
        async fn get_follower_ids(&self, _user_id: u64) -> Result<Vec<i64>, Status> {
            Ok(vec![])
        }
        async fn get_subscribed_user_ids(&self, _viewer_id: u64) -> Result<Vec<i64>, Status> {
            Ok(vec![])
        }
        async fn get_device_following_user_ids(&self, _viewer_id: u64) -> Result<Vec<i64>, Status> {
            Ok(vec![])
        }
        async fn get_hide_recommendations_user_ids(
            &self,
            _viewer_id: u64,
        ) -> Result<Vec<i64>, Status> {
            Ok(vec![])
        }
    }

    fn hydrator(followers: Vec<i64>) -> BidirectionalFollowHydrator {
        BidirectionalFollowHydrator {
            socialgraph_client: Arc::new(MockSocialGraph { followers }),
        }
    }

    fn query(followed: Vec<i64>, enabled: bool) -> ScoredPostsQuery {
        let mut query = ScoredPostsQuery {
            user_id: 42,
            user_features: UserFeatures {
                followed_user_ids: followed,
                ..Default::default()
            },
            ..Default::default()
        };
        let fs = xai_feature_switches::FeatureSwitches::new(vec![]).unwrap();
        let mut results =
            fs.match_recipient(&xai_feature_switches::RecipientBuilder::new().build());
        results.override_fs(
            "rust_home_mixer_enable_bidirectional_follow_hydration".to_string(),
            if enabled { "true" } else { "false" },
        );
        query.params = results.into();
        query
    }

    fn candidate(author_id: u64) -> PostCandidate {
        PostCandidate {
            author_id,
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn tags_only_mutual_follow_authors() {
        let hydrator = hydrator(vec![2, 3, 5]);
        let q = query(vec![1, 2, 3], true);
        let candidates = vec![candidate(1), candidate(2), candidate(4)];

        let out = hydrator.hydrate(&q, &candidates).await;

        assert_eq!(
            out[0].as_ref().unwrap().is_mutual_follow_author,
            Some(false)
        );
        assert_eq!(out[1].as_ref().unwrap().is_mutual_follow_author, Some(true));
        assert_eq!(
            out[2].as_ref().unwrap().is_mutual_follow_author,
            Some(false)
        );
    }

    #[tokio::test]
    async fn empty_following_tags_all_false() {
        let hydrator = hydrator(vec![1, 2, 3]);
        let q = query(vec![], true);
        let candidates = vec![candidate(1), candidate(2)];

        let out = hydrator.hydrate(&q, &candidates).await;

        assert!(out
            .iter()
            .all(|c| c.as_ref().unwrap().is_mutual_follow_author == Some(false)));
    }

    #[test]
    fn follows_feature_switch() {
        let hydrator = hydrator(vec![]);
        assert!(!hydrator.enable(&query(vec![1], false)));
        assert!(hydrator.enable(&query(vec![1], true)));
    }

    #[test]
    fn disabled_for_cached_posts() {
        let hydrator = hydrator(vec![]);
        let mut q = query(vec![1], true);
        assert!(hydrator.enable(&q));
        q.has_cached_posts = true;
        assert!(!hydrator.enable(&q));
    }
}
