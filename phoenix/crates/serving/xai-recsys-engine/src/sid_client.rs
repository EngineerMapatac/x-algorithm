// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 X.AI Corp.
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use lazy_static::lazy_static;
use log::warn;
use prometheus::{
    CounterVec, HistogramVec, exponential_buckets, register_counter_vec, register_histogram_vec,
};
use pyo3::prelude::*;
use tonic::transport::Endpoint;
use xai_recsys::reply::ReplyMetadata;
use xai_recsys_proto as pb;
use xai_recsys_sid_proto::{LookupSidsRequest, sid_lookup_service_client::SidLookupServiceClient};

const REQUEST_TIMEOUT: Duration = Duration::from_millis(100);
const NUM_CHANNELS: usize = 2;
const CACHE_MAX_BYTES: u64 = 256 * 1024 * 1024;
const CACHE_TTL: Duration = Duration::from_secs(6 * 3600);
const CACHE_ENTRY_OVERHEAD_BYTES: u32 = 96;

lazy_static! {
    static ref SID_LOOKUP_LATENCY_SECS: HistogramVec = register_histogram_vec!(
        "recsys_engine_sid_lookup_latency_seconds",
        "Latency of semantic ID lookup gRPC calls",
        &["status"],
        exponential_buckets(0.001, 2.0, 15).unwrap(),
    )
    .unwrap();
    static ref SID_LOOKUP_COUNT: CounterVec = register_counter_vec!(
        "recsys_engine_sid_lookup_count",
        "Number of semantic ID lookups by status",
        &["status"],
    )
    .unwrap();
    static ref SID_CACHE_COUNT: CounterVec = register_counter_vec!(
        "recsys_engine_sid_cache_count",
        "Semantic ID in-process cache hits/misses",
        &["result"],
    )
    .unwrap();
    static ref SID_HYDRATION_CANDIDATES: CounterVec = register_counter_vec!(
        "recsys_engine_sid_hydration_candidates",
        "Candidates sent to / filled by request-path semantic ID hydration",
        &["status"],
    )
    .unwrap();
}

pub struct SemanticIdClient {
    clients: Vec<SidLookupServiceClient<tonic::transport::Channel>>,
    next: AtomicUsize,
    sid_num_levels: usize,
    cache: moka::future::Cache<i64, Arc<Vec<i32>>>,
}

impl SemanticIdClient {
    fn new(endpoint: &str, sid_num_levels: usize) -> Self {
        let endpoint = Endpoint::from_shared(endpoint.to_string())
            .expect("invalid SID endpoint URL")
            .timeout(REQUEST_TIMEOUT)
            .tcp_nodelay(true)
            .http2_keep_alive_interval(Duration::from_secs(30))
            .keep_alive_timeout(Duration::from_secs(20))
            .keep_alive_while_idle(true);
        let clients = (0..NUM_CHANNELS)
            .map(|_| SidLookupServiceClient::new(endpoint.connect_lazy()))
            .collect();
        Self {
            clients,
            next: AtomicUsize::new(0),
            sid_num_levels,
            cache: moka::future::Cache::builder()
                .weigher(|_k: &i64, v: &Arc<Vec<i32>>| {
                    CACHE_ENTRY_OVERHEAD_BYTES + (v.len() * std::mem::size_of::<i32>()) as u32
                })
                .max_capacity(CACHE_MAX_BYTES)
                .time_to_live(CACHE_TTL)
                .build(),
        }
    }

    pub async fn hydrate(
        &self,
        candidate_set: &mut pb::CandidateSet,
        reply: &mut ReplyMetadata,
        max_candidates: usize,
        deadline: Option<Instant>,
    ) {
        const MIN_BUDGET: Duration = Duration::from_millis(10);
        const BUDGET_MARGIN: Duration = Duration::from_millis(5);
        let mut ids: Vec<i64> = candidate_set
            .candidates
            .iter()
            .take(max_candidates)
            .filter(|c| c.semantic_ids.is_empty() && c.tweet_id != 0)
            .map(|c| c.tweet_id as i64)
            .collect();
        if reply.focal_semantic_ids.is_empty() {
            ids.push(reply.focal_post_id as i64);
        }
        if ids.is_empty() {
            return;
        }
        SID_HYDRATION_CANDIDATES
            .with_label_values(&["requested"])
            .inc_by(ids.len() as f64);
        let (mut sids, misses) = self.cached(&ids).await;
        if !misses.is_empty() {
            match deadline.map(|d| d.saturating_duration_since(Instant::now())) {
                Some(b) if b < MIN_BUDGET => {
                    SID_HYDRATION_CANDIDATES
                        .with_label_values(&["skipped_budget"])
                        .inc_by(misses.len() as f64);
                }
                budget => {
                    let timeout = budget.map_or(REQUEST_TIMEOUT, |b| {
                        (b - BUDGET_MARGIN).min(REQUEST_TIMEOUT)
                    });
                    match tokio::time::timeout(timeout, self.fetch(&misses)).await {
                        Ok(fetched) => sids.extend(fetched),
                        Err(_) => {
                            SID_HYDRATION_CANDIDATES
                                .with_label_values(&["timed_out"])
                                .inc_by(misses.len() as f64);
                        }
                    }
                }
            }
        }
        let mut filled = 0usize;
        for candidate in &mut candidate_set.candidates {
            if candidate.semantic_ids.is_empty()
                && let Some(codes) = sids.get(&(candidate.tweet_id as i64))
                && codes.first().is_some_and(|&c| c >= 0)
            {
                candidate.semantic_ids = codes.clone();
                filled += 1;
            }
        }
        if reply.focal_semantic_ids.is_empty()
            && let Some(codes) = sids.get(&(reply.focal_post_id as i64))
            && codes.first().is_some_and(|&c| c >= 0)
        {
            reply.focal_semantic_ids = codes.clone();
        }
        SID_HYDRATION_CANDIDATES
            .with_label_values(&["filled"])
            .inc_by(filled as f64);
    }

    async fn cached(&self, post_ids: &[i64]) -> (HashMap<i64, Vec<i32>>, Vec<i64>) {
        let mut result = HashMap::with_capacity(post_ids.len());
        let mut misses = Vec::new();
        let mut hits = 0usize;
        for &id in post_ids {
            if let Some(codes) = self.cache.get(&id).await {
                hits += 1;
                result.insert(id, (*codes).clone());
            } else {
                misses.push(id);
            }
        }
        SID_CACHE_COUNT
            .with_label_values(&["hit"])
            .inc_by(hits as f64);
        SID_CACHE_COUNT
            .with_label_values(&["miss"])
            .inc_by(misses.len() as f64);
        misses.sort_unstable();
        misses.dedup();
        (result, misses)
    }

    async fn fetch(&self, post_ids: &[i64]) -> HashMap<i64, Vec<i32>> {
        let fetched = self.lookup(post_ids).await;
        for (id, codes) in &fetched {
            if codes.first().is_some_and(|c| *c >= 0) {
                self.cache.insert(*id, Arc::new(codes.clone())).await;
            }
        }
        fetched
    }

    async fn lookup(&self, post_ids: &[i64]) -> HashMap<i64, Vec<i32>> {
        let start = Instant::now();
        let request = LookupSidsRequest {
            post_ids: post_ids.to_vec(),
        };

        let idx = self.next.fetch_add(1, Ordering::Relaxed) % self.clients.len();
        let mut client = self.clients[idx].clone();
        match client.lookup_sids(request).await {
            Ok(response) => {
                let elapsed = start.elapsed().as_secs_f64();
                SID_LOOKUP_LATENCY_SECS
                    .with_label_values(&["success"])
                    .observe(elapsed);

                let resp = response.into_inner();
                let mut result = HashMap::with_capacity(post_ids.len());
                let missing = vec![-1i32; self.sid_num_levels];
                for (post_id, post_sids) in post_ids.iter().zip(resp.results.iter()) {
                    if post_sids.codes.is_empty() {
                        SID_LOOKUP_COUNT.with_label_values(&["miss"]).inc();
                        result.insert(*post_id, missing.clone());
                    } else if post_sids.codes.len() != self.sid_num_levels {
                        SID_LOOKUP_COUNT.with_label_values(&["bad_length"]).inc();
                        result.insert(*post_id, missing.clone());
                    } else {
                        SID_LOOKUP_COUNT.with_label_values(&["hit"]).inc();
                        result.insert(*post_id, post_sids.codes.clone());
                    }
                }
                result
            }
            Err(e) => {
                let elapsed = start.elapsed().as_secs_f64();
                SID_LOOKUP_LATENCY_SECS
                    .with_label_values(&["error"])
                    .observe(elapsed);
                SID_LOOKUP_COUNT
                    .with_label_values(&["error"])
                    .inc_by(post_ids.len() as f64);
                warn!("SID lookup gRPC call failed: {}", e);
                let missing = vec![-1i32; self.sid_num_levels];
                post_ids.iter().map(|id| (*id, missing.clone())).collect()
            }
        }
    }
}

#[pyclass(module = "xai_recsys_engine")]
pub struct PySemanticIdClient {
    endpoint: String,
    sid_num_levels: usize,
}

impl PySemanticIdClient {
    pub fn build(&self) -> SemanticIdClient {
        SemanticIdClient::new(&self.endpoint, self.sid_num_levels)
    }
}

#[pymethods]
impl PySemanticIdClient {
    #[new]
    #[pyo3(signature = (endpoint, sid_num_levels))]
    fn new(endpoint: String, sid_num_levels: usize) -> PyResult<Self> {
        Ok(Self {
            endpoint,
            sid_num_levels,
        })
    }
}
