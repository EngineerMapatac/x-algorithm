use crate::models::candidate::PostCandidate;
use crate::models::query::ScoredPostsQuery;
use std::collections::HashSet;
use xai_candidate_pipeline::filter::{Filter, FilterResult};
use xai_home_mixer_proto::ServedType;

pub struct PopularTopicsAuthorDedupFilter;

impl Filter<ScoredPostsQuery, PostCandidate> for PopularTopicsAuthorDedupFilter {
    fn filter(
        &self,
        _query: &ScoredPostsQuery,
        candidates: Vec<PostCandidate>,
    ) -> FilterResult<PostCandidate> {
        let mut seen_authors: HashSet<u64> = HashSet::new();
        let mut kept = Vec::new();
        let mut removed = Vec::new();

        for candidate in candidates {
            if candidate.served_type != Some(ServedType::ForYouPopularTopics) {
                kept.push(candidate);
                continue;
            }
            if candidate.author_id == 0 || seen_authors.insert(candidate.author_id) {
                kept.push(candidate);
            } else {
                removed.push(candidate);
            }
        }

        FilterResult { kept, removed }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn popular(tweet_id: u64, author_id: u64) -> PostCandidate {
        PostCandidate {
            tweet_id,
            author_id,
            served_type: Some(ServedType::ForYouPopularTopics),
            ..Default::default()
        }
    }

    fn organic(tweet_id: u64, author_id: u64) -> PostCandidate {
        PostCandidate {
            tweet_id,
            author_id,
            served_type: Some(ServedType::ForYouPhoenixRetrieval),
            ..Default::default()
        }
    }

    #[test]
    fn keeps_first_popular_per_author() {
        let filter = PopularTopicsAuthorDedupFilter;
        let result = filter.filter(
            &ScoredPostsQuery::default(),
            vec![popular(1, 100), popular(2, 200), popular(3, 100)],
        );
        assert_eq!(result.kept.len(), 2);
        assert_eq!(result.kept[0].tweet_id, 1);
        assert_eq!(result.kept[1].tweet_id, 2);
        assert_eq!(result.removed.len(), 1);
        assert_eq!(result.removed[0].tweet_id, 3);
    }

    #[test]
    fn does_not_dedup_organic_candidates() {
        let filter = PopularTopicsAuthorDedupFilter;
        let result = filter.filter(
            &ScoredPostsQuery::default(),
            vec![organic(1, 100), organic(2, 100), popular(3, 100)],
        );
        assert_eq!(result.kept.len(), 3);
        assert_eq!(result.removed.len(), 0);
    }

    #[test]
    fn organic_does_not_block_popular() {
        let filter = PopularTopicsAuthorDedupFilter;
        let result = filter.filter(
            &ScoredPostsQuery::default(),
            vec![organic(1, 100), popular(2, 100), popular(3, 100)],
        );
        assert_eq!(result.kept.len(), 2);
        assert_eq!(result.kept[0].tweet_id, 1);
        assert_eq!(result.kept[1].tweet_id, 2);
        assert_eq!(result.removed.len(), 1);
        assert_eq!(result.removed[0].tweet_id, 3);
    }
}
