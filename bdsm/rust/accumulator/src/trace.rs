use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use dashmap::DashMap;
use rand::Rng;
use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord, Producer};
use serde::Serialize;
use serde_json::json;
use tokio::sync::mpsc;
use tracing::{debug, info, warn};

pub const TRACE_REDIS_ACTIVE_TTL_SECS: u64 = 600;
const FLUSH_INTERVAL_MS: u64 = 250;
const FLUSH_BATCH: usize = 500;
const CHANNEL_CAPACITY: usize = 100_000;
const SADD_CHANNEL_CAPACITY: usize = 200_000;
const SCHEMA_VERSION: i32 = 1;

#[derive(Clone)]
pub struct TraceConfig {
    pub enabled: bool,
    pub sample_rate: f32,
    pub kafka_bootstrap: String,
    pub kafka_topic: String,
    pub kafka_username: String,
    pub kafka_password: String,
    pub redis_url: String,
    pub pod_name: String,
}

impl TraceConfig {
    pub fn from_env() -> Self {
        let env = |k: &str, default: &str| std::env::var(k).unwrap_or_else(|_| default.to_string());
        Self {
            enabled: env("TRACE_ENABLED", "0") == "1",
            sample_rate: env("TRACE_SAMPLE_RATE", "0.10").parse().unwrap_or(0.10),
            kafka_bootstrap: env("TRACE_KAFKA_BOOTSTRAP", "localhost:9092"),
            kafka_topic: env("TRACE_KAFKA_TOPIC", "abuse_pipeline_trace"),
            kafka_username: std::env::var("TRACE_KAFKA_USERNAME")
                .or_else(|_| std::env::var("KAFKA_SASL_USERNAME"))
                .unwrap_or_default(),
            kafka_password: std::env::var("TRACE_KAFKA_PASSWORD")
                .or_else(|_| std::env::var("KAFKA_SASL_PASSWORD"))
                .unwrap_or_default(),
            redis_url: format!(
                "redis://{}:{}",
                env("TRACE_REDIS_HOST", "abuse-trace-redis"),
                env("TRACE_REDIS_PORT", "6379")
            ),
            pod_name: env(
                "POD_NAME",
                &std::env::var("HOSTNAME").unwrap_or_else(|_| "unknown".to_string()),
            ),
        }
    }
}

#[derive(Default)]
pub struct TraceMetrics {
    pub n_consumed_sampled: AtomicU64,
    pub n_emit_dropped_full: AtomicU64,
    pub n_kafka_err: AtomicU64,
    pub n_sadd_dropped_full: AtomicU64,
    pub n_sadd_err: AtomicU64,
    pub n_recently_sampled_size: AtomicU64,
}

pub struct TraceEmitter {
    pub enabled: bool,
    pub sample_rate: f32,
    pub pod_name: String,
    pub recently_sampled: Arc<DashMap<i64, Instant>>,
    pub event_tx: Option<mpsc::Sender<TraceMsg>>,
    pub sadd_tx: Option<mpsc::Sender<i64>>,
    pub metrics: Arc<TraceMetrics>,
}

#[derive(Clone, Debug, Serialize)]
pub struct TraceMsg {
    pub ts_ms: i64,
    pub stage: &'static str,
    pub uid: i64,
    pub decision: &'static str,
    pub pod: String,
    pub partition: Option<i32>,
    pub offset: Option<i64>,
    pub correlator: Option<f64>,
    pub batch_id: Option<String>,
    pub detail_json: Option<String>,
    pub sample_rate: Option<f32>,
    pub schema_version: i32,
}

impl TraceEmitter {
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            sample_rate: 0.0,
            pod_name: "disabled".into(),
            recently_sampled: Arc::new(DashMap::new()),
            event_tx: None,
            sadd_tx: None,
            metrics: Arc::new(TraceMetrics::default()),
        }
    }

    pub fn new_active(config: TraceConfig) -> Self {
        if !config.enabled {
            return Self::disabled();
        }

        let metrics = Arc::new(TraceMetrics::default());
        let recently_sampled: Arc<DashMap<i64, Instant>> =
            Arc::new(DashMap::with_capacity(1_000_000));

        let mut producer_cfg = ClientConfig::new();
        producer_cfg
            .set("bootstrap.servers", &config.kafka_bootstrap)
            .set("security.protocol", "SASL_SSL")
            .set("sasl.mechanism", "SCRAM-SHA-512")
            .set("sasl.username", &config.kafka_username)
            .set("sasl.password", &config.kafka_password)
            .set("linger.ms", "50")
            .set("batch.num.messages", &(FLUSH_BATCH * 2).to_string())
            .set("compression.type", "lz4")
            .set("acks", "1")
            .set("queue.buffering.max.messages", "200000");
        crate::kafka_tls_config(&mut producer_cfg);
        let producer: FutureProducer = producer_cfg
            .create()
            .expect("failed to build trace Kafka producer");

        let (event_tx, mut event_rx) = mpsc::channel::<TraceMsg>(CHANNEL_CAPACITY);

        {
            let topic = config.kafka_topic.clone();
            let metrics = Arc::clone(&metrics);
            tokio::spawn(async move {
                let mut batch: Vec<TraceMsg> = Vec::with_capacity(FLUSH_BATCH);
                let flush_interval = Duration::from_millis(FLUSH_INTERVAL_MS);
                loop {
                    let timeout = tokio::time::sleep(flush_interval);
                    tokio::pin!(timeout);
                    tokio::select! {
                        msg = event_rx.recv() => {
                            match msg {
                                Some(m) => batch.push(m),
                                None => break,
                            }
                        }
                        _ = &mut timeout => {}
                    }
                    if !batch.is_empty() {
                        for m in batch.drain(..) {
                            let key = m.uid.to_string();
                            let payload = match serde_json::to_vec(&m) {
                                Ok(p) => p,
                                Err(_) => continue,
                            };
                            let rec = FutureRecord::to(&topic).key(&key).payload(&payload);
                            if let Err((e, _)) = producer.send_result(rec) {
                                metrics.n_kafka_err.fetch_add(1, Ordering::Relaxed);
                                debug!("trace kafka send failed: {:?}", e);
                            }
                        }
                    }
                }
                producer.flush(Duration::from_secs(5)).ok();
            });
        }

        let (sadd_tx, mut sadd_rx) = mpsc::channel::<i64>(SADD_CHANNEL_CAPACITY);
        {
            let redis_url = config.redis_url.clone();
            let metrics = Arc::clone(&metrics);
            tokio::spawn(async move {
                let client = match redis::Client::open(redis_url.clone()) {
                    Ok(c) => c,
                    Err(e) => {
                        warn!("trace-redis client open failed: {e}; SADDs will all be dropped");
                        return;
                    }
                };
                let mut conn = match client.get_multiplexed_async_connection().await {
                    Ok(c) => c,
                    Err(e) => {
                        warn!("trace-redis connect failed: {e}; SADDs will all be dropped");
                        return;
                    }
                };

                let mut buf: Vec<i64> = Vec::with_capacity(500);
                loop {
                    let first = match sadd_rx.recv().await {
                        Some(u) => u,
                        None => break,
                    };
                    buf.push(first);
                    while let Ok(u) = sadd_rx.try_recv() {
                        buf.push(u);
                        if buf.len() >= 500 {
                            break;
                        }
                    }
                    let mut pipe = redis::pipe();
                    pipe.atomic();
                    for uid in buf.drain(..) {
                        pipe.set_options(
                            format!("trace:active:{uid}"),
                            "1",
                            redis::SetOptions::default()
                                .with_expiration(redis::SetExpiry::EX(TRACE_REDIS_ACTIVE_TTL_SECS)),
                        )
                        .ignore();
                    }
                    if let Err(e) = pipe.query_async::<()>(&mut conn).await {
                        metrics.n_sadd_err.fetch_add(1, Ordering::Relaxed);
                        debug!("trace-redis SADD pipeline failed: {e}");
                    }
                }
            });
        }

        {
            let map = Arc::clone(&recently_sampled);
            let metrics = Arc::clone(&metrics);
            tokio::spawn(async move {
                let interval = Duration::from_secs(60);
                let ttl = Duration::from_secs(TRACE_REDIS_ACTIVE_TTL_SECS);
                loop {
                    tokio::time::sleep(interval).await;
                    map.retain(|_uid, ts| ts.elapsed() < ttl);
                    metrics
                        .n_recently_sampled_size
                        .store(map.len() as u64, Ordering::Relaxed);
                }
            });
        }

        Self {
            enabled: true,
            sample_rate: config.sample_rate,
            pod_name: config.pod_name.clone(),
            recently_sampled,
            event_tx: Some(event_tx),
            sadd_tx: Some(sadd_tx),
            metrics,
        }
    }

    #[inline]
    pub fn maybe_sample_consume(&self, uid: i64, partition: i32, offset: i64) {
        if !self.enabled {
            return;
        }
        let mut rng = rand::thread_rng();
        if rng.gen::<f32>() >= self.sample_rate {
            return;
        }
        self.recently_sampled.insert(uid, Instant::now());
        self.metrics
            .n_consumed_sampled
            .fetch_add(1, Ordering::Relaxed);

        if let Some(tx) = &self.sadd_tx {
            if tx.try_send(uid).is_err() {
                self.metrics
                    .n_sadd_dropped_full
                    .fetch_add(1, Ordering::Relaxed);
            }
        }
        self.emit_msg(TraceMsg {
            ts_ms: now_ms(),
            stage: "accum_consume",
            uid,
            decision: "ok",
            pod: self.pod_name.clone(),
            partition: Some(partition),
            offset: Some(offset),
            correlator: None,
            batch_id: None,
            detail_json: None,
            sample_rate: Some(self.sample_rate),
            schema_version: SCHEMA_VERSION,
        });
    }

    #[inline]
    pub fn is_traced(&self, uid: i64) -> bool {
        if !self.enabled {
            return false;
        }
        match self.recently_sampled.get(&uid) {
            Some(ent) => ent.value().elapsed() < Duration::from_secs(TRACE_REDIS_ACTIVE_TTL_SECS),
            None => false,
        }
    }

    #[inline]
    pub fn emit_flush_decision(
        &self,
        uid: i64,
        decision: &'static str,
        correlator: Option<f64>,
        detail_json: Option<String>,
    ) {
        if !self.enabled {
            return;
        }
        self.emit_msg(TraceMsg {
            ts_ms: now_ms(),
            stage: "accum_flush_decision",
            uid,
            decision,
            pod: self.pod_name.clone(),
            partition: None,
            offset: None,
            correlator,
            batch_id: None,
            detail_json,
            sample_rate: Some(self.sample_rate),
            schema_version: SCHEMA_VERSION,
        });
    }

    #[inline]
    pub fn emit_try_send_full(&self, uid: i64, partition: i32, offset: i64) {
        if !self.enabled {
            return;
        }
        self.emit_msg(TraceMsg {
            ts_ms: now_ms(),
            stage: "accum_try_send_full",
            uid,
            decision: "dropped",
            pod: self.pod_name.clone(),
            partition: Some(partition),
            offset: Some(offset),
            correlator: None,
            batch_id: None,
            detail_json: None,
            sample_rate: Some(self.sample_rate),
            schema_version: SCHEMA_VERSION,
        });
    }

    #[inline]
    fn emit_msg(&self, msg: TraceMsg) {
        if let Some(tx) = &self.event_tx {
            if tx.try_send(msg).is_err() {
                self.metrics
                    .n_emit_dropped_full
                    .fetch_add(1, Ordering::Relaxed);
            }
        }
    }
}

#[inline]
fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

pub fn metrics_summary(m: &TraceMetrics) -> String {
    format!(
        "trace[sampled={} kafka_err={} emit_dropped={} sadd_dropped={} sadd_err={} mem_uids={}]",
        m.n_consumed_sampled.load(Ordering::Relaxed),
        m.n_kafka_err.load(Ordering::Relaxed),
        m.n_emit_dropped_full.load(Ordering::Relaxed),
        m.n_sadd_dropped_full.load(Ordering::Relaxed),
        m.n_sadd_err.load(Ordering::Relaxed),
        m.n_recently_sampled_size.load(Ordering::Relaxed),
    )
}
