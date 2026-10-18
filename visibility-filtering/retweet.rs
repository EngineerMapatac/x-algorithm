use crate::hydration::Hydrators;
use crate::metrics as ft_metrics;
use crate::metrics::Rpc;
use crate::models::{Decided, Evaluation, TweetId, Verdict, Withholding};
use crate::post_evaluator::{
    HydratedRequest, PostBatch, PostEvaluation, PostEvaluator, PostObserver,
};
use crate::rules::{NoTrace, PolicyTrace};
use std::collections::{HashMap, HashSet};

const RETWEET_SOURCES: &str = "evaluate_tweets_retweet_sources";

pub(crate) async fn evaluate_merging_sources(
    post_evaluator: &PostEvaluator,
    request: PostBatch,
) -> Vec<PostEvaluation> {
    let hydrated = post_evaluator.hydrate_with_retweet_sources(request).await;
    without_selections(merge_sources(post_evaluator, &hydrated, &mut NoTrace))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MergeSelection {
    RequestedOnly,
    RequestedIncomplete,
    SourceIncomplete,
    SourceWithheld,
    RequestedWithheld,
    SourceRestrictions,
    RequestedRestrictions,
}

#[derive(Default)]
pub(crate) struct Traced {
    pub(crate) policies: HashMap<TweetId, PolicyTrace>,
    pub(crate) selections: HashMap<TweetId, MergeSelection>,
}

pub(crate) async fn hydrate_and_trace(
    post_evaluator: &PostEvaluator,
    request: PostBatch,
) -> (HydratedRequest, Vec<PostEvaluation>, Traced) {
    let rpc = request.rpc;
    let mut traced = Traced::default();
    let (hydrated, selected) =
        hydrate_and_select(post_evaluator, request, &mut traced.policies).await;
    let outcomes = selected
        .into_iter()
        .map(|(outcome, selection)| {
            traced.selections.insert(outcome.tweet_id, selection);
            outcome
        })
        .collect();
    if rpc == Rpc::EvaluateTweets {
        trace_unmerged_sources(post_evaluator, &hydrated, &mut traced);
    }
    (hydrated, outcomes, traced)
}

pub(crate) async fn hydrate_and_evaluate(
    post_evaluator: &PostEvaluator,
    request: PostBatch,
) -> (HydratedRequest, Vec<PostEvaluation>) {
    let (hydrated, selected) = hydrate_and_select(post_evaluator, request, &mut NoTrace).await;
    (hydrated, without_selections(selected))
}

async fn hydrate_and_select<O: PostObserver>(
    post_evaluator: &PostEvaluator,
    request: PostBatch,
    observer: &mut O,
) -> (HydratedRequest, Vec<(PostEvaluation, MergeSelection)>) {
    match request.rpc {
        Rpc::FilterTweets => {
            let hydrated = post_evaluator.hydrate(request).await;
            let selected = post_evaluator
                .evaluate_in_request_order(&hydrated, observer)
                .into_iter()
                .map(|outcome| (outcome, MergeSelection::RequestedOnly))
                .collect();
            (hydrated, selected)
        }
        Rpc::EvaluateTweets => {
            let hydrated = post_evaluator.hydrate_with_retweet_sources(request).await;
            let selected = merge_sources(post_evaluator, &hydrated, observer);
            (hydrated, selected)
        }
    }
}

fn without_selections(selected: Vec<(PostEvaluation, MergeSelection)>) -> Vec<PostEvaluation> {
    selected.into_iter().map(|(outcome, _)| outcome).collect()
}

fn trace_unmerged_sources(
    post_evaluator: &PostEvaluator,
    hydrated: &HydratedRequest,
    traced: &mut Traced,
) {
    let hydration = hydrated.hydration();
    let unmerged: Vec<TweetId> = hydrated
        .requested_ids()
        .filter_map(|tweet_id| hydration.tweet(tweet_id)?.source_tweet_id())
        .filter(|source_id| {
            !traced.policies.contains_key(source_id)
                && hydration
                    .tweet(*source_id)
                    .is_some_and(|source| source.candidate().is_some())
        })
        .collect();
    if !unmerged.is_empty() {
        post_evaluator.trace(hydrated, unmerged, &mut traced.policies);
    }
}

fn merge_sources<O: PostObserver>(
    post_evaluator: &PostEvaluator,
    hydrated: &HydratedRequest,
    observer: &mut O,
) -> Vec<(PostEvaluation, MergeSelection)> {
    let (in_batch, fetched) = source_ids(hydrated);
    if in_batch.is_empty() && fetched.is_empty() {
        return post_evaluator
            .evaluate_in_request_order(hydrated, observer)
            .into_iter()
            .map(|outcome| {
                let selection = requested_selection(&outcome.evaluation);
                (outcome, selection)
            })
            .collect();
    }
    ft_metrics::incr_nonzero(
        RETWEET_SOURCES,
        &[("outcome", "in_batch")],
        in_batch.len() as u64,
    );
    ft_metrics::incr_nonzero(
        RETWEET_SOURCES,
        &[("outcome", "fetched")],
        fetched.len() as u64,
    );
    let mut outcomes = post_evaluator.evaluate(
        hydrated,
        hydrated.requested_ids().chain(fetched.iter().copied()),
        |tweet_id| !fetched.contains(&tweet_id),
        observer,
    );
    let mut sources: HashMap<TweetId, Evaluation> = in_batch
        .iter()
        .filter_map(|&source_id| Some((source_id, outcomes.get(&source_id)?.evaluation.clone())))
        .collect();
    sources.extend(fetched.iter().filter_map(|source_id| {
        let source = outcomes.remove(source_id)?;
        Some((source.tweet_id, source.evaluation))
    }));
    let mut merged: Vec<(PostEvaluation, MergeSelection)> =
        Vec::with_capacity(hydrated.requested_ids().len());
    for tweet_id in hydrated.requested_ids() {
        let selected = match outcomes.remove(&tweet_id) {
            Some(outcome) => merge_source(outcome, &sources),
            None => match merged.iter().rfind(|(prior, _)| prior.tweet_id == tweet_id) {
                Some(selected) => selected.clone(),
                None => continue,
            },
        };
        merged.push(selected);
    }
    merged
}

fn merge_source(
    outcome: PostEvaluation,
    sources: &HashMap<TweetId, Evaluation>,
) -> (PostEvaluation, MergeSelection) {
    let (evaluation, selection) = match (outcome.evaluation, outcome.source_tweet_id) {
        (
            Evaluation::Complete {
                verdict,
                fail_open_defaults,
            },
            Some(source_id),
        ) => {
            let source = sources.get(&source_id);
            let fail_open_defaults = fail_open_defaults
                .union(source.map_or(Hydrators::empty(), Evaluation::fail_open_defaults));
            match source {
                Some(Evaluation::Complete {
                    verdict: source, ..
                }) => {
                    let selection = merge_selection(&verdict, source);
                    let verdict = match selection {
                        MergeSelection::SourceWithheld | MergeSelection::SourceRestrictions => {
                            source.clone()
                        }
                        MergeSelection::RequestedOnly
                        | MergeSelection::RequestedIncomplete
                        | MergeSelection::SourceIncomplete
                        | MergeSelection::RequestedWithheld
                        | MergeSelection::RequestedRestrictions => verdict,
                    };
                    (
                        Evaluation::Complete {
                            verdict,
                            fail_open_defaults,
                        },
                        selection,
                    )
                }
                Some(
                    Evaluation::Partial { .. } | Evaluation::NotFound(_) | Evaluation::Failed(_),
                )
                | None => (
                    Evaluation::Partial {
                        verdict,
                        fail_open_defaults,
                    },
                    MergeSelection::SourceIncomplete,
                ),
            }
        }
        (evaluation, _) => {
            let selection = requested_selection(&evaluation);
            (evaluation, selection)
        }
    };
    (
        PostEvaluation {
            evaluation,
            ..outcome
        },
        selection,
    )
}

fn requested_selection(evaluation: &Evaluation) -> MergeSelection {
    match evaluation {
        Evaluation::Partial { .. } => MergeSelection::RequestedIncomplete,
        Evaluation::Complete { .. } | Evaluation::NotFound(_) | Evaluation::Failed(_) => {
            MergeSelection::RequestedOnly
        }
    }
}

fn source_ids(hydrated: &HydratedRequest) -> (HashSet<TweetId>, HashSet<TweetId>) {
    let hydration = hydrated.hydration();
    let sources: HashSet<TweetId> = hydrated
        .requested_ids()
        .filter_map(|tweet_id| hydration.tweet(tweet_id)?.source_to_merge())
        .collect();
    if sources.is_empty() {
        return Default::default();
    }
    let requested: HashSet<TweetId> = hydrated.requested_ids().collect();
    sources
        .into_iter()
        .partition(|source_id| requested.contains(source_id))
}

fn merge_selection(retweet: &Verdict, source: &Verdict) -> MergeSelection {
    match (retweet, source) {
        (
            Verdict::Withheld(Decided {
                value: Withholding::Drop(_),
                ..
            }),
            Verdict::Withheld(Decided {
                value: Withholding::Tombstone(_),
                ..
            }),
        ) => MergeSelection::RequestedWithheld,
        (_, Verdict::Withheld(_)) => MergeSelection::SourceWithheld,
        (Verdict::Withheld(_), _) => MergeSelection::RequestedWithheld,
        (
            Verdict::Shown {
                notice: None,
                media: None,
                engagement: None,
            },
            _,
        ) => MergeSelection::SourceRestrictions,
        _ => MergeSelection::RequestedRestrictions,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clients::socialgraph_client::{EdgeDirection, Graph};
    use crate::hydration::inventory::Source;
    use crate::hydration::sources::{control, suspended, InMemorySources};
    use crate::hydration::{Hydrator, Lookup};
    use crate::metrics::Rpc;
    use crate::models::{
        ClientCapability, DropReason, LimitedEngagement, LimitedEngagementReason, MediaFeature,
        MediaInterstitial, MediaRestriction, NsfwFeature, RawCandidate, SafetyLabelType,
        TombstoneReason, TweetFeatures,
    };
    use crate::rules::fixtures::{allow, legacy_interstitial, limited, noticed};
    use crate::rules::SafetyLevel;
    use crate::scenario::{raw, Scenario};
    use std::time::Duration;
    use xai_core_entities::entities::{ConversationControlArm, PureCoreData};
    use xai_visibility_filtering::models::FilteredReason;
    use xai_visibility_filtering_proto as vf_pb;
    use xai_x_thrift::action::InterstitialReason;

    #[test]
    fn merge_source_prefers_the_more_severe_withholding_then_restricted_retweet() {
        let dropped = Verdict::Withheld(Decided {
            value: Withholding::Drop(DropReason::Legacy(FilteredReason::AuthorIsSuspended)),
            by: "suspended_author/drop",
        });
        let tombstoned = Verdict::Withheld(Decided {
            value: Withholding::Tombstone(TombstoneReason::LocalRegulations),
            by: "nsfw_high_precision/tombstone/local_regulations",
        });
        let blurred = Verdict::Shown {
            notice: None,
            media: Some(Decided {
                value: MediaRestriction::MediaInterstitial(MediaInterstitial {
                    legacy: FilteredReason::ContainNsfwMedia,
                    reason: InterstitialReason::Sensitive(true),
                    prompt: None,
                }),
                by: "nsfw_user/blur/sensitive_user",
            }),
            engagement: None,
        };
        let limited = Verdict::Shown {
            notice: None,
            media: None,
            engagement: Some(Decided {
                value: LimitedEngagement::new(LimitedEngagementReason::ConversationControl),
                by: "limit_replies_by_invitation/limited_engagement/conversation_control",
            }),
        };
        let unrestricted = Verdict::Shown {
            notice: None,
            media: None,
            engagement: None,
        };
        let noticed = noticed(
            true,
            false,
            "fosnr_abuse_insults_follower/soft_intervention/abuse",
        );
        for (retweet, source, merged) in [
            (&Verdict::not_found(), &dropped, &dropped),
            (&dropped, &tombstoned, &dropped),
            (&tombstoned, &dropped, &dropped),
            (&unrestricted, &tombstoned, &tombstoned),
            (&dropped, &limited, &dropped),
            (&unrestricted, &limited, &limited),
            (&blurred, &limited, &blurred),
            (&unrestricted, &noticed, &noticed),
            (&limited, &noticed, &limited),
        ] {
            let retweet = PostEvaluation {
                tweet_id: TweetId(1),
                source_tweet_id: Some(TweetId(5)),
                evaluation: Evaluation::complete(retweet.clone()),
                safety_labels: None,
            };
            let sources = HashMap::from([(TweetId(5), Evaluation::complete(source.clone()))]);
            let (outcome, _) = merge_source(retweet, &sources);
            assert_eq!(outcome.evaluation, Evaluation::complete(merged.clone()));
        }
    }

    #[tokio::test]
    async fn a_retweet_has_media_exactly_when_its_source_uploaded_some_in_the_batch_or_fetched() {
        let features = |nsfw_admin, media| TweetFeatures {
            nsfw: NsfwFeature {
                user: false,
                admin: nsfw_admin,
            },
            media,
            ..Default::default()
        };
        let media = |has_media, has_uploaded_media| MediaFeature {
            has_media,
            has_uploaded_media,
            ..Default::default()
        };
        let scenario = evaluate_tweets(
            InMemorySources::default()
                .retweet(4, 40, 6, None)
                .retweet(5, 50, 7, None)
                .tweet(6, 60)
                .tweet(7, 70)
                .authors(&[40, 50, 60, 70])
                .tweet_features(4, features(true, media(false, false)))
                .tweet_features(5, features(true, media(true, true)))
                .tweet_features(6, features(false, media(true, true)))
                .tweet_features(7, features(false, media(true, false))),
        );
        let blurred = Evaluation::complete(legacy_interstitial("nsfw_account/legacy_interstitial"));
        let allowed = Evaluation::complete(allow());
        for tweet_ids in [&[4, 5, 6, 7][..], &[4, 5]] {
            let evaluations = scenario.evaluate(&requested(tweet_ids)).await.evaluations();
            assert_eq!(evaluations[0], blurred, "{tweet_ids:?}");
            assert_eq!(evaluations[1], allowed, "{tweet_ids:?}");
        }
    }

    #[tokio::test]
    async fn a_plain_article_retweet_merges_its_sources_age_drop_or_blur() {
        use crate::hydration::sources::found_user;
        use crate::models::{ArticleLifecycle, NsfwViewerDropReason};
        use crate::rules::fixtures::{blurred, nsfw_viewer_dropped, CLIENT_CLASSES};
        use std::num::NonZeroU64;
        use xai_core_entities::entities::{ExtendedProfile, GizmoduckUser};
        for (age, country, expected) in [
            (
                Some(17),
                "us",
                nsfw_viewer_dropped(
                    NsfwViewerDropReason::IsUnderage,
                    "sensitive_viewer_underage/drop",
                ),
            ),
            (
                None,
                "de",
                nsfw_viewer_dropped(
                    NsfwViewerDropReason::HasNoStatedAge,
                    "sensitive_viewer_no_stated_age/drop",
                ),
            ),
            (
                Some(18),
                "us",
                blurred(
                    InterstitialReason::SensitiveUser(true),
                    "nsfw_user/blur/sensitive_user",
                ),
            ),
        ] {
            for cover in [true, false] {
                for tweet_ids in [&[4, 6][..], &[4]] {
                    let mut author = found_user(60);
                    author.user.as_mut().unwrap().safety.nsfw_user = true;
                    let sources = InMemorySources::default()
                        .retweet(4, 40, 6, Some(60))
                        .tweet(6, 60)
                        .authors(&[40])
                        .user(60, author)
                        .viewer(
                            1,
                            GizmoduckUser {
                                user_id: 1,
                                extended_profile: Some(ExtendedProfile { age_in_years: age }),
                                ..Default::default()
                            },
                        )
                        .tweet_features(
                            6,
                            TweetFeatures {
                                article_id: NonZeroU64::new(70),
                                nsfw: NsfwFeature {
                                    user: true,
                                    admin: false,
                                },
                                ..Default::default()
                            },
                        )
                        .lifecycle(70, ArticleLifecycle::Published)
                        .article_cover_media(70, cover)
                        .article_media_entities(70, !cover);
                    let scenario = evaluate_tweets(sources)
                        .country(country)
                        .client(CLIENT_CLASSES[0].capability);
                    let evaluations = scenario.evaluate(&requested(tweet_ids)).await.evaluations();
                    assert_eq!(
                        evaluations[0],
                        Evaluation::complete(expected.clone()),
                        "{age:?} {cover} {tweet_ids:?}"
                    );
                }
            }
        }
    }

    fn evaluate_tweets(sources: InMemorySources) -> Scenario<InMemorySources> {
        Scenario::new(sources, SafetyLevel::TimelineHomeHydration)
            .viewer(Some(1))
            .rpc(Rpc::EvaluateTweets)
    }

    fn requested(tweet_ids: &[u64]) -> Vec<RawCandidate> {
        tweet_ids
            .iter()
            .map(|&tweet_id| raw(tweet_id, None))
            .collect()
    }

    fn retweet_world(share_names_author: bool) -> InMemorySources {
        let share = |source_user_id| Some(source_user_id).filter(|_| share_names_author);
        let community = |root| control(ConversationControlArm::Community, root, &[]);
        InMemorySources::default()
            .retweet(1, 10, 5, share(20))
            .tweet(2, 20)
            .retweet(3, 30, 5, share(20))
            .retweet(4, 40, 6, share(60))
            .tweet(5, 20)
            .pure_core(
                6,
                PureCoreData {
                    author_id: 60,
                    conversation_id: Some(100),
                    in_reply_to_tweet_id: Some(100),
                    in_reply_to_user_id: Some(70),
                    ..Default::default()
                },
            )
            .control(2, community(90))
            .control(5, community(80))
            .authors(&[10, 20, 30, 40, 60])
    }

    const TWEET_KEYED: [Source; 4] = [
        Source::TesPureCoreData,
        Source::TesTweet,
        Source::TesConversationControl,
        Source::SafetyLabels,
    ];

    fn batches(sources: &InMemorySources) -> String {
        let mut lines: Vec<String> = TWEET_KEYED
            .into_iter()
            .chain([Source::GizmoduckViewer, Source::GizmoduckAuthor])
            .map(|source| format!("{source:?} {:?}", sources.keys(source)))
            .collect();
        for queries in sources.selects() {
            let queries: Vec<String> = queries
                .iter()
                .map(|q| {
                    let direction = match q.direction {
                        EdgeDirection::Forward => "fwd",
                        EdgeDirection::Reverse => "rev",
                    };
                    format!(
                        "{}-{direction}{:?}",
                        <&str>::from(q.graph),
                        q.destination_ids
                    )
                })
                .collect();
            lines.push(format!("Flock {}", queries.join(" ")));
        }
        lines.join("\n")
    }

    #[tokio::test]
    async fn sources_join_unsent_calls_and_sent_calls_follow_with_only_new_keys() {
        let not_a_reply = || {
            retweet_world(true).pure_core(
                6,
                PureCoreData {
                    author_id: 60,
                    ..Default::default()
                },
            )
        };
        let tweet_keyed = |keys: &str| {
            TWEET_KEYED
                .map(|source| format!("{source:?} {keys}"))
                .join("\n")
        };
        let rows = [
            (
                "a",
                retweet_world(true),
                &[2][..],
                format!(
                    "{}\nGizmoduckViewer [[1]]\nGizmoduckAuthor [[20]]
Flock follows-rev[90] super_follows-fwd[] blocks-rev[90]
Flock follows-fwd[20] blocks-rev[20]",
                    tweet_keyed("[[2]]")
                ),
            ),
            (
                "b",
                retweet_world(true),
                &[1, 2, 3, 5][..],
                format!(
                    "{}\nGizmoduckViewer [[1]]\nGizmoduckAuthor [[10, 20, 30]]
Flock follows-rev[80, 90] super_follows-fwd[] blocks-rev[80, 90]
Flock follows-fwd[10, 20, 30] blocks-rev[10, 20, 30]",
                    tweet_keyed("[[1, 2, 3, 5]]")
                ),
            ),
            (
                "c",
                retweet_world(true),
                &[1, 2, 3, 4][..],
                format!(
                    "{}\nGizmoduckViewer [[1]]\nGizmoduckAuthor [[10, 20, 30, 40, 60]]
Flock follows-rev[90] super_follows-fwd[] blocks-rev[90]
Flock follows-fwd[10, 20, 30, 40, 60] blocks-rev[10, 20, 30, 40, 60]
Flock follows-rev[80] super_follows-fwd[] blocks-rev[80]
Flock follows-fwd[] blocks-rev[70]",
                    tweet_keyed("[[1, 2, 3, 4], [5, 6]]")
                ),
            ),
            (
                "c, share without the source author",
                retweet_world(false),
                &[1, 2, 3, 4][..],
                format!(
                    "{}\nGizmoduckViewer [[1]]\nGizmoduckAuthor [[10, 20, 30, 40], [60]]
Flock follows-rev[90] super_follows-fwd[] blocks-rev[90]
Flock follows-fwd[10, 20, 30, 40] blocks-rev[10, 20, 30, 40]
Flock follows-rev[80] super_follows-fwd[] blocks-rev[80]
Flock follows-fwd[60] blocks-rev[60, 70]",
                    tweet_keyed("[[1, 2, 3, 4], [5, 6]]")
                ),
            ),
            (
                "c, source 6 no reply",
                not_a_reply(),
                &[1, 2, 3, 4][..],
                format!(
                    "{}\nGizmoduckViewer [[1]]\nGizmoduckAuthor [[10, 20, 30, 40, 60]]
Flock follows-rev[90] super_follows-fwd[] blocks-rev[90]
Flock follows-fwd[10, 20, 30, 40, 60] blocks-rev[10, 20, 30, 40, 60]
Flock follows-rev[80] super_follows-fwd[] blocks-rev[80]",
                    tweet_keyed("[[1, 2, 3, 4], [5, 6]]")
                ),
            ),
        ];
        for (name, world, tweet_ids, expected) in rows {
            let scenario = evaluate_tweets(world);
            let evaluations = scenario.evaluate(&requested(tweet_ids)).await.evaluations();
            assert_eq!(batches(scenario.sources()), expected, "{name}");
            assert_no_key_asked_twice(scenario.sources(), name);
            assert!(
                evaluations
                    .iter()
                    .all(|evaluation| matches!(evaluation, Evaluation::Complete { .. })),
                "{name}"
            );
        }
        for tweet_ids in [&[2][..], &[1, 2, 3, 5]] {
            let plain = evaluate_tweets(retweet_world(true)).rpc(Rpc::FilterTweets);
            plain.hydrate(&requested(tweet_ids)).await;
            let merged = evaluate_tweets(retweet_world(true));
            merged.evaluate(&requested(tweet_ids)).await;
            let (merged, plain) = (merged.sources(), plain.sources());
            assert_eq!(merged.calls(), plain.calls(), "{tweet_ids:?}");
            assert_eq!(batches(merged), batches(plain), "{tweet_ids:?}");
        }
    }

    fn assert_no_key_asked_twice(sources: &InMemorySources, name: &str) {
        let mut asked: HashSet<String> = HashSet::new();
        for source in TWEET_KEYED
            .into_iter()
            .chain([Source::GizmoduckViewer, Source::GizmoduckAuthor])
        {
            for key in sources.keys(source).into_iter().flatten() {
                assert!(
                    asked.insert(format!("{source:?} {key}")),
                    "{name}: {source:?} {key}"
                );
            }
        }
        for query in sources.selects().into_iter().flatten() {
            for key in &query.destination_ids {
                let edge = format!("{:?} {:?} {key}", query.graph, query.direction);
                assert!(asked.insert(edge.clone()), "{name}: {edge}");
            }
        }
    }

    #[derive(Clone, Copy, Debug, PartialEq)]
    enum Arm {
        Complete,
        Partial,
        NotFound(Lookup),
        Failed(Lookup),
    }

    fn arms(outcomes: &[PostEvaluation]) -> Vec<Arm> {
        outcomes
            .iter()
            .map(|outcome| match outcome.evaluation {
                Evaluation::Complete { .. } => Arm::Complete,
                Evaluation::Partial { .. } => Arm::Partial,
                Evaluation::NotFound(lookup) => Arm::NotFound(lookup),
                Evaluation::Failed(lookup) => Arm::Failed(lookup),
            })
            .collect()
    }

    fn starts(sources: &InMemorySources, t0: tokio::time::Instant) -> Vec<Vec<Duration>> {
        TWEET_KEYED
            .into_iter()
            .chain([
                Source::GizmoduckViewer,
                Source::GizmoduckAuthor,
                Source::Flock,
            ])
            .map(|source| {
                sources
                    .starts(source)
                    .into_iter()
                    .map(|started| started - t0)
                    .collect()
            })
            .collect()
    }

    fn slow_world(share_names_author: bool) -> InMemorySources {
        let ms = Duration::from_millis;
        retweet_world(share_names_author)
            .latency(Source::TesPureCoreData, ms(10))
            .latency(Source::TesTweet, ms(30))
            .latency(Source::TesConversationControl, ms(20))
            .latency(Source::SafetyLabels, ms(15))
            .latency(Source::GizmoduckViewer, ms(5))
            .latency(Source::GizmoduckAuthor, ms(10))
            .latency(Source::Flock, ms(10))
    }

    #[tokio::test(start_paused = true)]
    async fn requested_calls_start_as_they_do_without_sources() {
        let ms = Duration::from_millis;
        for share_names_author in [true, false] {
            let raw = requested(&[1, 2, 3, 4]);
            let alone = evaluate_tweets(slow_world(share_names_author)).rpc(Rpc::FilterTweets);
            let t0 = tokio::time::Instant::now();
            alone.hydrate(&raw).await;
            let alone = starts(alone.sources(), t0);

            let expanded = evaluate_tweets(slow_world(share_names_author));
            let t0 = tokio::time::Instant::now();
            expanded.evaluate(&raw).await;
            let elapsed = t0.elapsed();
            let expanded = starts(expanded.sources(), t0);

            for (alone, expanded) in alone.iter().zip(&expanded) {
                assert_eq!(alone[..], expanded[..alone.len()], "{share_names_author}");
            }
            for tweet_keyed in &expanded[..4] {
                assert_eq!(tweet_keyed, &[ms(0), ms(10)], "{share_names_author}");
            }
            let authors = if share_names_author {
                vec![ms(10)]
            } else {
                vec![ms(10), ms(20)]
            };
            assert_eq!(expanded[5], authors, "{share_names_author}");
            assert_eq!(
                expanded[6],
                [ms(10), ms(20), ms(20), ms(30)],
                "{share_names_author}"
            );
            assert_eq!(elapsed, ms(40), "{share_names_author}");
        }
    }

    #[tokio::test(start_paused = true)]
    async fn a_source_failure_fails_only_its_retweets() {
        use crate::hydration::{BEST_EFFORT_HYDRATION_TIMEOUT, HYDRATION_TIMEOUT};
        use Arm::{Complete, Partial};
        let rows = [
            (
                "source tweet row",
                retweet_world(true).fail_key(Source::TesTweet, 6),
                [
                    Complete,
                    Complete,
                    Complete,
                    Arm::Failed(Lookup::SharedTweet),
                ],
                Duration::ZERO,
            ),
            (
                "source pure core",
                retweet_world(true).fail_key(Source::TesPureCoreData, 6),
                [
                    Complete,
                    Complete,
                    Complete,
                    Arm::Failed(Lookup::SharedTweet),
                ],
                Duration::ZERO,
            ),
            (
                "source-only author in the shared author call",
                retweet_world(true).fail_key(Source::GizmoduckAuthor, 60),
                [
                    Complete,
                    Complete,
                    Complete,
                    Arm::Failed(Lookup::SharedAuthor),
                ],
                Duration::ZERO,
            ),
            (
                "deleted source",
                retweet_world(true).retweet(4, 40, 7, Some(70)),
                [
                    Complete,
                    Complete,
                    Complete,
                    Arm::NotFound(Lookup::SharedTweet),
                ],
                Duration::ZERO,
            ),
            (
                "source reply-root select",
                retweet_world(true).fail_edge(Graph::Blocks, 70),
                [Complete; 4],
                Duration::ZERO,
            ),
            (
                "source root-edge select, the root requested as a Subscribers root",
                retweet_world(true)
                    .control(2, control(ConversationControlArm::Subscribers, 80, &[]))
                    .fail_edge(Graph::Follows, 80),
                [Complete; 4],
                Duration::ZERO,
            ),
            (
                "timed-out source root select",
                retweet_world(true).key_latency(Source::Flock, 80, HYDRATION_TIMEOUT * 2),
                [Complete; 4],
                BEST_EFFORT_HYDRATION_TIMEOUT,
            ),
            (
                "timed-out source controls",
                retweet_world(true).key_latency(
                    Source::TesConversationControl,
                    5,
                    HYDRATION_TIMEOUT * 2,
                ),
                [Complete; 4],
                BEST_EFFORT_HYDRATION_TIMEOUT,
            ),
            (
                "source safety labels",
                retweet_world(true).fail_key(Source::SafetyLabels, 6),
                [Complete, Complete, Complete, Partial],
                Duration::ZERO,
            ),
        ];
        for (name, world, expected, elapsed) in rows {
            let t0 = tokio::time::Instant::now();
            let evaluated = evaluate_tweets(world)
                .evaluate(&requested(&[1, 2, 3, 4]))
                .await;
            assert_eq!(t0.elapsed(), elapsed, "{name}");
            assert_eq!(arms(&evaluated.outcomes), expected, "{name}");
        }
    }

    #[tokio::test(start_paused = true)]
    async fn a_retweet_whose_only_failure_is_best_effort_still_takes_its_sources_drop() {
        use crate::rules::fixtures::dropped;
        let world = || {
            retweet_world(true)
                .user(60, suspended())
                .fail_edge(Graph::Follows, 40)
        };
        assert_eq!(
            evaluate_tweets(world())
                .evaluate(&requested(&[4]))
                .await
                .evaluations(),
            [Evaluation::Complete {
                verdict: dropped(FilteredReason::AuthorIsSuspended, "suspended_author/drop"),
                fail_open_defaults: Hydrators::of(Hydrator::BlockedByAuthor),
            }]
        );
    }

    #[derive(Clone, Copy, Debug)]
    enum Answer {
        Found,
        NotFound,
        Failed,
    }

    fn shared_lookups(
        author: Answer,
        shared_tweet: Answer,
        shared_author: Answer,
        share_names_author: bool,
    ) -> InMemorySources {
        let source_user_id = Some(20).filter(|_| share_names_author);
        let sources = InMemorySources::default().retweet(1, 10, 5, source_user_id);
        let sources = match shared_tweet {
            Answer::Found => sources.tweet(5, 20),
            Answer::NotFound => sources,
            Answer::Failed => sources.tweet(5, 20).fail_key(Source::TesPureCoreData, 5),
        };
        [(10, author), (20, shared_author)].into_iter().fold(
            sources,
            |sources, (user_id, answer)| match answer {
                Answer::Found => sources.authors(&[user_id]),
                Answer::NotFound => sources,
                Answer::Failed => sources.fail_key(Source::GizmoduckAuthor, user_id),
            },
        )
    }

    async fn retweet_status(sources: InMemorySources, tweet_ids: &[u64]) -> Evaluation {
        let evaluated = evaluate_tweets(sources)
            .evaluate(&requested(tweet_ids))
            .await;
        evaluated.evaluations().remove(0)
    }

    #[tokio::test]
    async fn a_retweet_resolves_only_when_its_shared_tweet_and_shared_author_answer() {
        use Answer::{Failed, Found, NotFound};
        let judged = Evaluation::complete(allow());
        let both = |expected: Evaluation| [expected.clone(), expected];
        let rows = [
            (Found, Found, Found, both(judged.clone())),
            (
                Found,
                Found,
                NotFound,
                both(Evaluation::NotFound(Lookup::SharedAuthor)),
            ),
            (
                Found,
                Found,
                Failed,
                both(Evaluation::Failed(Lookup::SharedAuthor)),
            ),
            (
                Found,
                NotFound,
                Found,
                both(Evaluation::NotFound(Lookup::SharedTweet)),
            ),
            (
                Found,
                NotFound,
                NotFound,
                both(Evaluation::NotFound(Lookup::SharedTweet)),
            ),
            (
                Found,
                NotFound,
                Failed,
                both(Evaluation::NotFound(Lookup::SharedTweet)),
            ),
            (
                Found,
                Failed,
                Found,
                both(Evaluation::Failed(Lookup::SharedTweet)),
            ),
            (
                Found,
                Failed,
                NotFound,
                [
                    Evaluation::NotFound(Lookup::SharedAuthor),
                    Evaluation::Failed(Lookup::SharedTweet),
                ],
            ),
            (
                Found,
                Failed,
                Failed,
                both(Evaluation::Failed(Lookup::SharedTweet)),
            ),
            (
                Failed,
                Found,
                NotFound,
                both(Evaluation::NotFound(Lookup::SharedAuthor)),
            ),
            (
                Failed,
                Found,
                Failed,
                both(Evaluation::Failed(Lookup::Author)),
            ),
            (
                Failed,
                Failed,
                Found,
                both(Evaluation::Failed(Lookup::SharedTweet)),
            ),
            (
                NotFound,
                Failed,
                Found,
                both(Evaluation::NotFound(Lookup::Author)),
            ),
        ];
        for (author, shared_tweet, shared_author, expected) in rows {
            for (tweet_ids, expected) in [&[1][..], &[1, 5]].into_iter().zip(expected) {
                assert_eq!(
                    retweet_status(
                        shared_lookups(author, shared_tweet, shared_author, true),
                        tweet_ids
                    )
                    .await,
                    expected,
                    "{author:?} {shared_tweet:?} {shared_author:?} {tweet_ids:?}"
                );
            }
        }

        let unnamed = [
            (Found, NotFound, Evaluation::NotFound(Lookup::SharedAuthor)),
            (Found, Failed, Evaluation::Failed(Lookup::SharedAuthor)),
            (Failed, NotFound, Evaluation::Failed(Lookup::SharedTweet)),
        ];
        for (shared_tweet, shared_author, expected) in unnamed {
            assert_eq!(
                retweet_status(
                    shared_lookups(Found, shared_tweet, shared_author, false),
                    &[1]
                )
                .await,
                expected,
                "unnamed {shared_tweet:?} {shared_author:?}"
            );
        }

        let evaluated = evaluate_tweets(shared_lookups(Found, NotFound, NotFound, true))
            .rpc(Rpc::FilterTweets)
            .evaluate(&requested(&[1]))
            .await;
        assert_eq!(evaluated.evaluations(), [judged]);
    }

    #[tokio::test(start_paused = true)]
    async fn a_requested_key_asked_by_a_source_batch_shares_its_fate() {
        let ms = Duration::from_millis;
        let world = |root_fails: bool| {
            let world = retweet_world(true)
                .control(5, control(ConversationControlArm::Community, 90, &[]))
                .key_latency(Source::TesConversationControl, 2, ms(30));
            if root_fails {
                world.fail_edge(Graph::Follows, 90)
            } else {
                world
            }
        };
        for (root_fails, failed_ids) in [(false, &[][..]), (true, &[2])] {
            let hydrated = evaluate_tweets(world(root_fails))
                .hydrate(&requested(&[1, 2, 3, 4]))
                .await;
            assert_eq!(
                hydrated.failed_ids,
                failed_ids.iter().map(|&id| TweetId(id)).collect()
            );
            let scenario = evaluate_tweets(world(root_fails));
            let t0 = tokio::time::Instant::now();
            let evaluated = scenario.evaluate(&requested(&[1, 2, 3, 4])).await;
            assert_eq!(arms(&evaluated.outcomes), [Arm::Complete; 4]);
            let sources = scenario.sources();
            let root_selects: Vec<(Duration, Vec<u64>)> = sources
                .selects()
                .into_iter()
                .zip(sources.starts(Source::Flock))
                .filter_map(|(queries, started)| {
                    let query = queries.iter().find(|q| q.graph == Graph::Follows)?;
                    (query.direction == EdgeDirection::Reverse)
                        .then(|| (started - t0, query.destination_ids.clone()))
                })
                .collect();
            assert_eq!(root_selects, [(ms(0), vec![90])]);
            assert_eq!(
                sources.keys(Source::TesConversationControl),
                [vec![1, 2, 3, 4], vec![5, 6]]
            );
        }
    }

    #[tokio::test]
    async fn the_source_of_a_failed_retweet_changes_no_verdict() {
        let world = || retweet_world(true).fail_key(Source::TesTweet, 4);
        let judged = |world: InMemorySources| async move {
            let scenario = evaluate_tweets(world);
            let evaluated = scenario.evaluate(&requested(&[1, 2, 3, 4])).await;
            assert_eq!(
                scenario.sources().keys(Source::TesPureCoreData),
                [vec![1, 2, 3, 4], vec![5, 6]]
            );
            evaluated.evaluations()
        };
        let healthy = judged(world()).await;
        assert_eq!(healthy[3], Evaluation::Failed(Lookup::Tweet));
        for source_side in [
            world().user(60, suspended()),
            world().fail_key(Source::TesPureCoreData, 6),
            world().fail_key(Source::GizmoduckAuthor, 60),
            world().fail_edge(Graph::Blocks, 70),
        ] {
            assert_eq!(judged(source_side).await, healthy);
        }
    }

    #[tokio::test]
    async fn a_follower_of_the_source_author_sees_its_fosnr_notice_on_any_retweet() {
        let fosnr = vf_pb::SafetyLabelMap {
            labels: HashMap::from([(
                SafetyLabelType::FOSNR_ABUSE_INSULTS.0,
                vf_pb::SafetyLabel::default(),
            )]),
        };
        let sources = InMemorySources::default()
            .retweet(1, 10, 5, Some(20))
            .tweet(5, 20)
            .labels(5, fosnr)
            .authors(&[10, 20])
            .edge(Graph::Follows, 1, 20);
        let evaluated = evaluate_tweets(sources).evaluate(&requested(&[1])).await;
        assert_eq!(
            evaluated.evaluations(),
            [Evaluation::complete(noticed(
                true,
                false,
                "fosnr_abuse_insults_follower/soft_intervention/abuse"
            ))]
        );
    }

    #[tokio::test]
    async fn a_viewer_removed_from_the_community_is_limited_on_any_retweet_of_its_post() {
        let limits = ClientCapability {
            community_viewer_removed_limits: true,
            ..ClientCapability::default()
        };
        let world = || {
            InMemorySources::default()
                .retweet(1, 10, 5, Some(20))
                .tweet(5, 20)
                .community_post(5, 500)
                .authors(&[10, 20])
                .removed_from(500)
        };
        let evaluated = evaluate_tweets(world())
            .client(limits)
            .evaluate(&requested(&[1, 5]))
            .await;
        let removed_limit = Evaluation::complete(limited(
            LimitedEngagementReason::CommunityTweetViewerRemoved,
            "community_tweet_viewer_removed/limited_engagement",
        ));
        assert_eq!(
            evaluated.evaluations(),
            [removed_limit.clone(), removed_limit]
        );

        let unrequested_source = evaluate_tweets(world()).client(limits);
        unrequested_source.evaluate(&requested(&[1])).await;
        assert_eq!(
            unrequested_source
                .sources()
                .keys(Source::CommunityViewerRemoved),
            [vec![500]]
        );
    }

    #[tokio::test(start_paused = true)]
    async fn a_root_follows_the_viewer_by_the_select_that_asked_it() {
        use ConversationControlArm::{MyNetwork, Subscribers};
        let scenario = evaluate_tweets(
            retweet_world(true)
                .control(2, control(MyNetwork, 90, &[]))
                .control(5, control(Subscribers, 90, &[]))
                .edge(Graph::Follows, 90, 1)
                .key_latency(Source::TesConversationControl, 2, Duration::from_millis(30)),
        );
        let evaluated = scenario.evaluate(&requested(&[1, 2, 3, 4])).await;
        assert!(evaluated
            .evaluations()
            .iter()
            .all(|evaluation| matches!(evaluation, Evaluation::Complete { .. })));
        let sources = scenario.sources();
        let root_selects: Vec<String> = batches(sources)
            .lines()
            .filter(|line| line.contains("super_follows-fwd"))
            .map(str::to_owned)
            .collect();
        assert_eq!(
            root_selects,
            [
                "Flock follows-rev[] super_follows-fwd[90] blocks-rev[90]",
                "Flock follows-rev[90] super_follows-fwd[] blocks-rev[]",
            ]
        );
        assert!(sources.keys(Source::Wingman).is_empty());
    }

    #[tokio::test]
    async fn a_fetched_source_whose_conversation_root_blocks_the_viewer_limits_its_retweet() {
        let scenario = evaluate_tweets(
            InMemorySources::default()
                .retweet(5, 40, 2, Some(20))
                .pure_core(
                    2,
                    PureCoreData {
                        author_id: 20,
                        conversation_id: Some(100),
                        in_reply_to_tweet_id: Some(101),
                        in_reply_to_user_id: Some(99),
                        ..Default::default()
                    },
                )
                .tweet(100, 30)
                .authors(&[20, 40])
                .edge(Graph::Blocks, 30, 1),
        );
        let evaluated = scenario.evaluate(&requested(&[5])).await;
        assert_eq!(
            evaluated.evaluations(),
            [Evaluation::complete(limited(
                LimitedEngagementReason::RootAuthorBlockedViewer,
                "blocked_viewer/limited_engagement/root_author_blocked_viewer",
            ))]
        );
        let sources = scenario.sources();
        assert_eq!(
            batches(sources),
            "\
TesPureCoreData [[5], [2]]
TesTweet [[5], [2]]
TesConversationControl [[5], [2]]
SafetyLabels [[5], [2]]
GizmoduckViewer [[1]]
GizmoduckAuthor [[20, 40]]
Flock follows-fwd[20, 40] blocks-rev[20, 40]
Flock blocks-rev[30]"
        );
        assert_eq!(sources.keys(Source::TesConversationRoot), [vec![100]]);
    }

    #[tokio::test]
    async fn each_request_kind_labels_its_phase_samples() {
        use crate::metrics::RetweetSources::{Fetched, InBatch, NoSource};
        let scenario = evaluate_tweets(retweet_world(true));
        for (tweet_ids, expected) in [
            (&[2][..], NoSource),
            (&[1, 2, 3, 5], InBatch),
            (&[1, 2, 3, 4], Fetched),
        ] {
            let evaluated = scenario.evaluate(&requested(tweet_ids)).await;
            assert_eq!(evaluated.retweet_sources, expected, "{tweet_ids:?}");
        }
        let plain = scenario.rpc(Rpc::FilterTweets);
        let evaluated = plain.evaluate(&requested(&[1, 2, 3, 4])).await;
        assert_eq!(evaluated.retweet_sources, NoSource);
        assert_eq!(<&str>::from(NoSource), "none");
    }
}
