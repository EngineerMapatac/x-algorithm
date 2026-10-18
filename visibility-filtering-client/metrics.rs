use std::sync::Arc;
use std::time::{Duration, Instant};

use xai_stats_receiver::{StatsReceiverExt, global_stats_receiver};

use crate::tweet_safety_label::{BATCH_LATENCY_VM_METRIC, LATENCY_VM_METRIC};
use crate::vf_client::FILTER_TWEETS_LATENCY_VM_METRIC;

#[derive(Clone, Copy)]
pub(crate) enum ClientRpc {
    GetSafetyLabels,
    FilterTweets,
    EvaluateTweets,
}

struct Names {
    requests: &'static str,
    latency_ms: &'static str,
    batch_error: &'static str,
    batch_latency_ms: &'static str,
    failed_ids: Option<&'static str>,
}

impl ClientRpc {
    fn names(self) -> Names {
        match self {
            Self::GetSafetyLabels => Names {
                requests: "vf_client_get_safety_labels",
                latency_ms: LATENCY_VM_METRIC,
                batch_error: "vf_client_get_safety_labels_batch_error",
                batch_latency_ms: BATCH_LATENCY_VM_METRIC,
                failed_ids: Some("vf_client_get_safety_labels_failed_ids"),
            },
            Self::FilterTweets => Names {
                requests: "vf_client_filter_tweets",
                latency_ms: FILTER_TWEETS_LATENCY_VM_METRIC,
                batch_error: "vf_client_filter_tweets_batch_error",
                batch_latency_ms: "vf_client_filter_tweets_batch_latency_ms_vm",
                failed_ids: Some("vf_client_filter_tweets_failed_ids"),
            },
            Self::EvaluateTweets => Names {
                requests: "vf_client_evaluate_tweets",
                latency_ms: "vf_client_evaluate_tweets_latency_ms_vm",
                batch_error: "vf_client_evaluate_tweets_batch_error",
                batch_latency_ms: "vf_client_evaluate_tweets_batch_latency_ms_vm",
                failed_ids: None,
            },
        }
    }
}

fn error_code(code: tonic::Code) -> &'static str {
    match code {
        tonic::Code::DeadlineExceeded => "deadline_exceeded",
        tonic::Code::Cancelled => "cancelled",
        tonic::Code::Unavailable => "unavailable",
        _ => "other",
    }
}

fn millis(elapsed: Duration) -> f64 {
    elapsed.as_secs_f64() * 1000.0
}

pub(crate) struct RequestMetrics {
    rpc: ClientRpc,
    receiver: Option<Arc<dyn StatsReceiverExt>>,
    start: Instant,
    status: &'static str,
}

impl RequestMetrics {
    pub(crate) fn start(rpc: ClientRpc) -> Self {
        Self::with_receiver(rpc, global_stats_receiver())
    }

    pub(crate) fn with_receiver(
        rpc: ClientRpc,
        receiver: Option<Arc<dyn StatsReceiverExt>>,
    ) -> Self {
        Self {
            rpc,
            receiver,
            start: Instant::now(),
            status: "cancelled",
        }
    }

    pub(crate) fn batch(&self, elapsed: Duration, error: Option<tonic::Code>) {
        let Some(sr) = &self.receiver else {
            return;
        };
        let names = self.rpc.names();
        let result = match error {
            None => "success",
            Some(code) => {
                sr.incr(names.batch_error, &[("code", error_code(code))], 1);
                "error"
            }
        };
        sr.observe_vm(
            names.batch_latency_ms,
            &[("result", result)],
            millis(elapsed),
        );
    }

        pub(crate) fn answered(&mut self, failed_ids: usize) {
        self.status = if failed_ids > 0 {
            "degraded"
        } else {
            "completed"
        };
        let Some(sr) = &self.receiver else {
            return;
        };
        let names = self.rpc.names();
        sr.observe_vm(names.latency_ms, &[], millis(self.start.elapsed()));
        if failed_ids > 0
            && let Some(metric) = names.failed_ids
        {
            sr.incr(metric, &[], failed_ids as u64);
        }
    }

        pub(crate) fn failed(&mut self) {
        self.status = "failed";
        if let Some(sr) = &self.receiver {
            sr.observe_vm(
                self.rpc.names().latency_ms,
                &[],
                millis(self.start.elapsed()),
            );
        }
    }
}

impl Drop for RequestMetrics {
    fn drop(&mut self) {
        if let Some(sr) = &self.receiver {
            sr.incr(self.rpc.names().requests, &[("requests", self.status)], 1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::RecordingReceiver;

    fn series(rpc: ClientRpc, call: impl FnOnce(RequestMetrics)) -> Vec<String> {
        let sr = Arc::new(RecordingReceiver::default());
        call(RequestMetrics::with_receiver(rpc, Some(sr.clone())));
        sr.series()
    }

    fn completed(mut request: RequestMetrics) {
        request.batch(Duration::from_millis(5), None);
        request.answered(0);
    }

    fn degraded(mut request: RequestMetrics) {
        request.batch(Duration::from_millis(5), None);
        request.batch(Duration::from_millis(5), Some(tonic::Code::Unavailable));
        request.answered(50);
    }

    fn cancelled(request: RequestMetrics) {
        drop(request);
    }

    #[test]
    fn each_rpc_emits_these_series_when_completed_degraded_or_cancelled() {
        assert_eq!(
            series(ClientRpc::GetSafetyLabels, completed),
            [
                "incr vf_client_get_safety_labels|requests=completed",
                "observe_vm vf_client_get_safety_labels_batch_latency_ms_vm|result=success",
                "observe_vm vf_client_get_safety_labels_latency_ms_vm",
            ]
        );
        assert_eq!(
            series(ClientRpc::GetSafetyLabels, degraded),
            [
                "incr vf_client_get_safety_labels_batch_error|code=unavailable",
                "incr vf_client_get_safety_labels_failed_ids",
                "incr vf_client_get_safety_labels|requests=degraded",
                "observe_vm vf_client_get_safety_labels_batch_latency_ms_vm|result=error",
                "observe_vm vf_client_get_safety_labels_batch_latency_ms_vm|result=success",
                "observe_vm vf_client_get_safety_labels_latency_ms_vm",
            ]
        );
        assert_eq!(
            series(ClientRpc::GetSafetyLabels, cancelled),
            ["incr vf_client_get_safety_labels|requests=cancelled"]
        );
        assert_eq!(
            series(ClientRpc::FilterTweets, completed),
            [
                "incr vf_client_filter_tweets|requests=completed",
                "observe_vm vf_client_filter_tweets_batch_latency_ms_vm|result=success",
                "observe_vm vf_client_filter_tweets_latency_ms_vm",
            ]
        );
        assert_eq!(
            series(ClientRpc::FilterTweets, degraded),
            [
                "incr vf_client_filter_tweets_batch_error|code=unavailable",
                "incr vf_client_filter_tweets_failed_ids",
                "incr vf_client_filter_tweets|requests=degraded",
                "observe_vm vf_client_filter_tweets_batch_latency_ms_vm|result=error",
                "observe_vm vf_client_filter_tweets_batch_latency_ms_vm|result=success",
                "observe_vm vf_client_filter_tweets_latency_ms_vm",
            ]
        );
        assert_eq!(
            series(ClientRpc::FilterTweets, cancelled),
            ["incr vf_client_filter_tweets|requests=cancelled"]
        );
    }

    #[test]
    fn a_batch_error_counts_its_grpc_code() {
        let sr = Arc::new(RecordingReceiver::default());
        let request = RequestMetrics::with_receiver(ClientRpc::FilterTweets, Some(sr.clone()));
        for code in [
            tonic::Code::DeadlineExceeded,
            tonic::Code::Cancelled,
            tonic::Code::Unavailable,
            tonic::Code::Internal,
            tonic::Code::Unknown,
        ] {
            request.batch(Duration::ZERO, Some(code));
        }
        for (code, count) in [
            ("deadline_exceeded", 1),
            ("cancelled", 1),
            ("unavailable", 1),
            ("other", 2),
        ] {
            let key = format!("vf_client_filter_tweets_batch_error|code={code}");
            assert_eq!(sr.counter(&key), count, "{key}");
        }
    }
}
