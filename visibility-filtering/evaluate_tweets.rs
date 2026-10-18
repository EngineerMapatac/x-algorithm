use crate::caller_identity::Endpoint;
use crate::filter_tweets::normalize_viewer_id;
use crate::metrics::{self as ft_metrics, Rpc};
use crate::models::{Evaluation, RawCandidate, TweetId};
use crate::params::limited_actions_copy::LimitedActionsCopy;
use crate::params::{ClientSwitches, LimitedActionsPolicies};
use crate::post_evaluator::{PostBatch, PostEvaluator};
use crate::result_label::{label, state_label, Class, Label, NONE};
use crate::retweet;
use crate::rules::SafetyLevel;
use crate::treatment;
use std::collections::HashMap;
use std::sync::Arc;
use tonic::{Request, Response, Status};
use vf_pb::tweet_evaluation::Outcome;
use xai_twittercontext_proto::TwitterContextViewer;
use xai_visibility_filtering_proto as vf_pb;
use xai_x_thrift::safety_level::SafetyLevel as ThriftLevel;
use xai_x_thrift::tweet_service::TweetFieldsResultState;

const BATCH_SIZE: &str = "evaluate_tweets_batch_size";

pub struct EvaluateTweetsEndpoint {
    post_evaluator: Arc<PostEvaluator>,
    client_switches: ClientSwitches,
    limited_actions_copy: LimitedActionsCopy,
}

impl EvaluateTweetsEndpoint {
    pub(crate) fn new(
        post_evaluator: Arc<PostEvaluator>,
        client_switches: ClientSwitches,
        limited_actions_copy: LimitedActionsCopy,
    ) -> Self {
        Self {
            post_evaluator,
            client_switches,
            limited_actions_copy,
        }
    }

    pub async fn handle(
        &self,
        request: Request<vf_pb::EvaluateTweetsRequest>,
    ) -> Result<Response<vf_pb::EvaluateTweetsResponse>, Status> {
        let entered = tokio::time::Instant::now();
        ft_metrics::observe_request(
            Endpoint::EvaluateTweets,
            request,
            |request, grpc_timeout| async move {
                let context = crate::hydration::request_context(entered, grpc_timeout);
                let twitter_context =
                    xai_twittercontext::extract_twitter_context(request.metadata());
                context
                    .scope(self.handle_inner(request.into_inner(), twitter_context))
                    .await
                    .map(Response::new)
            },
        )
        .await
    }

    async fn handle_inner(
        &self,
        req: vf_pb::EvaluateTweetsRequest,
        twitter_context: Option<TwitterContextViewer>,
    ) -> Result<vf_pb::EvaluateTweetsResponse, Status> {
        let level = ThriftLevel(req.safety_level);
        if !ThriftLevel::ENUM_VALUES.contains(&level) {
            return Err(Status::invalid_argument("unknown safety level"));
        }
        let Some(safety_level) = SafetyLevel::from_thrift(level) else {
            return Err(Status::unimplemented("safety level has no Rust policy"));
        };
        ft_metrics::record_batch_size(BATCH_SIZE, req.tweets.len());
        let candidates = req
            .tweets
            .iter()
            .filter(|o| o.quote_context.is_none())
            .map(|o| RawCandidate {
                tweet_id: TweetId(o.tweet_id),
                request_author_id: None,
            })
            .collect();
        let viewer_id = normalize_viewer_id(req.viewer_id);
        let request = evaluate_tweets_request(
            &self.client_switches,
            twitter_context.as_ref(),
            viewer_id,
            safety_level,
            candidates,
        );
        let country_code = request.country_code.clone();
        let outcomes = retweet::evaluate_merging_sources(&self.post_evaluator, request).await;
        ft_metrics::record_verdicts(
            Rpc::EvaluateTweets,
            safety_level,
            outcomes.iter().map(|outcome| outcome.evaluation.verdict()),
        );
        ft_metrics::record_unresolved(
            Rpc::EvaluateTweets,
            safety_level,
            outcomes.iter().map(|outcome| &outcome.evaluation),
        );
        ft_metrics::record_fail_open_defaults(
            Rpc::EvaluateTweets,
            safety_level,
            outcomes
                .iter()
                .map(|outcome| outcome.evaluation.fail_open_defaults()),
        );
        let policies = self.client_switches.limited_actions_policies(
            twitter_context.as_ref(),
            viewer_id,
            country_code.as_deref(),
            outcomes
                .iter()
                .flat_map(|outcome| outcome.evaluation.verdict().limited_engagement_reasons()),
            &self.limited_actions_copy,
            xai_stats_receiver::global_stats_receiver().as_deref(),
        );
        let evaluations: HashMap<TweetId, Evaluation> = outcomes
            .into_iter()
            .map(|outcome| (outcome.tweet_id, outcome.evaluation))
            .collect();
        let results = req
            .tweets
            .into_iter()
            .map(|tweet| {
                let outcome = if tweet.quote_context.is_some() {
                    Outcome::NotEvaluated(vf_pb::NotEvaluated {})
                } else {
                    match evaluations.get(&TweetId(tweet.tweet_id)) {
                        Some(evaluation) => {
                            evaluated_outcome(Sends::new(evaluation, safety_level, &policies))
                        }
                        None => Outcome::Failed(vf_pb::Failed {}),
                    }
                };
                vf_pb::TweetEvaluation {
                    outcome: Some(outcome),
                }
            })
            .collect();
        Ok(vf_pb::EvaluateTweetsResponse { results })
    }
}

pub(crate) fn evaluate_tweets_request(
    client_switches: &ClientSwitches,
    context: Option<&TwitterContextViewer>,
    viewer_id: Option<u64>,
    safety_level: SafetyLevel,
    candidates: Vec<RawCandidate>,
) -> PostBatch {
    let country_code = context
        .map(|context| context.request_country_code.as_str())
        .filter(|code| !code.is_empty());
    PostBatch {
        viewer_id,
        country_code: country_code.map(str::to_owned),
        client_capability: client_switches.resolve(context, viewer_id, country_code),
        safety_level,
        candidates,
        rpc: Rpc::EvaluateTweets,
    }
}

#[derive(Clone, Copy)]
pub(crate) struct Sends<'a> {
    evaluation: &'a Evaluation,
    level: SafetyLevel,
    policies: &'a LimitedActionsPolicies,
}

impl<'a> Sends<'a> {
    pub(crate) fn new(
        evaluation: &'a Evaluation,
        level: SafetyLevel,
        policies: &'a LimitedActionsPolicies,
    ) -> Self {
        Self {
            evaluation,
            level,
            policies,
        }
    }

    pub(crate) fn result_state(self) -> Option<TweetFieldsResultState> {
        match self.evaluation {
            Evaluation::Complete { verdict, .. } => Some(treatment::thrift_result_state(
                verdict,
                self.level,
                self.policies,
            )),
            Evaluation::Partial { .. } | Evaluation::NotFound(_) | Evaluation::Failed(_) => None,
        }
    }

    pub(crate) fn label(self) -> Label {
        match self.evaluation {
            Evaluation::Complete { verdict, .. } => state_label(&treatment::thrift_result_state(
                verdict,
                self.level,
                self.policies,
            )),
            Evaluation::NotFound(_) => label(Class::NotFound, NONE),
            Evaluation::Partial { .. } | Evaluation::Failed(_) => label(Class::Failed, NONE),
        }
    }
}

fn evaluated_outcome(sends: Sends<'_>) -> Outcome {
    match sends
        .result_state()
        .map(|state| xai_x_thrift::serialize_compact(&state))
    {
        Some(Ok(bytes)) => Outcome::ResultStateThriftCompact(bytes.into()),
        Some(Err(_)) | None => Outcome::Failed(vf_pb::Failed {}),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hydration::inventory::Source;
    use crate::hydration::sources::{Fault, InMemorySources};
    use crate::models::{TweetFeatures, VerifyBlurSupport};
    use crate::rules::RuleEngine;
    use xai_core_entities::entities::{
        GizmoduckUser, GizmoduckUserResult, PureCoreData, Safety, UserResponseState,
    };
    use xai_x_thrift::action::{self, Action, DropReason};
    use xai_x_thrift::safety_result::{FilteredReason as ThriftFilteredReason, SafetyResult};
    use xai_x_thrift::tweet_service::{
        TweetFieldsResultFiltered, TweetFieldsResultFound, TweetFieldsResultState,
    };

    fn encoded(state: &TweetFieldsResultState) -> Outcome {
        Outcome::ResultStateThriftCompact(xai_x_thrift::serialize_compact(state).unwrap().into())
    }

    fn filtered(reason: ThriftFilteredReason) -> Outcome {
        encoded(&TweetFieldsResultState::Filtered(
            TweetFieldsResultFiltered::new(reason),
        ))
    }

    #[tokio::test]
    async fn evaluate_tweets_gates_levels_maps_outcomes_and_merges_retweet_sources() {
        let core = |author_id, source_tweet_id| PureCoreData {
            author_id,
            source_tweet_id,
            ..Default::default()
        };
        let sources = Arc::new(
            InMemorySources::default()
                .tweet_features(1, TweetFeatures::default())
                .fail_key(Source::TesPureCoreData, 1)
                .pure_core(3, core(30, None))
                .pure_core(4, core(40, Some(6)))
                .pure_core(5, core(50, Some(6)))
                .pure_core(6, core(60, None))
                .pure_core(7, core(70, Some(8)))
                .authors(&[30, 40, 50, 70])
                .user(
                    60,
                    GizmoduckUserResult {
                        user: Some(GizmoduckUser {
                            safety: Safety {
                                suspended: true,
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                        response_state: Some(UserResponseState::Found),
                    },
                ),
        );
        let endpoint = EvaluateTweetsEndpoint::new(
            Arc::new(PostEvaluator::new(
                Arc::<InMemorySources>::clone(&sources),
                RuleEngine::for_tests(),
                Arc::default(),
            )),
            ClientSwitches::for_tests(),
            LimitedActionsCopy::from_json("[]"),
        );
        for (level, code) in [
            (0, tonic::Code::Unimplemented),
            (4, tonic::Code::Unimplemented),
            (9999, tonic::Code::InvalidArgument),
        ] {
            let error = endpoint
                .handle(Request::new(vf_pb::EvaluateTweetsRequest {
                    safety_level: level,
                    ..Default::default()
                }))
                .await
                .unwrap_err();
            assert_eq!(error.code(), code);
        }
        let tweet = |tweet_id, outer_tweet_id: Option<u64>| vf_pb::TweetData {
            tweet_id,
            quote_context: outer_tweet_id.map(|outer_tweet_id| vf_pb::QuoteContext {
                outer_tweet_id,
                outer_author_id: None,
            }),
        };
        let tweets = vec![
            tweet(1, None),
            tweet(1, Some(2)),
            tweet(3, None),
            tweet(3, None),
        ];
        for (level, evaluated) in [
            (
                ThriftLevel::FILTER_ALL.0,
                filtered(ThriftFilteredReason::SafetyResult(SafetyResult::new(
                    None,
                    Action::Drop(action::Drop::new(Some(DropReason::Unspecified(true)), None)),
                ))),
            ),
            (
                ThriftLevel::TIMELINE_HOME_HYDRATION.0,
                encoded(&TweetFieldsResultState::Found(TweetFieldsResultFound::new(
                    None,
                ))),
            ),
        ] {
            let response = endpoint
                .handle(Request::new(vf_pb::EvaluateTweetsRequest {
                    safety_level: level,
                    tweets: tweets.clone(),
                    ..Default::default()
                }))
                .await
                .unwrap()
                .into_inner();
            assert_eq!(
                response
                    .results
                    .into_iter()
                    .map(|r| r.outcome.unwrap())
                    .collect::<Vec<_>>(),
                vec![
                    Outcome::Failed(vf_pb::Failed {}),
                    Outcome::NotEvaluated(vf_pb::NotEvaluated {}),
                    evaluated.clone(),
                    evaluated,
                ]
            );
        }
        let suspended = filtered(ThriftFilteredReason::AuthorIsSuspended(true));
        for (tweet_ids, core_data_calls, outcomes) in [
            (vec![4, 6], 1, vec![suspended.clone(), suspended.clone()]),
            (
                vec![5, 7],
                2,
                vec![suspended, Outcome::Failed(vf_pb::Failed {})],
            ),
        ] {
            let calls_before = sources.keys(Source::TesPureCoreData).len();
            let response = endpoint
                .handle(Request::new(vf_pb::EvaluateTweetsRequest {
                    safety_level: 8,
                    tweets: tweet_ids.into_iter().map(|id| tweet(id, None)).collect(),
                    ..Default::default()
                }))
                .await
                .unwrap()
                .into_inner();
            assert_eq!(
                sources.keys(Source::TesPureCoreData).len() - calls_before,
                core_data_calls
            );
            assert_eq!(
                response
                    .results
                    .into_iter()
                    .map(|r| r.outcome.unwrap())
                    .collect::<Vec<_>>(),
                outcomes
            );
        }
    }

    #[tokio::test]
    async fn an_author_removed_community_post_and_its_retweet_drop_or_fail_with_the_lookup() {
        use crate::models::CommunityModeration;
        use std::num::NonZeroU64;
        let dropped = filtered(ThriftFilteredReason::SafetyResult(SafetyResult::new(
            None,
            Action::Drop(action::Drop::new(None, None)),
        )));
        for (fault, outcome) in [(None, dropped), (Some(Fault::Fails), failed())] {
            let mut world = InMemorySources::default()
                .pure_core(
                    4,
                    PureCoreData {
                        author_id: 40,
                        source_tweet_id: Some(6),
                        ..Default::default()
                    },
                )
                .tweet(6, 60)
                .tweet_features(
                    6,
                    TweetFeatures {
                        community_id: NonZeroU64::new(500),
                        ..Default::default()
                    },
                )
                .community_moderation(
                    6,
                    CommunityModeration {
                        is_hidden: false,
                        is_author_removed: true,
                    },
                )
                .authors(&[40, 60]);
            if let Some(fault) = fault {
                world = world.fault(Source::CommunityModeration, fault);
            }
            let endpoint = EvaluateTweetsEndpoint::new(
                Arc::new(PostEvaluator::new(
                    Arc::new(world),
                    RuleEngine::for_tests(),
                    Arc::default(),
                )),
                ClientSwitches::for_tests(),
                LimitedActionsCopy::from_json("[]"),
            );
            let response = endpoint
                .handle(Request::new(vf_pb::EvaluateTweetsRequest {
                    safety_level: ThriftLevel::TIMELINE_HOME_HYDRATION.0,
                    viewer_id: Some(50),
                    tweets: [4, 6]
                        .map(|tweet_id| vf_pb::TweetData {
                            tweet_id,
                            quote_context: None,
                        })
                        .to_vec(),
                }))
                .await
                .unwrap()
                .into_inner();
            assert_eq!(
                response
                    .results
                    .into_iter()
                    .map(|r| r.outcome.unwrap())
                    .collect::<Vec<_>>(),
                [outcome.clone(), outcome],
                "{fault:?}"
            );
        }
    }

    const VIEWER: u64 = 50;

    async fn evaluate_tweets(
        world: InMemorySources,
        level: ThriftLevel,
        tweet_ids: &[u64],
    ) -> Vec<Outcome> {
        let endpoint = EvaluateTweetsEndpoint::new(
            Arc::new(PostEvaluator::new(
                Arc::new(world),
                RuleEngine::for_tests(),
                Arc::default(),
            )),
            ClientSwitches::for_tests(),
            LimitedActionsCopy::from_json("[]"),
        );
        let request = Request::new(vf_pb::EvaluateTweetsRequest {
            safety_level: level.0,
            viewer_id: Some(VIEWER),
            tweets: tweet_ids
                .iter()
                .map(|&tweet_id| vf_pb::TweetData {
                    tweet_id,
                    quote_context: None,
                })
                .collect(),
        });
        let response = endpoint.handle(request).await.unwrap().into_inner();
        response
            .results
            .into_iter()
            .map(|result| result.outcome.unwrap())
            .collect()
    }

    fn shown() -> Outcome {
        encoded(&TweetFieldsResultState::Found(TweetFieldsResultFound::new(
            None,
        )))
    }

    fn failed() -> Outcome {
        Outcome::Failed(vf_pb::Failed {})
    }

    #[tokio::test(start_paused = true)]
    async fn a_hung_flock_call_fails_a_post_only_through_a_must_have_node_it_carries() {
        use crate::hydration::{
            Hydrator, BEST_EFFORT_HYDRATION_TIMEOUT, HYDRATION_TIMEOUT, INBOUND_ALLOWANCE,
        };
        use crate::models::{ClientCapability, SafetyLabelType};
        use crate::rules::fixtures::{allow, dropped};
        use xai_visibility_filtering::models::FilteredReason;
        let protected = GizmoduckUserResult {
            user: Some(GizmoduckUser {
                safety: Safety {
                    is_protected: true,
                    ..Default::default()
                },
                ..Default::default()
            }),
            response_state: Some(UserResponseState::Found),
        };
        let spam = vf_pb::SafetyLabelMap {
            labels: HashMap::from([(SafetyLabelType::SPAM.0, vf_pb::SafetyLabel::default())]),
        };
        let world = || {
            InMemorySources::default()
                .tweet(1, 10)
                .tweet(2, 20)
                .tweet(3, 30)
                .labels(3, spam.clone())
                .authors(&[10, 30])
                .user(20, protected.clone())
                .fault(Source::Flock, Fault::Hangs)
        };
        let verdicts = [
            allow(),
            dropped(FilteredReason::AuthorIsProtected, "protected_author/drop"),
            dropped(FilteredReason::PossiblyUndesirable, "spam/drop/undesirable"),
        ];
        let served = vec![
            shown(),
            filtered(ThriftFilteredReason::AuthorIsProtected(true)),
            filtered(ThriftFilteredReason::SafetyResult(SafetyResult::new(
                None,
                Action::Drop(action::Drop::new(None, None)),
            ))),
        ];
        for (level, thrift, is_partial, outcomes, answered_at) in [
            (
                SafetyLevel::TimelineHomeHydration,
                ThriftLevel::TIMELINE_HOME_HYDRATION,
                false,
                served,
                BEST_EFFORT_HYDRATION_TIMEOUT,
            ),
            (
                SafetyLevel::TimelineHomeRecommendations,
                ThriftLevel::TIMELINE_HOME_RECOMMENDATIONS,
                true,
                vec![failed(); 3],
                HYDRATION_TIMEOUT - INBOUND_ALLOWANCE,
            ),
            (
                SafetyLevel::TimelineHome,
                ThriftLevel::TIMELINE_HOME,
                true,
                vec![failed(); 3],
                HYDRATION_TIMEOUT - INBOUND_ALLOWANCE,
            ),
        ] {
            let post_evaluator =
                PostEvaluator::new(Arc::new(world()), RuleEngine::for_tests(), Arc::default());
            let request = |rpc| PostBatch {
                viewer_id: Some(VIEWER),
                country_code: None,
                client_capability: ClientCapability::default(),
                safety_level: level,
                candidates: [1, 2, 3]
                    .map(|tweet_id| RawCandidate {
                        tweet_id: TweetId(tweet_id),
                        request_author_id: None,
                    })
                    .to_vec(),
                rpc,
            };
            let counted = [
                post_evaluator
                    .run(request(Rpc::FilterTweets))
                    .await
                    .outcomes,
                retweet::evaluate_merging_sources(&post_evaluator, request(Rpc::EvaluateTweets))
                    .await,
            ];
            for outcomes in counted {
                assert!(
                    outcomes[1]
                        .evaluation
                        .fail_open_defaults()
                        .contains(Hydrator::Follows),
                    "{level:?}"
                );
                for (outcome, verdict) in outcomes.into_iter().zip(&verdicts) {
                    assert_eq!(
                        matches!(outcome.evaluation, Evaluation::Partial { .. }),
                        is_partial,
                        "{level:?} {:?}",
                        outcome.evaluation
                    );
                    assert_eq!(&outcome.evaluation.into_verdict(), verdict, "{level:?}");
                }
            }
            let started = tokio::time::Instant::now();
            assert_eq!(
                evaluate_tweets(world(), thrift, &[1, 2, 3]).await,
                outcomes,
                "{level:?}"
            );
            assert_eq!(started.elapsed(), answered_at, "{level:?}");
        }
    }

    #[tokio::test(start_paused = true)]
    async fn a_failed_must_have_lookup_fails_exactly_the_posts_it_failed_for() {
        let world = || {
            InMemorySources::default()
                .tweet(1, 10)
                .tweet(2, 20)
                .tweet(3, 30)
                .authors(&[10, 20, 30])
        };
        for (world, outcomes) in [
            (
                world().fail_key(Source::SafetyLabels, 2),
                vec![shown(), failed(), shown()],
            ),
            (
                world().fault(Source::GizmoduckViewer, Fault::Fails),
                vec![failed(); 3],
            ),
        ] {
            assert_eq!(
                evaluate_tweets(world, ThriftLevel::TIMELINE_HOME_HYDRATION, &[1, 2, 3]).await,
                outcomes
            );
        }
    }

    #[test]
    fn a_conversation_control_limit_carries_its_prompt_in_the_request_language() {
        use crate::models::LimitedEngagementReason::ConversationControl;
        use crate::models::{Decided, LimitedEngagement, MediaRestriction, Verdict};
        use crate::params::limited_actions_copy::tests::BUNDLE;
        use crate::rules::fixtures::limited;
        use xai_x_thrift::action::{
            AnyInterstitial, CtaLimitedActionPrompt, Interstitial, InterstitialReason,
            LimitedAction, LimitedActionCtaType, LimitedActionPrompt, LimitedActionsPolicy,
            LimitedEngagements, TweetInterstitial,
        };
        let copy = LimitedActionsCopy::from_json(BUNDLE);
        let limit = limited(ConversationControl, "rule");
        let composite = Verdict::Shown {
            notice: None,
            media: Some(Decided {
                value: MediaRestriction::NsfwInterstitial,
                by: "nsfw_rule",
            }),
            engagement: Some(Decided {
                value: LimitedEngagement::new(ConversationControl),
                by: "rule",
            }),
        };
        for (language, subtext) in [
            ("", "Only some accounts can reply."),
            ("ja", "一部のアカウントのみが返信できます。"),
        ] {
            let context = TwitterContextViewer {
                request_language_code: language.to_string(),
                ..TwitterContextViewer::default()
            };
            let policies = ClientSwitches::for_tests().limited_actions_policies(
                Some(&context),
                None,
                None,
                [ConversationControl],
                &copy,
                None,
            );
            let thrift_limit = LimitedEngagements::new(
                action::LimitedEngagementReason::ConversationControl(
                    action::ConversationControl::new(),
                ),
                LimitedActionsPolicy::new(vec![LimitedAction::new(
                    action::LimitedActionType::REPLY,
                    LimitedActionPrompt::CtaLimitedActionPrompt(CtaLimitedActionPrompt::new(
                        "Who can reply?".to_string(),
                        subtext.to_string(),
                        LimitedActionCtaType::SEE_CONVERSATION,
                    )),
                )]),
                "limited_replies".to_string(),
            );
            let found = |action| {
                encoded(&TweetFieldsResultState::Found(TweetFieldsResultFound::new(
                    ThriftFilteredReason::SafetyResult(SafetyResult::new(None, action)),
                )))
            };
            for (verdict, action) in [
                (&limit, Action::LimitedEngagements(thrift_limit.clone())),
                (
                    &composite,
                    Action::TweetInterstitial(TweetInterstitial {
                        interstitial: Some(AnyInterstitial::Interstitial(Interstitial::new(
                            InterstitialReason::ContainsNsfwMedia(true),
                            None,
                        ))),
                        limited_engagements: Some(thrift_limit.clone()),
                        ..TweetInterstitial::default()
                    }),
                ),
            ] {
                let evaluation = Evaluation::complete(verdict.clone());
                assert_eq!(
                    evaluated_outcome(Sends::new(
                        &evaluation,
                        SafetyLevel::TimelineHomeHydration,
                        &policies
                    )),
                    found(action),
                    "{language:?} {verdict:?}"
                );
            }
        }
    }

    #[test]
    fn each_evaluation_sends_and_labels_as_vf_counts_it() {
        use crate::hydration::{Hydrator, Hydrators, Lookup};
        use crate::rules::fixtures::{allow, dropped};
        use xai_visibility_filtering::models::FilteredReason;
        let shown = TweetFieldsResultState::Found(TweetFieldsResultFound::new(None));
        let suspended = TweetFieldsResultState::Filtered(TweetFieldsResultFiltered::new(
            ThriftFilteredReason::AuthorIsSuspended(true),
        ));
        let cases = [
            (
                Evaluation::complete(allow()),
                Some(&shown),
                ("allow", "none"),
            ),
            (
                Evaluation::complete(dropped(FilteredReason::AuthorIsSuspended, "rule")),
                Some(&suspended),
                ("drop", "author_is_suspended"),
            ),
            (
                Evaluation::Partial {
                    verdict: dropped(FilteredReason::AuthorIsSuspended, "rule"),
                    fail_open_defaults: Hydrators::of(Hydrator::Follows),
                },
                None,
                ("failed", "none"),
            ),
            (
                Evaluation::NotFound(Lookup::Author),
                None,
                ("not_found", "none"),
            ),
            (Evaluation::Failed(Lookup::Tweet), None, ("failed", "none")),
        ];
        let policies = LimitedActionsPolicies::default();
        for (evaluation, sent, (class, reason)) in &cases {
            let sends = Sends::new(evaluation, SafetyLevel::TimelineHomeHydration, &policies);
            assert_eq!(sends.result_state().as_ref(), *sent, "{evaluation:?}");
            assert_eq!(
                sends.label(),
                ((*class).to_owned(), (*reason).to_owned()),
                "{evaluation:?}"
            );
        }
    }

    #[tokio::test]
    async fn the_forwarded_client_reaches_the_fetched_source_and_no_header_gets_the_defaults() {
        use crate::models::ClientCapability;
        use crate::rules::fixtures::CLIENT_CLASSES;
        let sources = Arc::new(
            InMemorySources::default()
                .pure_core(
                    4,
                    PureCoreData {
                        author_id: 40,
                        source_tweet_id: Some(6),
                        ..Default::default()
                    },
                )
                .tweet(6, 60),
        );
        let post_evaluator = Arc::new(PostEvaluator::new(
            sources,
            RuleEngine::for_tests(),
            Arc::default(),
        ));
        let endpoint = EvaluateTweetsEndpoint::new(
            Arc::clone(&post_evaluator),
            ClientSwitches::for_tests(),
            LimitedActionsCopy::from_json("[]"),
        );
        let class = CLIENT_CLASSES
            .iter()
            .find(|class| {
                class.capability.verify_blur_support == Some(VerifyBlurSupport::IosNeedsUpdate)
            })
            .unwrap();
        let header = xai_twittercontext::hydrate_twitter_context(&TwitterContextViewer {
            client_application_id: class.app_id,
            user_agent: class.user_agent.into(),
            ..TwitterContextViewer::default()
        })
        .unwrap();
        for (header, expected) in [
            (Some(header), class.capability),
            (None, ClientCapability::default()),
        ] {
            let mut request = Request::new(vf_pb::EvaluateTweetsRequest {
                safety_level: ThriftLevel::TIMELINE_HOME_HYDRATION.0,
                viewer_id: Some(1),
                tweets: vec![vf_pb::TweetData {
                    tweet_id: 4,
                    quote_context: None,
                }],
            });
            if let Some(header) = header {
                request.metadata_mut().insert("twittercontext", header);
            }
            endpoint.handle(request).await.unwrap();
            assert_eq!(
                std::mem::take(&mut *post_evaluator.client_capabilities.lock().unwrap()),
                [expected]
            );
        }
    }
}
