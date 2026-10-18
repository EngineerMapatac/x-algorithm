use std::collections::HashMap;
use std::time::{Duration, Instant};

use tonic::{Request, Status};
use xai_stats_receiver::{global_stats_receiver, HistogramBuckets, StatsReceiverExt};

use crate::caller_identity::{self, Endpoint};
use crate::filter_tweets::parse_grpc_timeout;
use crate::hydration::{Cause, Hydrators, Lookup, INBOUND_ALLOWANCE};
use crate::models::{Evaluation, Verdict};
use crate::rules::SafetyLevel;
use crate::treatment;

const REQUESTS: &str = "filter_tweets_requests";
const LATENCY_MS: &str = "filter_tweets_latency_ms";
pub(crate) const BATCH_SIZE: &str = "filter_tweets_batch_size";
const VERDICTS: &str = "filter_tweets_verdicts";
const VERDICTS_BY_RULE: &str = "filter_tweets_verdicts_by_rule";
const EVALUATED_CANDIDATES: &str = "filter_tweets_evaluated_candidates";
const FAIL_OPEN_DEFAULTS_NONE: &str = "none";
const UNRESOLVED_TWEETS: &str = "vf_unresolved_tweets";
const LOGGED_OUT_VIEWER: &str = "filter_tweets_logged_out_viewer";
const VIEWER_ID_NORMALIZED: &str = "filter_tweets_viewer_id_normalized";
const PHASE_MS: &str = "filter_tweets_phase_ms";
const DEADLINE: &str = "filter_tweets_deadline";
const DEADLINE_REMAINING_MS: &str = "filter_tweets_deadline_remaining_ms";
const DEADLINE_OVERRUN_MS: &str = "filter_tweets_deadline_overrun_ms";
const EVALUATE_TWEETS_REQUESTS: &str = "evaluate_tweets_requests";
const EVALUATE_TWEETS_LATENCY_MS: &str = "evaluate_tweets_latency_ms";
const EVALUATE_TWEETS_DEADLINE: &str = "evaluate_tweets_deadline";
const LOOKUP_REQUESTS: &str = "safety_labels_lookup_requests";
const LOOKUP_LATENCY_MS: &str = "safety_labels_lookup_latency_ms_vm";
const LOOKUP_DEADLINE: &str = "safety_labels_lookup_deadline";
const REQUESTS_BY_CALLER: &str = "vf_requests_by_caller";
const LATENCY_BY_CALLER_MS: &str = "vf_request_latency_by_caller_ms";
const UNKNOWN_IDENTITY: &str = "unknown";

#[derive(Clone, Copy, Debug, PartialEq, Eq, strum::IntoStaticStr)]
#[strum(serialize_all = "snake_case")]
pub enum Rpc {
    FilterTweets,
    EvaluateTweets,
}

#[derive(Clone, Copy, strum::IntoStaticStr)]
#[strum(serialize_all = "snake_case")]
pub(crate) enum RequestOutcome {
    Started,
    Cancelled,
    Success,
    Failure,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, strum::IntoStaticStr)]
#[strum(serialize_all = "snake_case")]
pub(crate) enum DeadlineOutcome {
    Absent,
    Within(Duration),
    Overrun(Duration),
    CancelledEarly,
    CancelledAtDeadline,
    CancelledNoDeadline,
}

impl DeadlineOutcome {
    pub(crate) fn of(
        outcome: RequestOutcome,
        grpc_timeout: Option<Duration>,
        elapsed: Duration,
    ) -> Option<Self> {
        match (outcome, grpc_timeout) {
            (RequestOutcome::Started | RequestOutcome::Failure, _) => None,
            (RequestOutcome::Success, None) => Some(Self::Absent),
            (RequestOutcome::Cancelled, None) => Some(Self::CancelledNoDeadline),
            (RequestOutcome::Success, Some(deadline)) => {
                Some(match deadline.checked_sub(elapsed) {
                    Some(remaining) => Self::Within(remaining),
                    None => Self::Overrun(elapsed - deadline),
                })
            }
            (RequestOutcome::Cancelled, Some(deadline)) => {
                Some(if elapsed + INBOUND_ALLOWANCE < deadline {
                    Self::CancelledEarly
                } else {
                    Self::CancelledAtDeadline
                })
            }
        }
    }
}

#[derive(Clone, Copy, strum::IntoStaticStr)]
#[strum(serialize_all = "snake_case")]
pub(crate) enum Stage {
    Hydration,
    PostHydration,
}

pub(crate) fn record_viewer_state(raw: Option<u64>, normalized: Option<u64>) {
    if normalized.is_some() {
        return;
    }
    incr(LOGGED_OUT_VIEWER, &[], 1);
    if raw.is_some() {
        incr(VIEWER_ID_NORMALIZED, &[], 1);
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct AggregatedVerdicts {
    pub mix: HashMap<&'static str, u64>,
    pub by_rule: HashMap<(&'static str, &'static str), u64>,
}

pub(crate) fn aggregate_verdicts<'a>(
    verdicts: impl IntoIterator<Item = &'a Verdict>,
) -> AggregatedVerdicts {
    let mut out = AggregatedVerdicts::default();
    for verdict in verdicts {
        *out.mix.entry(treatment::metric_label(verdict)).or_default() += 1;
        for row in treatment::decided_rows(verdict) {
            *out.by_rule.entry(row).or_default() += 1;
        }
    }
    out
}

pub(crate) fn record_verdicts<'a>(
    rpc: Rpc,
    safety_level: SafetyLevel,
    verdicts: impl IntoIterator<Item = &'a Verdict>,
) {
    let aggregated = aggregate_verdicts(verdicts);
    let level = <&str>::from(safety_level);
    let rpc = rpc.into();
    for (action, count) in &aggregated.mix {
        incr_nonzero(
            VERDICTS,
            &[("action", action), ("safety_level", level), ("rpc", rpc)],
            *count,
        );
    }
    for ((rule, action), count) in &aggregated.by_rule {
        incr_nonzero(
            VERDICTS_BY_RULE,
            &[
                ("rule", rule),
                ("action", action),
                ("safety_level", level),
                ("rpc", rpc),
            ],
            *count,
        );
    }
}

pub(crate) fn record_unresolved<'a>(
    rpc: Rpc,
    safety_level: SafetyLevel,
    evaluations: impl IntoIterator<Item = &'a Evaluation>,
) {
    let mut counts: HashMap<(Lookup, Cause), u64> = HashMap::new();
    for evaluation in evaluations {
        let unresolved = match *evaluation {
            Evaluation::NotFound(lookup) => (lookup, Cause::NotFound),
            Evaluation::Failed(lookup) => (lookup, Cause::Failed),
            Evaluation::Complete { .. } | Evaluation::Partial { .. } => continue,
        };
        *counts.entry(unresolved).or_default() += 1;
    }
    let level = <&str>::from(safety_level);
    let rpc = rpc.into();
    for ((lookup, cause), count) in counts {
        incr(
            UNRESOLVED_TWEETS,
            &[
                ("lookup", lookup.into()),
                ("cause", cause.into()),
                ("rpc", rpc),
                ("safety_level", level),
            ],
            count,
        );
    }
}

pub(crate) fn record_fail_open_defaults(
    rpc: Rpc,
    safety_level: SafetyLevel,
    fail_open_defaults: impl IntoIterator<Item = Hydrators>,
) {
    let mut on_none = 0;
    let mut on_failed: HashMap<Hydrators, u64> = HashMap::new();
    for nodes in fail_open_defaults {
        if nodes.is_empty() {
            on_none += 1;
        } else {
            *on_failed.entry(nodes).or_default() += 1;
        }
    }
    let level = <&str>::from(safety_level);
    let rpc = rpc.into();
    incr_nonzero(
        EVALUATED_CANDIDATES,
        &[
            ("fail_open_defaults", FAIL_OPEN_DEFAULTS_NONE),
            ("safety_level", level),
            ("rpc", rpc),
        ],
        on_none,
    );
    for (nodes, count) in on_failed {
        let nodes: Vec<&str> = nodes.iter().map(<&str>::from).collect();
        incr(
            EVALUATED_CANDIDATES,
            &[
                ("fail_open_defaults", &nodes.join("+")),
                ("safety_level", level),
                ("rpc", rpc),
            ],
            count,
        );
    }
}

fn families(endpoint: Endpoint) -> (&'static str, &'static str, Option<&'static str>) {
    match endpoint {
        Endpoint::FilterTweets => (REQUESTS, LATENCY_MS, Some(DEADLINE)),
        Endpoint::EvaluateTweets => (
            EVALUATE_TWEETS_REQUESTS,
            EVALUATE_TWEETS_LATENCY_MS,
            Some(EVALUATE_TWEETS_DEADLINE),
        ),
        Endpoint::GetSafetyLabels => (LOOKUP_REQUESTS, LOOKUP_LATENCY_MS, Some(LOOKUP_DEADLINE)),
    }
}

pub(crate) async fn observe_request<R, T, F>(
    endpoint: Endpoint,
    request: Request<R>,
    handle: impl FnOnce(Request<R>, Option<Duration>) -> F,
) -> Result<T, Status>
where
    F: Future<Output = Result<T, Status>>,
{
    let grpc_timeout = parse_grpc_timeout(request.metadata());
    let (requests, _, _) = families(endpoint);
    incr(requests, &[("outcome", RequestOutcome::Started.into())], 1);
    let mut guard = RequestMetricsGuard {
        endpoint,
        caller_identity: caller_identity::identity(&request),
        grpc_timeout,
        start: Instant::now(),
        outcome: RequestOutcome::Cancelled,
    };
    let result = handle(request, grpc_timeout).await;
    guard.outcome = if result.is_ok() {
        RequestOutcome::Success
    } else {
        RequestOutcome::Failure
    };
    result
}

struct RequestMetricsGuard {
    endpoint: Endpoint,
    caller_identity: Option<String>,
    grpc_timeout: Option<Duration>,
    start: Instant,
    outcome: RequestOutcome,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, strum::IntoStaticStr)]
#[strum(serialize_all = "snake_case")]
pub(crate) enum RetweetSources {
    #[strum(to_string = "none")]
    NoSource,
    InBatch,
    Fetched,
}

pub(crate) fn record_phase(rpc: Rpc, stage: Stage, sources: RetweetSources, elapsed: Duration) {
    observe_vm(
        PHASE_MS,
        &[
            ("stage", stage.into()),
            ("rpc", rpc.into()),
            ("sources", sources.into()),
        ],
        millis(elapsed),
    );
}

fn millis(d: Duration) -> f64 {
    d.as_secs_f64() * 1000.0
}

impl RequestMetricsGuard {
    fn emit(&self, sr: &dyn StatsReceiverExt, elapsed: Duration) {
        let (requests, latency_ms, deadline) = families(self.endpoint);
        let outcome: &str = self.outcome.into();
        sr.incr(requests, &[("outcome", outcome)], 1);
        sr.observe_vm(latency_ms, &[], millis(elapsed));
        let caller = (
            "caller_identity",
            self.caller_identity.as_deref().unwrap_or(UNKNOWN_IDENTITY),
        );
        let rpc = ("rpc", <&str>::from(self.endpoint));
        sr.incr(REQUESTS_BY_CALLER, &[caller, rpc, ("outcome", outcome)], 1);
        if let RequestOutcome::Success = self.outcome {
            sr.observe_vm(LATENCY_BY_CALLER_MS, &[caller, rpc], millis(elapsed));
        }
        let Some(deadline) = deadline else {
            return;
        };
        let Some(deadline_outcome) = DeadlineOutcome::of(self.outcome, self.grpc_timeout, elapsed)
        else {
            return;
        };
        sr.incr(deadline, &[("outcome", deadline_outcome.into())], 1);
        if !matches!(self.endpoint, Endpoint::FilterTweets) {
            return;
        }
        match deadline_outcome {
            DeadlineOutcome::Within(remaining) => {
                sr.observe_vm(DEADLINE_REMAINING_MS, &[], millis(remaining));
            }
            DeadlineOutcome::Overrun(overrun) => {
                sr.observe_vm(DEADLINE_OVERRUN_MS, &[], millis(overrun));
            }
            DeadlineOutcome::Absent
            | DeadlineOutcome::CancelledEarly
            | DeadlineOutcome::CancelledAtDeadline
            | DeadlineOutcome::CancelledNoDeadline => {}
        }
    }
}

impl Drop for RequestMetricsGuard {
    fn drop(&mut self) {
        if let Some(sr) = global_stats_receiver() {
            self.emit(&*sr, self.start.elapsed());
        }
    }
}

pub(crate) fn record_batch_size(metric: &str, size: usize) {
    observe(metric, &[], size as f64, HistogramBuckets::Bucket50To500);
}

pub(crate) fn incr_nonzero(metric: &str, labels: &[(&str, &str)], count: u64) {
    if count > 0 {
        incr(metric, labels, count);
    }
}

fn incr(metric: &str, labels: &[(&str, &str)], count: u64) {
    if let Some(sr) = global_stats_receiver() {
        sr.incr(metric, labels, count);
    }
}

fn observe(metric: &str, labels: &[(&str, &str)], value: f64, buckets: HistogramBuckets) {
    if let Some(sr) = global_stats_receiver() {
        sr.observe(metric, labels, value, buckets);
    }
}

fn observe_vm(metric: &str, labels: &[(&str, &str)], value: f64) {
    if let Some(sr) = global_stats_receiver() {
        sr.observe_vm(metric, labels, value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{
        Decided, DropReason, LimitedEngagement, LimitedEngagementReason, MediaInterstitial,
        MediaRestriction, Withholding,
    };
    use std::fs;
    use std::path::Path;
    use xai_visibility_filtering::models::FilteredReason;
    use xai_visibility_filtering::vf_client::FILTER_TWEETS_LATENCY_VM_METRIC;
    use xai_x_thrift::action::InterstitialReason;

    fn allow() -> Verdict {
        Verdict::Shown {
            notice: None,
            media: None,
            engagement: None,
        }
    }

    fn drop_by(rule: &'static str) -> Verdict {
        Verdict::Withheld(Decided {
            value: Withholding::Drop(DropReason::Legacy(FilteredReason::UnspecifiedReason)),
            by: rule,
        })
    }

    fn limit_by(rule: &'static str) -> Decided<LimitedEngagement> {
        Decided {
            value: LimitedEngagement::new(LimitedEngagementReason::ConversationControl),
            by: rule,
        }
    }

    #[test]
    fn dashboard_generator_pins_the_evaluated_candidates_unresolved_tweets_and_deadline_metrics() {
        let cargo = concat!(env!("CARGO_MANIFEST_DIR"), "/scripts/dashboard.py");
        let ws = "crates/x-product/xai-visibility-filtering-service/scripts/dashboard.py";
        let path = if Path::new(cargo).exists() { cargo } else { ws };
        let dashboard = fs::read_to_string(path).unwrap_or_else(|e| panic!("read {path}: {e}"));
        assert!(dashboard.contains(&format!(
            "FT_EVALUATED_CANDIDATES_METRIC = \"{EVALUATED_CANDIDATES}\""
        )));
        assert!(dashboard.contains(&format!(
            "FT_FAIL_OPEN_DEFAULTS_FILTER = 'fail_open_defaults!~\"{FAIL_OPEN_DEFAULTS_NONE}|\"'"
        )));
        assert!(dashboard.contains(&format!(
            "UNRESOLVED_TWEETS_METRIC = \"{UNRESOLVED_TWEETS}\""
        )));
        assert!(dashboard.contains(&format!("FT_DEADLINE_METRIC = \"{DEADLINE}\"")));
        assert!(dashboard.contains(&format!(
            "ET_REQUESTS_METRIC = \"{EVALUATE_TWEETS_REQUESTS}\""
        )));
        assert!(dashboard.contains(&format!(
            "ET_LATENCY_METRIC = \"{EVALUATE_TWEETS_LATENCY_MS}\""
        )));
        assert!(dashboard.contains(&format!(
            "ET_DEADLINE_METRIC = \"{EVALUATE_TWEETS_DEADLINE}\""
        )));
        assert!(dashboard.contains(&format!(
            "LOOKUP_LATENCY_VM_METRIC = \"{LOOKUP_LATENCY_MS}\""
        )));
        assert!(dashboard.contains(&format!("LOOKUP_DEADLINE_METRIC = \"{LOOKUP_DEADLINE}\"")));
        assert!(dashboard.contains(&format!(
            "REQUESTS_BY_CALLER_METRIC = \"{REQUESTS_BY_CALLER}\""
        )));
        assert!(dashboard.contains(&format!(
            "VF_LATENCY_BY_CALLER_METRIC = \"{LATENCY_BY_CALLER_MS}\""
        )));
        let [success, failure, cancelled] = [
            RequestOutcome::Success,
            RequestOutcome::Failure,
            RequestOutcome::Cancelled,
        ]
        .map(<&str>::from);
        assert!(dashboard.contains(&format!(
            "REQUESTS_BY_CALLER_SUCCESS_FILTER = 'outcome=\"{success}\"'"
        )));
        assert!(dashboard.contains(&format!(
            "REQUESTS_BY_CALLER_NON_SUCCESS_FILTER = 'outcome=~\"{failure}|{cancelled}\"'"
        )));
        assert!(dashboard.contains(&format!(
            "FT_DEADLINE_REMAINING_METRIC = \"{DEADLINE_REMAINING_MS}\""
        )));
        assert!(dashboard.contains(&format!(
            "VF_CLIENT_FT_LATENCY_VM_METRIC = \"{FILTER_TWEETS_LATENCY_VM_METRIC}\""
        )));
        for outcome in [
            DeadlineOutcome::CancelledEarly,
            DeadlineOutcome::CancelledAtDeadline,
            DeadlineOutcome::CancelledNoDeadline,
            DeadlineOutcome::Overrun(Duration::ZERO),
        ] {
            let outcome: &str = outcome.into();
            assert!(dashboard.contains(&format!("'outcome=\"{outcome}\"'")));
        }
    }

    #[derive(Default)]
    struct Recorded(std::sync::Mutex<Vec<String>>);

    impl Recorded {
        fn push(&self, kind: &str, name: &str, scopes: &[(&str, &str)]) {
            let labels: Vec<String> = scopes.iter().map(|(k, v)| format!("{k}={v}")).collect();
            let line = format!("{kind} {name}{{{}}}", labels.join(","));
            self.0.lock().unwrap().push(line);
        }
    }

    impl StatsReceiverExt for Recorded {
        fn incr(&self, name: &str, scopes: &[(&str, &str)], _: u64) {
            self.push("incr", name, scopes);
        }
        fn observe(&self, _: &str, _: &[(&str, &str)], _: f64, _: HistogramBuckets) {}
        fn observe_expo(&self, _: &str, _: &[(&str, &str)], _: f64) {}
        fn observe_vm(&self, name: &str, scopes: &[(&str, &str)], _: f64) {
            self.push("observe_vm", name, scopes);
        }
        fn gauge(&self, _: &str, _: &[(&str, &str)], _: f64) {}
    }

    #[test]
    fn each_endpoint_emits_its_request_deadline_and_caller_series() {
        let recorded = Recorded::default();
        for endpoint in [
            Endpoint::FilterTweets,
            Endpoint::EvaluateTweets,
            Endpoint::GetSafetyLabels,
        ] {
            for outcome in [
                RequestOutcome::Success,
                RequestOutcome::Failure,
                RequestOutcome::Cancelled,
            ] {
                let guard = RequestMetricsGuard {
                    endpoint,
                    caller_identity: None,
                    grpc_timeout: Some(Duration::from_millis(400)),
                    start: Instant::now(),
                    outcome,
                };
                guard.emit(&recorded, Duration::from_millis(100));
            }
        }
        assert_eq!(
            recorded.0.into_inner().unwrap(),
            [
                "incr filter_tweets_requests{outcome=success}",
                "observe_vm filter_tweets_latency_ms{}",
                "incr vf_requests_by_caller{caller_identity=unknown,rpc=filter_tweets,outcome=success}",
                "observe_vm vf_request_latency_by_caller_ms{caller_identity=unknown,rpc=filter_tweets}",
                "incr filter_tweets_deadline{outcome=within}",
                "observe_vm filter_tweets_deadline_remaining_ms{}",
                "incr filter_tweets_requests{outcome=failure}",
                "observe_vm filter_tweets_latency_ms{}",
                "incr vf_requests_by_caller{caller_identity=unknown,rpc=filter_tweets,outcome=failure}",
                "incr filter_tweets_requests{outcome=cancelled}",
                "observe_vm filter_tweets_latency_ms{}",
                "incr vf_requests_by_caller{caller_identity=unknown,rpc=filter_tweets,outcome=cancelled}",
                "incr filter_tweets_deadline{outcome=cancelled_early}",
                "incr evaluate_tweets_requests{outcome=success}",
                "observe_vm evaluate_tweets_latency_ms{}",
                "incr vf_requests_by_caller{caller_identity=unknown,rpc=evaluate_tweets,outcome=success}",
                "observe_vm vf_request_latency_by_caller_ms{caller_identity=unknown,rpc=evaluate_tweets}",
                "incr evaluate_tweets_deadline{outcome=within}",
                "incr evaluate_tweets_requests{outcome=failure}",
                "observe_vm evaluate_tweets_latency_ms{}",
                "incr vf_requests_by_caller{caller_identity=unknown,rpc=evaluate_tweets,outcome=failure}",
                "incr evaluate_tweets_requests{outcome=cancelled}",
                "observe_vm evaluate_tweets_latency_ms{}",
                "incr vf_requests_by_caller{caller_identity=unknown,rpc=evaluate_tweets,outcome=cancelled}",
                "incr evaluate_tweets_deadline{outcome=cancelled_early}",
                "incr safety_labels_lookup_requests{outcome=success}",
                "observe_vm safety_labels_lookup_latency_ms_vm{}",
                "incr vf_requests_by_caller{caller_identity=unknown,rpc=get_safety_labels,outcome=success}",
                "observe_vm vf_request_latency_by_caller_ms{caller_identity=unknown,rpc=get_safety_labels}",
                "incr safety_labels_lookup_deadline{outcome=within}",
                "incr safety_labels_lookup_requests{outcome=failure}",
                "observe_vm safety_labels_lookup_latency_ms_vm{}",
                "incr vf_requests_by_caller{caller_identity=unknown,rpc=get_safety_labels,outcome=failure}",
                "incr safety_labels_lookup_requests{outcome=cancelled}",
                "observe_vm safety_labels_lookup_latency_ms_vm{}",
                "incr vf_requests_by_caller{caller_identity=unknown,rpc=get_safety_labels,outcome=cancelled}",
                "incr safety_labels_lookup_deadline{outcome=cancelled_early}",
            ]
        );
    }

    #[test]
    fn a_cancel_counts_whether_the_caller_hung_up_before_its_deadline() {
        let deadline = Some(Duration::from_millis(400));
        let of = |outcome, ms| DeadlineOutcome::of(outcome, deadline, Duration::from_millis(ms));
        assert_eq!(
            of(RequestOutcome::Cancelled, 389),
            Some(DeadlineOutcome::CancelledEarly)
        );
        assert_eq!(
            of(RequestOutcome::Cancelled, 390),
            Some(DeadlineOutcome::CancelledAtDeadline)
        );
        assert_eq!(
            of(RequestOutcome::Cancelled, 401),
            Some(DeadlineOutcome::CancelledAtDeadline)
        );
        assert_eq!(
            DeadlineOutcome::of(RequestOutcome::Cancelled, None, Duration::ZERO),
            Some(DeadlineOutcome::CancelledNoDeadline)
        );
        assert_eq!(
            DeadlineOutcome::of(RequestOutcome::Success, None, Duration::ZERO),
            Some(DeadlineOutcome::Absent)
        );
        assert_eq!(
            of(RequestOutcome::Success, 400),
            Some(DeadlineOutcome::Within(Duration::ZERO))
        );
        assert_eq!(
            of(RequestOutcome::Success, 401),
            Some(DeadlineOutcome::Overrun(Duration::from_millis(1)))
        );
        assert_eq!(of(RequestOutcome::Failure, 5), None);
    }

    #[test]
    fn each_filled_slot_counts_its_own_row() {
        let a = allow();
        let b = allow();
        let d = drop_by("nsfw_media");
        let both = Verdict::Shown {
            notice: None,
            media: Some(Decided {
                value: MediaRestriction::MediaInterstitial(MediaInterstitial {
                    legacy: FilteredReason::ContainNsfwMedia,
                    reason: InterstitialReason::Sensitive(true),
                    prompt: None,
                }),
                by: "nsfw_media",
            }),
            engagement: Some(limit_by("conversation_control")),
        };
        let limit_only = Verdict::Shown {
            notice: None,
            media: None,
            engagement: Some(limit_by("conversation_control")),
        };
        let aggregated = aggregate_verdicts([&a, &b, &d, &both, &limit_only]);

        assert_eq!(aggregated.mix.get("allow"), Some(&2));
        assert_eq!(aggregated.mix.get("drop"), Some(&1));
        assert_eq!(aggregated.mix.get("tweet_interstitial"), Some(&1));
        assert_eq!(aggregated.mix.get("limited_engagement"), Some(&1));
        assert_eq!(aggregated.mix.values().sum::<u64>(), 5);
        assert_eq!(aggregated.by_rule.values().sum::<u64>(), 4);
        assert_eq!(aggregated.by_rule.get(&("nsfw_media", "drop")), Some(&1));
        assert_eq!(
            aggregated.by_rule.get(&("nsfw_media", "interstitial")),
            Some(&1)
        );
        assert_eq!(
            aggregated
                .by_rule
                .get(&("conversation_control", "limited_engagement")),
            Some(&2)
        );
    }
}
