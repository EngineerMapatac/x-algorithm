use crate::models::candidate::PostCandidate;
use crate::models::query::ScoredPostsQuery;
use std::collections::HashSet;
use xai_candidate_pipeline::filter::{Filter, FilterResult};

pub struct SelfReplyChainFilter;

impl Filter<ScoredPostsQuery, PostCandidate> for SelfReplyChainFilter {
    fn filter(
        &self,
        query: &ScoredPostsQuery,
        candidates: Vec<PostCandidate>,
    ) -> FilterResult<PostCandidate> {
        let mut allowed: HashSet<u64> = query
            .user_features
            .followed_user_ids
            .iter()
            .map(|&id| id as u64)
            .collect();
        allowed.insert(query.user_id);

        let (kept, removed): (Vec<_>, Vec<_>) = candidates
            .into_iter()
            .partition(|c| should_keep(c, &allowed));

        FilterResult { kept, removed }
    }
}

fn should_keep(candidate: &PostCandidate, allowed_users: &HashSet<u64>) -> bool {
    let is_reply = candidate.in_reply_to_tweet_id.is_some();
    if !is_reply || candidate.ancestor_users.is_empty() || candidate.author_id == 0 {
        return true;
    }

    let self_id = candidate.author_id;
    match candidate
        .ancestor_users
        .iter()
        .copied()
        .find(|uid| *uid != self_id)
    {
        Some(effective_directed_at) => allowed_users.contains(&effective_directed_at),
        None => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::user_features::UserFeatures;

    fn candidate(
        tweet_id: u64,
        author_id: u64,
        in_reply_to: Option<u64>,
        ancestor_users: Vec<u64>,
    ) -> PostCandidate {
        PostCandidate {
            tweet_id,
            author_id,
            in_reply_to_tweet_id: in_reply_to,
            ancestor_users,
            ..Default::default()
        }
    }

    fn query(viewer: u64, followed: Vec<i64>) -> ScoredPostsQuery {
        ScoredPostsQuery {
            user_id: viewer,
            user_features: UserFeatures {
                followed_user_ids: followed,
                ..Default::default()
            },
            ..Default::default()
        }
    }

    #[test]
    fn keeps_non_replies_and_missing_ancestor_users() {
        let filter = SelfReplyChainFilter;
        let q = query(1, vec![10]);
        let result = filter.filter(
            &q,
            vec![
                candidate(1, 10, None, vec![]),
                candidate(2, 10, Some(100), vec![]),
                candidate(3, 0, Some(100), vec![20]),
            ],
        );
        assert!(result.removed.is_empty());
        assert_eq!(result.kept.len(), 3);
    }

    #[test]
    fn keeps_pure_self_reply_chain() {
        let filter = SelfReplyChainFilter;
        let q = query(1, vec![10]);
        let result = filter.filter(&q, vec![candidate(1, 10, Some(100), vec![10, 10])]);
        assert!(result.removed.is_empty());
        assert_eq!(result.kept[0].tweet_id, 1);
    }

    #[test]
    fn keeps_when_effective_target_is_followed() {
        let filter = SelfReplyChainFilter;
        let q = query(1, vec![20]);
        let result = filter.filter(&q, vec![candidate(1, 10, Some(100), vec![10, 20])]);
        assert!(result.removed.is_empty());
    }

    #[test]
    fn keeps_when_effective_target_is_viewer() {
        let filter = SelfReplyChainFilter;
        let q = query(1, vec![]);
        let result = filter.filter(&q, vec![candidate(1, 10, Some(100), vec![10, 1])]);
        assert!(result.removed.is_empty());
    }

    #[test]
    fn drops_self_reply_directed_at_unfollowed_user() {
        let filter = SelfReplyChainFilter;
        let q = query(1, vec![10]);
        let result = filter.filter(
            &q,
            vec![
                candidate(1, 10, None, vec![]),
                candidate(2, 10, Some(100), vec![10, 99]),
            ],
        );
        assert_eq!(
            result
                .removed
                .iter()
                .map(|c| c.tweet_id)
                .collect::<Vec<_>>(),
            vec![2]
        );
        assert_eq!(
            result.kept.iter().map(|c| c.tweet_id).collect::<Vec<_>>(),
            vec![1]
        );
    }

    #[test]
    fn drops_majority_of_batch_when_rules_match() {
        let filter = SelfReplyChainFilter;
        let q = query(1, vec![]);
        let candidates = vec![
            candidate(1, 10, Some(100), vec![10, 99]),
            candidate(2, 10, Some(200), vec![10, 98]),
            candidate(3, 10, None, vec![]),
        ];
        let result = filter.filter(&q, candidates);
        assert_eq!(
            result
                .removed
                .iter()
                .map(|c| c.tweet_id)
                .collect::<Vec<_>>(),
            vec![1, 2]
        );
        assert_eq!(
            result.kept.iter().map(|c| c.tweet_id).collect::<Vec<_>>(),
            vec![3]
        );
    }
}
