use crate::models::candidate::PostCandidate;
use crate::models::query::ScoredPostsQuery;
use xai_candidate_pipeline::filter::{Filter, FilterResult};
use xai_visibility_filtering::models::Action;

pub struct VFFilter;

impl Filter<ScoredPostsQuery, PostCandidate> for VFFilter {
    fn filter(
        &self,
        _query: &ScoredPostsQuery,
        candidates: Vec<PostCandidate>,
    ) -> FilterResult<PostCandidate> {
        let (removed, kept): (Vec<_>, Vec<_>) = candidates
            .into_iter()
            .partition(|c| c.visibility_action.as_ref().is_some_and(should_drop_action));

        FilterResult { kept, removed }
    }
}

pub(crate) fn should_drop_action(action: &Action) -> bool {
    match action {
        Action::Allow | Action::Interstitial | Action::Avoid | Action::Downrank => false,
        Action::Drop(_) | Action::Tombstone | Action::NotEvaluated => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use xai_visibility_filtering::models::DropReason;

    #[test]
    fn partitions_by_action() {
        let cases = [
            (None, false),
            (Some(Action::Allow), false),
            (Some(Action::Interstitial), false),
            (Some(Action::Avoid), false),
            (Some(Action::Downrank), false),
            (Some(Action::Drop(DropReason {})), true),
            (Some(Action::Tombstone), true),
            (Some(Action::NotEvaluated), true),
        ];
        for (action, dropped) in cases {
            let candidate = PostCandidate {
                visibility_action: action,
                ..Default::default()
            };
            let result = VFFilter.filter(&ScoredPostsQuery::default(), vec![candidate]);
            assert_eq!(result.removed.len(), usize::from(dropped));
            assert_eq!(result.kept.len(), usize::from(!dropped));
        }
    }
}
