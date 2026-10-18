use crate::models::candidate::PostCandidate;
use crate::models::in_network_reply::InNetworkReply;
use crate::models::query::ScoredPostsQuery;
use crate::params::{
    EnableFollowingRepliedUsersFacepile, FollowingRepliedUsersFacepileMaxPosts,
    FollowingRepliedUsersFacepileMinUsers,
};
use rustc_hash::FxHashSet;
use std::collections::HashMap;
use tonic::async_trait;
use xai_candidate_pipeline::hydrator::Hydrator;

const VIEWER_FOLLOWERS_THRESHOLD: i64 = 1000;
const FACEPILE_MAX_USERS: usize = 3;

pub struct FollowingRepliedUsersHydrator;

impl FollowingRepliedUsersHydrator {
    fn build_reply_author_map(
        replies: &[InNetworkReply],
        excluded_author_ids: &FxHashSet<u64>,
    ) -> HashMap<u64, Vec<u64>> {
        let mut map: HashMap<u64, Vec<u64>> = HashMap::new();
        for reply in replies {
            if excluded_author_ids.contains(&reply.author_id) {
                continue;
            }
            map.entry(reply.in_reply_to_tweet_id)
                .or_default()
                .push(reply.author_id);
        }
        for authors in map.values_mut() {
            authors.sort_unstable();
            authors.dedup();
        }
        map
    }
}

#[async_trait]
impl Hydrator<ScoredPostsQuery, PostCandidate> for FollowingRepliedUsersHydrator {
    fn enable(&self, query: &ScoredPostsQuery) -> bool {
        let has_enough_followers = query
            .user_features
            .follower_count
            .is_some_and(|c| c >= VIEWER_FOLLOWERS_THRESHOLD);

        has_enough_followers && query.params.get(EnableFollowingRepliedUsersFacepile)
    }

    async fn hydrate(
        &self,
        query: &ScoredPostsQuery,
        candidates: &[PostCandidate],
    ) -> Vec<Result<PostCandidate, String>> {
        let min_users = query
            .params
            .get(FollowingRepliedUsersFacepileMinUsers)
            .max(0) as usize;
        let max_posts = query
            .params
            .get(FollowingRepliedUsersFacepileMaxPosts)
            .max(0) as usize;

        let empty = Vec::new();
        let replies = query.in_network_replies.get().unwrap_or(&empty);

        let excluded_author_ids: FxHashSet<u64> = query
            .user_features
            .blocked_user_ids
            .iter()
            .chain(query.user_features.muted_user_ids.iter())
            .map(|&id| id as u64)
            .collect();

        let reply_author_map = Self::build_reply_author_map(replies, &excluded_author_ids);

        let mut results = Vec::with_capacity(candidates.len());
        let mut selected_count: usize = 0;

        for candidate in candidates {
            let is_root_tweet = candidate.in_reply_to_tweet_id.is_none();

            let authors: Vec<u64> = reply_author_map
                .get(&candidate.tweet_id)
                .map(|ids| {
                    ids.iter()
                        .copied()
                        .filter(|&aid| aid != candidate.author_id)
                        .collect()
                })
                .unwrap_or_default();

            let eligible = is_root_tweet && authors.len() >= min_users;
            let user_ids = if eligible && selected_count < max_posts {
                selected_count += 1;
                authors.into_iter().take(FACEPILE_MAX_USERS).collect()
            } else {
                vec![]
            };

            results.push(Ok(PostCandidate {
                following_replied_user_ids: user_ids,
                ..Default::default()
            }));
        }

        results
    }

    fn update(&self, candidate: &mut PostCandidate, hydrated: PostCandidate) {
        candidate.following_replied_user_ids = hydrated.following_replied_user_ids;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::OnceLock;

    fn make_query(follower_count: i64, replies: Vec<InNetworkReply>) -> ScoredPostsQuery {
        make_query_with_socialgraph(follower_count, replies, vec![], vec![])
    }

    fn make_query_with_socialgraph(
        follower_count: i64,
        replies: Vec<InNetworkReply>,
        blocked_user_ids: Vec<i64>,
        muted_user_ids: Vec<i64>,
    ) -> ScoredPostsQuery {
        let fs = xai_feature_switches::FeatureSwitches::new(vec![]).unwrap();
        let mut results =
            fs.match_recipient(&xai_feature_switches::RecipientBuilder::new().build());
        results.override_fs(
            "rust_home_mixer_enable_following_replied_users_facepile".to_string(),
            "true",
        );
        results.override_fs(
            "rust_home_mixer_following_replied_users_facepile_min_users".to_string(),
            "1",
        );
        results.override_fs(
            "rust_home_mixer_following_replied_users_facepile_max_posts".to_string(),
            "3",
        );
        ScoredPostsQuery {
            user_features: crate::models::user_features::UserFeatures {
                follower_count: Some(follower_count),
                blocked_user_ids,
                muted_user_ids,
                ..Default::default()
            },
            in_network_replies: OnceLock::from(replies),
            params: results.into(),
            ..Default::default()
        }
    }

    fn root_candidate(tweet_id: u64, author_id: u64) -> PostCandidate {
        PostCandidate {
            tweet_id,
            author_id,
            in_reply_to_tweet_id: None,
            ..Default::default()
        }
    }

    fn reply_candidate(tweet_id: u64, author_id: u64, reply_to: u64) -> PostCandidate {
        PostCandidate {
            tweet_id,
            author_id,
            in_reply_to_tweet_id: Some(reply_to),
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn test_basic_hydration() {
        let hydrator = FollowingRepliedUsersHydrator;
        let replies = vec![
            InNetworkReply {
                author_id: 1,
                in_reply_to_tweet_id: 10,
            },
            InNetworkReply {
                author_id: 2,
                in_reply_to_tweet_id: 10,
            },
        ];
        let query = make_query(1000, replies);
        let candidates = vec![root_candidate(10, 99), root_candidate(11, 100)];

        let results = hydrator.hydrate(&query, &candidates).await;

        let first = results[0].as_ref().unwrap();
        assert_eq!(first.following_replied_user_ids, vec![1, 2]);

        let second = results[1].as_ref().unwrap();
        assert!(second.following_replied_user_ids.is_empty());
    }

    #[tokio::test]
    async fn test_skips_reply_candidates() {
        let hydrator = FollowingRepliedUsersHydrator;
        let replies = vec![InNetworkReply {
            author_id: 1,
            in_reply_to_tweet_id: 10,
        }];
        let query = make_query(1000, replies);
        let candidates = vec![reply_candidate(10, 99, 999)];

        let results = hydrator.hydrate(&query, &candidates).await;
        let first = results[0].as_ref().unwrap();
        assert!(first.following_replied_user_ids.is_empty());
    }

    #[tokio::test]
    async fn test_excludes_root_author() {
        let hydrator = FollowingRepliedUsersHydrator;
        let replies = vec![
            InNetworkReply {
                author_id: 99,
                in_reply_to_tweet_id: 10,
            },
            InNetworkReply {
                author_id: 2,
                in_reply_to_tweet_id: 10,
            },
        ];
        let query = make_query(1000, replies);
        let candidates = vec![root_candidate(10, 99)];

        let results = hydrator.hydrate(&query, &candidates).await;
        let first = results[0].as_ref().unwrap();
        assert_eq!(first.following_replied_user_ids, vec![2]);
    }

    #[tokio::test]
    async fn test_max_posts_cap() {
        let hydrator = FollowingRepliedUsersHydrator;
        let replies = vec![
            InNetworkReply {
                author_id: 1,
                in_reply_to_tweet_id: 10,
            },
            InNetworkReply {
                author_id: 2,
                in_reply_to_tweet_id: 11,
            },
            InNetworkReply {
                author_id: 3,
                in_reply_to_tweet_id: 12,
            },
            InNetworkReply {
                author_id: 4,
                in_reply_to_tweet_id: 13,
            },
        ];
        let query = make_query(1000, replies);
        let candidates = vec![
            root_candidate(10, 100),
            root_candidate(11, 101),
            root_candidate(12, 102),
            root_candidate(13, 103),
        ];

        let results = hydrator.hydrate(&query, &candidates).await;
        assert!(!results[0]
            .as_ref()
            .unwrap()
            .following_replied_user_ids
            .is_empty());
        assert!(!results[1]
            .as_ref()
            .unwrap()
            .following_replied_user_ids
            .is_empty());
        assert!(!results[2]
            .as_ref()
            .unwrap()
            .following_replied_user_ids
            .is_empty());
        assert!(results[3]
            .as_ref()
            .unwrap()
            .following_replied_user_ids
            .is_empty());
    }

    #[tokio::test]
    async fn test_disabled_below_follower_threshold() {
        let hydrator = FollowingRepliedUsersHydrator;
        let query = make_query(999, vec![]);
        assert!(!hydrator.enable(&query));
    }

    #[tokio::test]
    async fn test_deduplicates_authors() {
        let hydrator = FollowingRepliedUsersHydrator;
        let replies = vec![
            InNetworkReply {
                author_id: 1,
                in_reply_to_tweet_id: 10,
            },
            InNetworkReply {
                author_id: 1,
                in_reply_to_tweet_id: 10,
            },
            InNetworkReply {
                author_id: 2,
                in_reply_to_tweet_id: 10,
            },
        ];
        let query = make_query(1000, replies);
        let candidates = vec![root_candidate(10, 99)];

        let results = hydrator.hydrate(&query, &candidates).await;
        let first = results[0].as_ref().unwrap();
        assert_eq!(first.following_replied_user_ids, vec![1, 2]);
    }

    #[tokio::test]
    async fn test_excludes_blocked_and_muted_repliers() {
        let hydrator = FollowingRepliedUsersHydrator;
        let replies = vec![
            InNetworkReply {
                author_id: 1,
                in_reply_to_tweet_id: 10,
            },
            InNetworkReply {
                author_id: 2,
                in_reply_to_tweet_id: 10,
            },
            InNetworkReply {
                author_id: 3,
                in_reply_to_tweet_id: 10,
            },
        ];
        let query = make_query_with_socialgraph(1000, replies, vec![1], vec![3]);
        let candidates = vec![root_candidate(10, 99)];

        let results = hydrator.hydrate(&query, &candidates).await;
        let first = results[0].as_ref().unwrap();
        assert_eq!(first.following_replied_user_ids, vec![2]);
    }

    #[tokio::test]
    async fn test_all_repliers_blocked_yields_no_facepile() {
        let hydrator = FollowingRepliedUsersHydrator;
        let replies = vec![InNetworkReply {
            author_id: 1,
            in_reply_to_tweet_id: 10,
        }];
        let query = make_query_with_socialgraph(1000, replies, vec![1], vec![]);
        let candidates = vec![root_candidate(10, 99)];

        let results = hydrator.hydrate(&query, &candidates).await;
        let first = results[0].as_ref().unwrap();
        assert!(first.following_replied_user_ids.is_empty());
    }
}
