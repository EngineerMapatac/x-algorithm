use crate::hydration::fallback_cache::FallbackCaches;
use crate::hydration::sources::Sources;
use crate::hydration::{HydratedTweet, Hydration, HydrationPlan, Hydrators, Unresolved};
use crate::metrics::{RetweetSources, Rpc};
use crate::models::{
    ClientCapability, Evaluation, PostCandidate, RawCandidate, TweetId, ViewerFeatures,
};
use crate::post_evaluator::{PostBatch, PostEvaluation, PostEvaluator};
use crate::retweet::hydrate_and_evaluate;
use crate::rules::{RuleEngine, SafetyLevel};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use xai_visibility_filtering_proto as vf_pb;

pub(crate) fn raw(tweet_id: u64, request_author_id: Option<u64>) -> RawCandidate {
    RawCandidate {
        tweet_id: TweetId(tweet_id),
        request_author_id,
    }
}

pub(crate) struct Scenario<S> {
    sources: Arc<S>,
    level: SafetyLevel,
    viewer_id: Option<u64>,
    country_code: Option<String>,
    client_capability: ClientCapability,
    rpc: Rpc,
    fallback_caches: Arc<FallbackCaches>,
}

pub(crate) struct InRequestOrder {
    pub(crate) viewer_features: ViewerFeatures,
    pub(crate) candidates: Vec<PostCandidate>,
    pub(crate) safety_labels: HashMap<TweetId, Arc<vf_pb::SafetyLabelMap>>,
    pub(crate) failed_ids: HashSet<TweetId>,
    pub(crate) unresolved: HashMap<TweetId, Unresolved>,
    pub(crate) failed_nodes: Vec<Result<Hydrators, Unresolved>>,
}

pub(crate) struct Evaluated {
    pub(crate) outcomes: Vec<PostEvaluation>,
    pub(crate) retweet_sources: RetweetSources,
}

impl Evaluated {
    pub(crate) fn evaluations(self) -> Vec<Evaluation> {
        self.outcomes
            .into_iter()
            .map(|outcome| outcome.evaluation)
            .collect()
    }
}

impl<S: Sources + 'static> Scenario<S> {
    pub(crate) fn new(sources: S, level: SafetyLevel) -> Self {
        Self {
            sources: Arc::new(sources),
            level,
            viewer_id: None,
            country_code: None,
            client_capability: ClientCapability::default(),
            rpc: Rpc::FilterTweets,
            fallback_caches: Arc::default(),
        }
    }

    pub(crate) fn viewer(mut self, viewer_id: Option<u64>) -> Self {
        self.viewer_id = viewer_id;
        self
    }

    pub(crate) fn country(mut self, country_code: &str) -> Self {
        self.country_code = Some(country_code.to_owned());
        self
    }

    pub(crate) fn client(mut self, client_capability: ClientCapability) -> Self {
        self.client_capability = client_capability;
        self
    }

    pub(crate) fn rpc(mut self, rpc: Rpc) -> Self {
        self.rpc = rpc;
        self
    }

    pub(crate) fn fallback_caches(mut self, fallback_caches: FallbackCaches) -> Self {
        self.fallback_caches = Arc::new(fallback_caches);
        self
    }

    pub(crate) fn sources(&self) -> &S {
        &self.sources
    }

    pub(crate) async fn hydrate(&self, raw: &[RawCandidate]) -> InRequestOrder {
        self.hydrate_with_plan(RuleEngine::for_tests().plan(self.level), raw)
            .await
    }

    pub(crate) async fn hydrate_with_plan(
        &self,
        plan: &HydrationPlan,
        raw: &[RawCandidate],
    ) -> InRequestOrder {
        let mut request = self.request(raw);
        let is_evaluate_tweets = self.rpc == Rpc::EvaluateTweets;
        let request = request.hydration_request(is_evaluate_tweets, &self.fallback_caches);
        let hydration = plan.hydrate(&*self.sources, request).await;
        InRequestOrder::of(&hydration, raw)
    }

    pub(crate) async fn evaluate(&self, raw: &[RawCandidate]) -> Evaluated {
        self.evaluate_with(RuleEngine::for_tests(), raw).await
    }

    pub(crate) async fn evaluate_with(
        &self,
        rule_engine: RuleEngine,
        raw: &[RawCandidate],
    ) -> Evaluated {
        let sources = Arc::clone(&self.sources) as Arc<dyn Sources>;
        let caches = Arc::clone(&self.fallback_caches);
        let post_evaluator = PostEvaluator::new(sources, rule_engine, caches);
        let (hydrated, outcomes) = hydrate_and_evaluate(&post_evaluator, self.request(raw)).await;
        Evaluated {
            outcomes,
            retweet_sources: hydrated.retweet_sources(),
        }
    }

    fn request(&self, raw: &[RawCandidate]) -> PostBatch {
        PostBatch {
            viewer_id: self.viewer_id,
            country_code: self.country_code.clone(),
            client_capability: self.client_capability,
            safety_level: self.level,
            candidates: raw.to_vec(),
            rpc: self.rpc,
        }
    }
}

impl InRequestOrder {
    fn of(hydration: &Hydration, raw: &[RawCandidate]) -> Self {
        let tweets = || {
            raw.iter()
                .filter_map(|c| Some((c.tweet_id, hydration.tweet(c.tweet_id)?)))
        };
        Self {
            viewer_features: hydration.viewer().clone(),
            candidates: tweets()
                .filter_map(|(_, tweet)| tweet.candidate().cloned())
                .collect(),
            safety_labels: tweets()
                .filter_map(|(id, tweet)| Some((id, Arc::clone(tweet.safety_labels()?))))
                .collect(),
            failed_ids: tweets()
                .filter(|(_, tweet)| {
                    tweet
                        .candidate()
                        .is_some_and(|candidate| !candidate.failed.is_empty())
                })
                .map(|(id, _)| id)
                .collect(),
            unresolved: tweets()
                .filter_map(|(id, tweet)| match tweet {
                    HydratedTweet::Resolved { .. } => None,
                    HydratedTweet::Unresolved { reason, .. } => Some((id, *reason)),
                })
                .collect(),
            failed_nodes: tweets()
                .map(|(_, tweet)| match tweet {
                    HydratedTweet::Resolved { candidate, .. } => Ok(candidate.failed),
                    HydratedTweet::Unresolved { reason, .. } => Err(*reason),
                })
                .collect(),
        }
    }
}
