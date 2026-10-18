// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 X.AI Corp.
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use lazy_static::lazy_static;
use prometheus::{
    Gauge, Histogram, HistogramVec, IntCounter, IntCounterVec, IntGauge, exponential_buckets,
    register_gauge, register_histogram, register_histogram_vec, register_int_counter,
    register_int_counter_vec, register_int_gauge,
};
use tonic::{Request, Response, Status};

use crate::admission::AdmissionController;

pub fn latency_buckets_ms() -> Vec<f64> {
    vec![
        1.0, 2.0, 4.0, 7.0, 10.0, 20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0, 90.0, 100.0, 115.0,
        130.0, 150.0, 175.0, 200.0, 225.0, 250.0, 350.0, 500.0, 1000.0, 2500.0, 10000.0,
    ]
}

lazy_static! {
    pub static ref NUM_REQUESTS: IntCounterVec = register_int_counter_vec!(
        "recsys_engine_num_requests",
        "Number of requests completed, labeled by terminal gRPC status code, the \
         caller (x-client-name; 'unknown' when absent), the caller's k8s \
         workload/deployment name (src_workload; 'unknown' when absent), and the \
         caller's source datacenter (src_dc; 'unknown' when absent)",
        &["grpc_status", "client", "workload", "src_dc"]
    )
    .unwrap();
    pub static ref NUM_REQUESTS_RECEIVED: IntCounter = register_int_counter!(
        "recsys_engine_num_requests_received",
        "Number of requests received by the recsys grpc engine"
    )
    .unwrap();
    pub static ref NUM_REQUESTS_REJECTED: IntCounterVec = register_int_counter_vec!(
        "recsys_engine_num_requests_rejected",
        "Number of requests rejected at the front door, labeled by the concrete \
         admission gate that fired (reason) and the caller (client, from \
         x-client-name; 'unknown' when absent). Reasons: checkpoint_loading \
         (reload in progress -- readiness is already down, the router should \
         have stopped sending), inflight_cap (max_inflight_requests semaphore), \
         deadline_admission (predicted completion exceeds the caller's \
         remaining deadline), lb_ticket (loadbalancer ticket not acquirable). \
         Existing sum()/rate() queries keep working; group by reason to see \
         which gate is rejecting.",
        &["reason", "client"]
    )
    .unwrap();
    pub static ref NUM_REQUESTS_SUCCEEDED: IntCounter = register_int_counter!(
        "recsys_engine_num_requests_succeeded",
        "Number of requests succeeded"
    )
    .unwrap();
    pub static ref NUM_REQUESTS_FAILED: IntCounter = register_int_counter!(
        "recsys_engine_num_requests_failed",
        "Number of requests failed"
    )
    .unwrap();
    pub static ref NUM_REQUESTS_CANCELED: IntCounter = register_int_counter!(
        "recsys_engine_num_requests_canceled",
        "Number of requests canceled"
    )
    .unwrap();
    pub static ref REQUEST_LATENCY_MS: HistogramVec = {
        let buckets = latency_buckets_ms();
        register_histogram_vec!(
            "recsys_engine_request_latency_ms",
            "Total request latency in ms, labeled by terminal gRPC status code \
             and the caller's source datacenter (src_dc; 'unknown' when absent)",
            &["grpc_status", "src_dc"],
            buckets
        )
        .unwrap()
    };
    pub static ref NOT_READY_ARRIVAL_SECONDS: HistogramVec = register_histogram_vec!(
        "recsys_engine_not_ready_arrival_seconds",
        "Time since this pod dropped readiness (set_ready(false): reload drain, \
         graceful restart, or SIGTERM) at which a request STILL arrived, labeled \
         by caller. Directly measures how long the routing layer (EndpointSlice \
         -> Contour proxy / kube-discovery EDS / xDS clients) keeps sending \
         after readiness drops -- the safe graceful_restart_timeout is above \
         this distribution's tail. Arrivals below the drain budget are served \
         with the old model; arrivals after it hit the checkpoint_loading \
         reject. No samples while the pod is ready.",
        &["client"],
        vec![
            0.5, 1.0, 2.0, 3.0, 5.0, 7.5, 10.0, 15.0, 20.0, 30.0, 45.0, 60.0, 90.0, 120.0, 180.0
        ]
    )
    .unwrap();
    pub static ref BATCHING_TIMEOUT_DROP: IntCounter = register_int_counter!(
        "batching_timeout_drop",
        "Number of requests dropped due to enqueue timeout (queue full)"
    )
    .unwrap();
    pub static ref DEADLINE_SHED: IntCounterVec = register_int_counter_vec!(
        "recsys_engine_deadline_shed",
        "Number of requests shed at admission because predicted completion exceeds the deadline",
        &["client"]
    )
    .unwrap();
    pub static ref REQUESTS_WITH_GRPC_TIMEOUT: IntCounter = register_int_counter!(
        "recsys_engine_requests_with_grpc_timeout_header",
        "Requests that arrived carrying a grpc-timeout (client-set deadline) header"
    )
    .unwrap();
    pub static ref REQUESTS_WITHOUT_GRPC_TIMEOUT: IntCounter = register_int_counter!(
        "recsys_engine_requests_without_grpc_timeout_header",
        "Requests that arrived WITHOUT a grpc-timeout header (admission falls back to deadline_fallback_budget)"
    )
    .unwrap();
    pub static ref ITEMS_IN_FLIGHT: IntGauge = register_int_gauge!(
        "recsys_engine_items_in_flight",
        "Number of recsys items that are queued or being processed"
    )
    .unwrap();
    pub static ref ITEM_QUEUE_TIME: Histogram = {
        let buckets = latency_buckets_ms();
        register_histogram!(
            "recsys_engine_item_queue_time",
            "How long an item waited in the request queue before dequeue (ms)",
            buckets
        )
        .unwrap()
    };
    pub static ref WAITING_QUEUE_SIZE: IntGauge = register_int_gauge!(
        "recsys_engine_waiting_queue_size",
        "Number of recsys requests waiting in the queue for backend to process"
    )
    .unwrap();
    pub static ref NUM_REQUESTS_STALE_DROPPED: IntCounter = register_int_counter!(
        "recsys_engine_num_requests_stale_dropped",
        "Number of requests dropped because they sat in the queue longer than queue_max_staleness_ms"
    )
    .unwrap();
    pub static ref SEQUENCES_PROCESSED: IntCounter = register_int_counter!(
        "recsys_engine_sequences_processed_total",
        "Total sequences (rows) actually scored by the ranking engine (real, pre-padding)"
    )
    .unwrap();
    pub static ref TOKENS_PROCESSED: IntCounterVec = register_int_counter_vec!(
        "recsys_engine_tokens_processed_total",
        "Tokens processed by the ranking engine, split by kind (\"history\" vs \"candidate\"). \
         Real positions fed to the model, counted pre-padding: \"history\" sums the real \
         history count per row (<= history_seq_len), \"candidate\" sums the real candidate \
         count per row (capped at candidate_seq_len). Total tokens/s is `sum without (kind)`.",
        &["kind"]
    )
    .unwrap();
    pub static ref TOKENS_PADDED: IntCounter = register_int_counter!(
        "recsys_engine_tokens_padded_total",
        "Total padded tokens dispatched to the ranking model: scored rows x \
         (history_seq_len + candidate_seq_len). Denominator for token padding \
         efficiency (real recsys_engine_tokens_processed_total / this padded total)."
    )
    .unwrap();
    pub static ref TOKENS_PER_ROW: HistogramVec = register_histogram_vec!(
        "recsys_engine_tokens_per_row",
        "Per-row distribution of real (pre-padding) tokens fed to the ranking model, by kind \
         (\"history\" vs \"candidate\").",
        &["kind"],
        exponential_buckets(1.0, 2.0, 13).unwrap()
    )
    .unwrap();
    pub static ref CLIENT_SERVER_NETWORK_DELAY: HistogramVec = {
        let buckets = latency_buckets_ms();
        register_histogram_vec!(
            "recsys_engine_client_server_network_delay_ms",
            "Approximate client->server network delay (ms): wall-clock delta \
             between the caller's x-request-send-time-ms (stamped just before \
             the request hits the wire) and server handler entry. Labels: \
             client, src_dc.",
            &["client", "src_dc"],
            buckets
        )
        .unwrap()
    };
    pub static ref ADMISSION_BUDGET_REMAINING: HistogramVec = {
        let buckets = latency_buckets_ms();
        register_histogram_vec!(
            "recsys_engine_admission_budget_remaining_ms",
            "Remaining deadline budget (ms) at the admission point: the \
             caller's grpc-timeout (or the configured fallback) minus elapsed \
             handler time.",
            &["client"],
            buckets
        )
        .unwrap()
    };
    pub static ref QUEUE_BACKLOG: IntGauge = register_int_gauge!(
        "recsys_engine_queue_backlog",
        "Live request backlog (mpsc channel occupancy + staging buffer) — the q \
         deadline admission predicts against."
    )
    .unwrap();
    pub static ref ADMISSION_SERVICE_TIME_MS: Gauge = register_gauge!(
        "recsys_engine_admission_service_time_ms",
        "EWMA per-batch service time S (ms) used by deadline admission."
    )
    .unwrap();
    pub static ref ADMISSION_PREDICTED_ETA: Histogram = {
        let buckets = latency_buckets_ms();
        register_histogram!(
            "recsys_engine_admission_predicted_eta_ms",
            "Predicted completion time (ms) for arriving requests. \
             loop: (ceil((q+1)/B)+depth)*S + post. \
             pipeline: q*S/B + P + post.",
            buckets
        )
        .unwrap()
    };
    pub static ref ADMISSION_PIPELINE_TIME_MS: Gauge = register_gauge!(
        "recsys_engine_admission_pipeline_time_ms",
        "EWMA request sojourn residual P (ms) used by deadline admission."
    )
    .unwrap();
}

static NOT_READY_SINCE_EPOCH_MS: AtomicU64 = AtomicU64::new(0);

fn epoch_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

pub fn arm_not_ready_probe() {
    let _ = NOT_READY_SINCE_EPOCH_MS.compare_exchange(
        0,
        epoch_millis(),
        Ordering::Relaxed,
        Ordering::Relaxed,
    );
}

pub fn disarm_not_ready_probe() {
    NOT_READY_SINCE_EPOCH_MS.store(0, Ordering::Relaxed);
}

fn not_ready_elapsed_seconds() -> Option<f64> {
    let since = NOT_READY_SINCE_EPOCH_MS.load(Ordering::Relaxed);
    if since == 0 {
        return None;
    }
    Some(epoch_millis().saturating_sub(since) as f64 / 1000.0)
}

pub fn record_not_ready_arrival(client: &str) {
    if let Some(elapsed_s) = not_ready_elapsed_seconds() {
        NOT_READY_ARRIVAL_SECONDS
            .with_label_values(&[client])
            .observe(elapsed_s);
    }
}

pub fn grpc_status_label(code: tonic::Code) -> &'static str {
    match code {
        tonic::Code::Ok => "ok",
        tonic::Code::Cancelled => "cancelled",
        tonic::Code::Unknown => "unknown",
        tonic::Code::InvalidArgument => "invalid_argument",
        tonic::Code::DeadlineExceeded => "deadline_exceeded",
        tonic::Code::NotFound => "not_found",
        tonic::Code::AlreadyExists => "already_exists",
        tonic::Code::PermissionDenied => "permission_denied",
        tonic::Code::ResourceExhausted => "resource_exhausted",
        tonic::Code::FailedPrecondition => "failed_precondition",
        tonic::Code::Aborted => "aborted",
        tonic::Code::OutOfRange => "out_of_range",
        tonic::Code::Unimplemented => "unimplemented",
        tonic::Code::Internal => "internal",
        tonic::Code::Unavailable => "unavailable",
        tonic::Code::DataLoss => "data_loss",
        tonic::Code::Unauthenticated => "unauthenticated",
    }
}

pub fn status_code<T>(result: &tonic::Result<Response<T>>) -> tonic::Code {
    match result {
        Ok(_) => tonic::Code::Ok,
        Err(status) => status.code(),
    }
}

pub fn client_label<T>(request: &Request<T>) -> String {
    match request.metadata().get("x-client-name").map(|v| v.to_str()) {
        Some(Ok(name)) if !name.is_empty() => name.to_string(),
        _ => "unknown".to_string(),
    }
}

pub fn workload_label<T>(request: &Request<T>) -> String {
    match request.metadata().get("src_workload").map(|v| v.to_str()) {
        Some(Ok(name)) if !name.is_empty() => name.to_string(),
        _ => "unknown".to_string(),
    }
}

pub fn src_dc_label<T>(request: &Request<T>) -> String {
    match request.metadata().get("src_dc").map(|v| v.to_str()) {
        Some(Ok(dc)) if !dc.is_empty() => dc.to_string(),
        _ => "unknown".to_string(),
    }
}

pub fn emit_request_metrics(
    start: Instant,
    code: tonic::Code,
    client: &str,
    workload: &str,
    src_dc: &str,
) {
    let label = grpc_status_label(code);
    REQUEST_LATENCY_MS
        .with_label_values(&[label, src_dc])
        .observe(start.elapsed().as_secs_f64() * 1000.0);
    NUM_REQUESTS
        .with_label_values(&[label, client, workload, src_dc])
        .inc();
}

pub fn export() {
    let _ = &*NUM_REQUESTS;
    let _ = &*NUM_REQUESTS_RECEIVED;
    let _ = &*NUM_REQUESTS_REJECTED;
    let _ = &*NUM_REQUESTS_SUCCEEDED;
    let _ = &*NUM_REQUESTS_FAILED;
    let _ = &*NUM_REQUESTS_CANCELED;
    let _ = &*REQUEST_LATENCY_MS;
    let _ = &*NOT_READY_ARRIVAL_SECONDS;
    let _ = &*BATCHING_TIMEOUT_DROP;
    let _ = &*DEADLINE_SHED;
    let _ = &*REQUESTS_WITH_GRPC_TIMEOUT;
    let _ = &*REQUESTS_WITHOUT_GRPC_TIMEOUT;
    let _ = &*ITEMS_IN_FLIGHT;
    let _ = &*ITEM_QUEUE_TIME;
    let _ = &*WAITING_QUEUE_SIZE;
    let _ = &*NUM_REQUESTS_STALE_DROPPED;
    let _ = &*SEQUENCES_PROCESSED;
    let _ = &*TOKENS_PROCESSED;
    let _ = &*TOKENS_PADDED;
    let _ = &*TOKENS_PER_ROW;
    let _ = &*CLIENT_SERVER_NETWORK_DELAY;
    let _ = &*ADMISSION_BUDGET_REMAINING;
    let _ = &*QUEUE_BACKLOG;
    let _ = &*ADMISSION_SERVICE_TIME_MS;
    let _ = &*ADMISSION_PREDICTED_ETA;
    let _ = &*ADMISSION_PIPELINE_TIME_MS;
    NUM_REQUESTS
        .with_label_values(&["ok", "unknown", "unknown", "unknown"])
        .inc_by(0);
    NUM_REQUESTS_REJECTED
        .with_label_values(&["checkpoint_loading", "unknown"])
        .inc_by(0);
    NUM_REQUESTS_REJECTED
        .with_label_values(&["inflight_cap", "unknown"])
        .inc_by(0);
    DEADLINE_SHED.with_label_values(&["unknown"]).inc_by(0);
    TOKENS_PROCESSED.with_label_values(&["history"]).inc_by(0);
    TOKENS_PROCESSED.with_label_values(&["candidate"]).inc_by(0);
    let _ = TOKENS_PER_ROW.with_label_values(&["history"]);
    let _ = TOKENS_PER_ROW.with_label_values(&["candidate"]);
    let _ = NOT_READY_ARRIVAL_SECONDS.with_label_values(&["unknown"]);
    let _ = CLIENT_SERVER_NETWORK_DELAY.with_label_values(&["unknown", "unknown"]);
    let _ = ADMISSION_BUDGET_REMAINING.with_label_values(&["unknown"]);
    NUM_REQUESTS_REJECTED
        .with_label_values(&["deadline_admission", "unknown"])
        .inc_by(0);
}

pub fn record_client_server_network_delay<T>(request: &Request<T>, client: &str) {
    let Some(sent_ms) = request
        .metadata()
        .get("x-request-send-time-ms")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<u64>().ok())
    else {
        return;
    };
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);
    let src_dc = src_dc_label(request);
    CLIENT_SERVER_NETWORK_DELAY
        .with_label_values(&[client, &src_dc])
        .observe(now_ms.saturating_sub(sent_ms) as f64);
}

pub fn remaining_deadline_budget_us(
    grpc_timeout: Option<Duration>,
    fallback_budget_us: Option<u64>,
    elapsed_us: u64,
) -> Option<u64> {
    grpc_timeout
        .map(|t| t.as_micros() as u64)
        .or(fallback_budget_us)
        .map(|budget| budget.saturating_sub(elapsed_us))
}

pub fn mpsc_backlog<T>(tx: &tokio::sync::mpsc::Sender<T>) -> i64 {
    tx.max_capacity().saturating_sub(tx.capacity()) as i64
}

pub fn observe_admission<T, Item>(
    request: &Request<T>,
    client: &str,
    elapsed_us: u64,
    cpu_sender: &tokio::sync::mpsc::Sender<Item>,
    admission: &AdmissionController,
    guard: &mut RequestMetricsGuard,
) -> Result<(), Status> {
    record_client_server_network_delay(request, client);
    let grpc_timeout = extract_grpc_timeout(request);
    let remaining =
        remaining_deadline_budget_us(grpc_timeout, admission.fallback_budget_us(), elapsed_us);
    if let Some(remaining_us) = remaining {
        ADMISSION_BUDGET_REMAINING
            .with_label_values(&[client])
            .observe(remaining_us as f64 / 1000.0);
    }
    let backlog = mpsc_backlog(cpu_sender);
    WAITING_QUEUE_SIZE.set(backlog);
    guard.admit_backlog = backlog;
    if let Some(eta_us) = admission.predicted_eta_us(backlog) {
        ADMISSION_PREDICTED_ETA.observe(eta_us as f64 / 1000.0);
    }
    if !admission.should_reject(remaining, backlog) {
        return Ok(());
    }
    DEADLINE_SHED.with_label_values(&[client]).inc();
    guard.record_reject("deadline_admission");
    let budget_source = if grpc_timeout.is_some() {
        "grpc-timeout"
    } else {
        "deadline_fallback_budget_ms"
    };
    Err(Status::resource_exhausted(format!(
        "shed at admission: predicted completion {}ms + margin {}ms > remaining {}ms \
         (from {budget_source}), backlog={backlog}",
        admission.predicted_eta_us(backlog).unwrap_or(0) / 1000,
        admission.margin_us() / 1000,
        remaining.unwrap_or(0) / 1000,
    )))
}

pub fn record_admission_service_sample(admission: &AdmissionController, sample_us: u64) {
    admission.record_service_time_us(sample_us);
    ADMISSION_SERVICE_TIME_MS.set(admission.service_time_us() as f64 / 1000.0);
}

pub fn reset_admission_estimates(admission: &AdmissionController) {
    admission.reset_estimates();
    ADMISSION_SERVICE_TIME_MS.set(0.0);
    ADMISSION_PIPELINE_TIME_MS.set(0.0);
}

pub fn record_admission_sojourn_sample(
    admission: &AdmissionController,
    sample_us: u64,
    admit_backlog: i64,
) {
    admission.record_sojourn_us(sample_us, admit_backlog);
    ADMISSION_PIPELINE_TIME_MS.set(admission.pipeline_time_us() as f64 / 1000.0);
}

pub struct RequestLabels {
    pub client: String,
    pub workload: String,
    pub src_dc: String,
    pub start: Instant,
}

impl RequestLabels {
    pub fn from_request<T>(request: &Request<T>) -> Self {
        Self {
            client: client_label(request),
            workload: workload_label(request),
            src_dc: src_dc_label(request),
            start: Instant::now(),
        }
    }
}

pub fn extract_grpc_timeout<T>(request: &Request<T>) -> Option<Duration> {
    let raw = request.metadata().get("grpc-timeout")?.to_str().ok()?;
    let (value, unit) = raw.split_at(raw.len().checked_sub(1)?);
    let n: u64 = value.parse().ok()?;
    let d = match unit {
        "H" => Duration::from_secs(n.checked_mul(3600)?),
        "M" => Duration::from_secs(n.checked_mul(60)?),
        "S" => Duration::from_secs(n),
        "m" => Duration::from_millis(n),
        "u" => Duration::from_micros(n),
        "n" => Duration::from_nanos(n),
        _ => return None,
    };
    Some(d)
}

pub fn record_grpc_timeout_presence(grpc_timeout: Option<Duration>) {
    if grpc_timeout.is_some() {
        REQUESTS_WITH_GRPC_TIMEOUT.inc();
    } else {
        REQUESTS_WITHOUT_GRPC_TIMEOUT.inc();
    }
}

pub fn on_request_received(labels: &RequestLabels) {
    NUM_REQUESTS_RECEIVED.inc();
    record_not_ready_arrival(&labels.client);
}

pub fn on_request_received_from<T>(request: &Request<T>, labels: &RequestLabels) {
    on_request_received(labels);
    record_grpc_timeout_presence(extract_grpc_timeout(request));
}

pub fn record_ranking_token_stats<I>(rows: I, history_seq_len: usize, candidate_seq_len: usize)
where
    I: IntoIterator<Item = (usize, usize)>,
{
    let history_hist = TOKENS_PER_ROW.with_label_values(&["history"]);
    let candidate_hist = TOKENS_PER_ROW.with_label_values(&["candidate"]);
    let mut n_rows = 0usize;
    let mut history_tokens = 0usize;
    let mut candidate_tokens = 0usize;
    for (num_history, num_candidates) in rows {
        history_hist.observe(num_history as f64);
        candidate_hist.observe(num_candidates as f64);
        history_tokens += num_history;
        candidate_tokens += num_candidates;
        n_rows += 1;
    }
    if n_rows == 0 {
        return;
    }
    SEQUENCES_PROCESSED.inc_by(n_rows as u64);
    TOKENS_PROCESSED
        .with_label_values(&["history"])
        .inc_by(history_tokens as u64);
    TOKENS_PROCESSED
        .with_label_values(&["candidate"])
        .inc_by(candidate_tokens as u64);
    TOKENS_PADDED.inc_by((n_rows * (history_seq_len + candidate_seq_len)) as u64);
}

pub fn on_request_canceled(labels: &RequestLabels) {
    emit_request_metrics(
        labels.start,
        tonic::Code::Cancelled,
        &labels.client,
        &labels.workload,
        &labels.src_dc,
    );
    NUM_REQUESTS_CANCELED.inc();
}

pub struct RequestMetricsGuard {
    labels: RequestLabels,
    admit_backlog: i64,
    admission: Option<Arc<AdmissionController>>,
    finished: bool,
    recorded_reject: bool,
}

impl RequestMetricsGuard {
    pub fn begin<T>(request: &Request<T>) -> Self {
        let labels = RequestLabels::from_request(request);
        on_request_received_from(request, &labels);
        Self {
            labels,
            admit_backlog: 0,
            admission: None,
            finished: false,
            recorded_reject: false,
        }
    }

    pub fn record_reject(&mut self, reason: &str) {
        NUM_REQUESTS_REJECTED
            .with_label_values(&[reason, self.labels.client.as_str()])
            .inc();
        self.recorded_reject = true;
    }

    pub fn client(&self) -> &str {
        &self.labels.client
    }

    pub fn admit_backlog(&self) -> i64 {
        self.admit_backlog
    }

    pub fn elapsed_us(&self) -> u64 {
        self.labels.start.elapsed().as_micros() as u64
    }

    pub fn arm_sojourn(&mut self, admission: Arc<AdmissionController>) {
        self.admission = Some(admission);
    }

    fn record_sojourn(&self) {
        let Some(admission) = self.admission.as_ref() else {
            return;
        };
        record_admission_sojourn_sample(admission, self.elapsed_us(), self.admit_backlog);
    }

    pub fn finish<R>(mut self, result: tonic::Result<Response<R>>) -> tonic::Result<Response<R>> {
        self.record_sojourn();
        self.finished = true;
        on_request_finished(&self.labels, &result, self.recorded_reject);
        result
    }
}

impl Drop for RequestMetricsGuard {
    fn drop(&mut self) {
        if self.finished {
            return;
        }
        self.record_sojourn();
        on_request_canceled(&self.labels);
    }
}

pub fn on_request_finished<T>(
    labels: &RequestLabels,
    result: &tonic::Result<Response<T>>,
    already_rejected: bool,
) {
    let code = status_code(result);
    emit_request_metrics(
        labels.start,
        code,
        &labels.client,
        &labels.workload,
        &labels.src_dc,
    );
    match code {
        tonic::Code::Ok => NUM_REQUESTS_SUCCEEDED.inc(),
        tonic::Code::Cancelled => NUM_REQUESTS_CANCELED.inc(),
        tonic::Code::Unavailable => {
            if !already_rejected {
                NUM_REQUESTS_REJECTED
                    .with_label_values(&["checkpoint_loading", labels.client.as_str()])
                    .inc();
            }
        }
        tonic::Code::ResourceExhausted => {
            if !already_rejected {
                NUM_REQUESTS_REJECTED
                    .with_label_values(&["inflight_cap", labels.client.as_str()])
                    .inc();
            }
        }
        tonic::Code::InvalidArgument => {}
        _ => NUM_REQUESTS_FAILED.inc(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::admission::{AdmissionConfig, AdmissionEtaModel};
    use tonic::Request;

    #[test]
    fn client_label_uses_client_name_and_defaults_to_unknown() {
        let mut req = Request::new(());
        req.metadata_mut().insert(
            "x-client-name",
            "example-client/example-model".parse().unwrap(),
        );
        assert_eq!(client_label(&req), "example-client/example-model");

        let absent = Request::new(());
        assert_eq!(client_label(&absent), "unknown");
    }

    #[test]
    fn workload_label_uses_src_workload_independently_of_client() {
        let mut both = Request::new(());
        both.metadata_mut()
            .insert("x-client-name", "example-service/ranking".parse().unwrap());
        both.metadata_mut()
            .insert("src_workload", "example-service-canary".parse().unwrap());
        assert_eq!(client_label(&both), "example-service/ranking");
        assert_eq!(workload_label(&both), "example-service-canary");

        let absent = Request::new(());
        assert_eq!(workload_label(&absent), "unknown");
    }

    #[test]
    fn src_dc_label_uses_src_dc_and_defaults_to_unknown() {
        let mut req = Request::new(());
        req.metadata_mut().insert("src_dc", "pdxa".parse().unwrap());
        assert_eq!(src_dc_label(&req), "pdxa");

        let absent = Request::new(());
        assert_eq!(src_dc_label(&absent), "unknown");
    }

    #[test]
    fn network_delay_records_under_client_and_src_dc() {
        let mut req = Request::new(());
        req.metadata_mut()
            .insert("x-request-send-time-ms", "1".parse().unwrap());
        req.metadata_mut().insert("src_dc", "pdxa".parse().unwrap());
        record_client_server_network_delay(&req, "example-service/ranking-xds");
        let _ =
            CLIENT_SERVER_NETWORK_DELAY.with_label_values(&["example-service/ranking-xds", "pdxa"]);
    }

    #[test]
    fn not_ready_probe_transitions() {
        disarm_not_ready_probe();
        assert!(not_ready_elapsed_seconds().is_none());

        arm_not_ready_probe();
        let armed = NOT_READY_SINCE_EPOCH_MS.load(Ordering::Relaxed);
        assert!(armed > 0);
        let elapsed = not_ready_elapsed_seconds().expect("armed while unavailable");
        assert!(elapsed >= 0.0);
        record_not_ready_arrival("test-client");

        arm_not_ready_probe();
        assert_eq!(NOT_READY_SINCE_EPOCH_MS.load(Ordering::Relaxed), armed);

        disarm_not_ready_probe();
        assert!(not_ready_elapsed_seconds().is_none());
        record_not_ready_arrival("test-client");
    }

    fn canceled_count(client: &str) -> u64 {
        NUM_REQUESTS
            .with_label_values(&["cancelled", client, "unknown", "unknown"])
            .get()
    }

    #[test]
    fn guard_drop_without_finish_counts_canceled() {
        let client = "test-guard-drop";
        let mut req = Request::new(());
        req.metadata_mut()
            .insert("x-client-name", client.parse().unwrap());
        let before = canceled_count(client);
        {
            let _guard = RequestMetricsGuard::begin(&req);
        }
        assert_eq!(canceled_count(client), before + 1);
    }

    #[test]
    fn guard_finish_does_not_count_canceled() {
        let client = "test-guard-finish";
        let mut req = Request::new(());
        req.metadata_mut()
            .insert("x-client-name", client.parse().unwrap());
        let before = canceled_count(client);
        let guard = RequestMetricsGuard::begin(&req);
        let _ = guard.finish::<()>(Err(tonic::Status::invalid_argument("x")));
        assert_eq!(canceled_count(client), before);
    }

    #[test]
    fn guard_drop_records_sojourn_when_armed() {
        let admission = Arc::new(AdmissionController::new(
            crate::admission::AdmissionConfig {
                enabled: false,
                batch_size: 32,
                margin_us: 0,
                post_us: 0,
                fallback_budget_us: 0,
                ewma_alpha: 1.0,
                pipeline_depth: 0,
                eta_model: crate::admission::AdmissionEtaModel::Pipeline,
            },
        ));
        let req = Request::new(());
        let mut guard = RequestMetricsGuard::begin(&req);
        guard.arm_sojourn(admission.clone());
        std::thread::sleep(Duration::from_millis(1));
        drop(guard);
        assert!(
            admission.pipeline_time_us() > 0,
            "client cancel must still fold P"
        );
    }

    fn rejected(reason: &str, client: &str) -> u64 {
        NUM_REQUESTS_REJECTED
            .with_label_values(&[reason, client])
            .get()
    }

    #[test]
    fn reset_admission_estimates_clears_gauges_and_p() {
        let admission = AdmissionController::new(AdmissionConfig {
            enabled: true,
            batch_size: 4,
            margin_us: 0,
            post_us: 0,
            fallback_budget_us: 0,
            ewma_alpha: 1.0,
            pipeline_depth: 0,
            eta_model: AdmissionEtaModel::Pipeline,
        });
        admission.record_service_time_us(50_000);
        admission.record_sojourn_us(80_000, 0);
        record_admission_service_sample(&admission, 50_000);
        record_admission_sojourn_sample(&admission, 80_000, 0);
        assert!(admission.pipeline_time_us() > 0);
        reset_admission_estimates(&admission);
        assert_eq!(admission.service_time_us(), 0);
        assert_eq!(admission.pipeline_time_us(), 0);
        assert_eq!(ADMISSION_SERVICE_TIME_MS.get(), 0.0);
        assert_eq!(ADMISSION_PIPELINE_TIME_MS.get(), 0.0);
    }

    #[test]
    fn deadline_shed_does_not_double_count_as_inflight_cap() {
        let client = "test-deadline-shed-no-double";
        let admission = AdmissionController::new(AdmissionConfig {
            enabled: true,
            batch_size: 1,
            margin_us: 0,
            post_us: 0,
            fallback_budget_us: 0,
            ewma_alpha: 1.0,
            pipeline_depth: 0,
            eta_model: AdmissionEtaModel::Loop,
        });
        admission.record_service_time_us(100_000);

        let (tx, _rx) = tokio::sync::mpsc::channel::<()>(2);
        tx.try_send(()).unwrap();

        let mut req = Request::new(());
        req.metadata_mut()
            .insert("x-client-name", client.parse().unwrap());
        req.metadata_mut()
            .insert("grpc-timeout", "1m".parse().unwrap());

        let shed_before = DEADLINE_SHED.with_label_values(&[client]).get();
        let deadline_before = rejected("deadline_admission", client);
        let inflight_before = rejected("inflight_cap", client);

        let mut guard = RequestMetricsGuard::begin(&req);
        let err = observe_admission(&req, client, 0, &tx, &admission, &mut guard)
            .expect_err("deadline admission must shed");
        assert_eq!(err.code(), tonic::Code::ResourceExhausted);
        let _ = guard.finish::<()>(Err(err));

        assert_eq!(
            DEADLINE_SHED.with_label_values(&[client]).get(),
            shed_before + 1
        );
        assert_eq!(rejected("deadline_admission", client), deadline_before + 1);
        assert_eq!(rejected("inflight_cap", client), inflight_before);
    }

    #[test]
    fn resource_exhausted_without_prior_reject_counts_inflight_cap() {
        let client = "test-inflight-cap-from-finish";
        let mut req = Request::new(());
        req.metadata_mut()
            .insert("x-client-name", client.parse().unwrap());
        let before = rejected("inflight_cap", client);
        let guard = RequestMetricsGuard::begin(&req);
        let _ = guard.finish::<()>(Err(Status::resource_exhausted("queue full")));
        assert_eq!(rejected("inflight_cap", client), before + 1);
    }
}
