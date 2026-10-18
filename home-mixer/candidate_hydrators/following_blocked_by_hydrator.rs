use crate::models::candidate::PostCandidate;
use crate::models::query::ScoredPostsQuery;
use std::sync::Arc;
use tonic::async_trait;
use xai_candidate_pipeline::component_library::clients::SocialGraphClientOps;
use xai_candidate_pipeline::hydrator::Hydrator;

pub struct FollowingBlockedByHydrator {
    socialgraph_client: Arc<dyn SocialGraphClientOps>,
}

impl FollowingBlockedByHydrator {
    pub async fn new(socialgraph_client: Arc<dyn SocialGraphClientOps>) -> Self {
        Self { socialgraph_client }
    }
}

#[async_trait]
impl Hydrator<ScoredPostsQuery, PostCandidate> for FollowingBlockedByHydrator {
    async fn hydrate(
        &self,
        query: &ScoredPostsQuery,
        candidates: &[PostCandidate],
    ) -> Vec<Result<PostCandidate, String>> {
        let user_ids: Vec<u64> = candidates
            .iter()
            .flat_map(|c| c.quoted_user_id.into_iter().chain(c.retweeted_user_id))
            .collect();

        let blocked_by_user_ids = match self
            .socialgraph_client
            .check_blocked_by(query.user_id, &user_ids)
            .await
        {
            Ok(ids) => ids,
            Err(e) => {
                let err_msg = e.to_string();
                return candidates.iter().map(|_| Err(err_msg.clone())).collect();
            }
        };
        candidates
            .iter()
            .map(|candidate| {
                let author_blocks_viewer = candidate
                    .retweeted_user_id
                    .is_some_and(|uid| blocked_by_user_ids.contains(&uid));
                let quoted_author_blocks_viewer = candidate
                    .quoted_user_id
                    .map(|uid| blocked_by_user_ids.contains(&uid));
                Ok(PostCandidate {
                    author_blocks_viewer: Some(author_blocks_viewer),
                    quoted_author_blocks_viewer,
                    ..Default::default()
                })
            })
            .collect()
    }

    fn update(&self, candidate: &mut PostCandidate, hydrated: PostCandidate) {
        candidate.author_blocks_viewer = hydrated.author_blocks_viewer;
        if hydrated.quoted_author_blocks_viewer.is_some() {
            candidate.quoted_author_blocks_viewer = hydrated.quoted_author_blocks_viewer;
        }
    }
}
