use byteorder::{LittleEndian, ReadBytesExt};
use clap::Parser;
use dashmap::DashMap;
use rdkafka::config::ClientConfig;
use rdkafka::consumer::{CommitMode, Consumer, StreamConsumer};
use rdkafka::message::Message;
use std::collections::HashSet;
use std::io::Cursor;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::sync::Mutex;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::mpsc;
use tracing::{error, info, warn};

mod trace;
use trace::{TraceConfig, TraceEmitter};

const USERID_OFFSET: usize = 7744;

const MIN_MSG_SIZE: usize = USERID_OFFSET + 8;

const ARROW_CONTINUATION: [u8; 4] = [0xff, 0xff, 0xff, 0xff];

#[derive(Parser, Debug)]
#[command(name = "abuse-score-accumulator")]
struct Args {
    #[arg(long, env = "KAFKA_BOOTSTRAP", default_value = "localhost:9092")]
    kafka_bootstrap: String,

    #[arg(long, env = "KAFKA_SASL_USERNAME", default_value = "kafka-user")]
    kafka_username: String,

    #[arg(long, env = "KAFKA_SASL_PASSWORD")]
    kafka_password: String,

    #[arg(long, env = "KAFKA_TOPIC", default_value = "user_action_sequences")]
    kafka_topic: String,

    #[arg(
        long,
        env = "KAFKA_GROUP",
        default_value = "abuse-score-accumulator-rust"
    )]
    kafka_group: String,

    #[arg(long, env = "REDIS_HOST", default_value = "abuse-score-redis")]
    redis_host: String,

    #[arg(long, env = "REDIS_PORT", default_value_t = 6379)]
    redis_port: u16,

    #[arg(long, env = "REDIS_KEY", default_value = "abuse:score_queue")]
    redis_key: String,

    #[arg(long, env = "HEALTH_PORT", default_value_t = 8081)]
    health_port: u16,

    #[arg(long, default_value_t = 2000)]
    batch_size: usize,

    #[arg(long, default_value_t = 500)]
    flush_interval_ms: u64,

    #[arg(long, env = "COOLDOWN_CACHE_TTL_SECS", default_value_t = 1800)]
    cooldown_cache_ttl_secs: u64,

    #[arg(long, env = "COOLDOWN_CACHE_MAX", default_value_t = 5_000_000)]
    cooldown_cache_max: usize,

    #[arg(long, env = "ENQUEUE_LEASE_TTL_SECS", default_value_t = 1800)]
    enqueue_lease_ttl_secs: u64,

    #[arg(long, env = "KAFKA_SESSION_TIMEOUT_MS", default_value_t = 180_000)]
    kafka_session_timeout_ms: u64,

    #[arg(long, env = "KAFKA_MAX_POLL_INTERVAL_MS", default_value_t = 600_000)]
    kafka_max_poll_interval_ms: u64,

    #[arg(long, env = "KAFKA_ASSIGNMENT_STRATEGY", default_value = "range")]
    kafka_assignment_strategy: String,

    #[arg(long, env = "LIVENESS_WINDOW_SECS", default_value_t = 60)]
    liveness_window_secs: u64,

    #[arg(long, env = "LIVENESS_REDIS_ERRORS_PER_SEC", default_value_t = 10)]
    liveness_redis_errors_per_sec: u64,

    #[arg(long, env = "LIVENESS_MIN_WRITES_PER_SEC", default_value_t = 5)]
    liveness_min_writes_per_sec: u64,

    #[arg(long, env = "LIVENESS_MIN_CONSUMED_PER_SEC", default_value_t = 100)]
    liveness_min_consumed_per_sec: u64,

    #[arg(long, env = "LIVENESS_GRACE_SECS", default_value_t = 180)]
    liveness_grace_secs: u64,

    #[arg(long, env = "GROUP_INSTANCE_ID")]
    group_instance_id: Option<String>,
}

struct Metrics {
    consumed: AtomicU64,
    redis_writes: AtomicU64,
    gate_filtered: AtomicU64,
    parse_errors: AtomicU64,
    kafka_errors: AtomicU64,
    redis_errors: AtomicU64,
    dedup_collapsed: AtomicU64,
    cache_skipped: AtomicU64,
    leases_set: AtomicU64,
}

impl Metrics {
    fn new() -> Self {
        Self {
            consumed: AtomicU64::new(0),
            redis_writes: AtomicU64::new(0),
            gate_filtered: AtomicU64::new(0),
            parse_errors: AtomicU64::new(0),
            kafka_errors: AtomicU64::new(0),
            redis_errors: AtomicU64::new(0),
            dedup_collapsed: AtomicU64::new(0),
            cache_skipped: AtomicU64::new(0),
            leases_set: AtomicU64::new(0),
        }
    }
}

struct HealthState {
    snapshots: Mutex<Option<HealthSnapshot>>,
    started_at: Instant,
    grace: Duration,
    err_threshold_per_sec: u64,
    min_writes_per_sec: u64,
    min_consumed_per_sec: u64,
}

#[derive(Clone)]
struct HealthSnapshot {
    consumed_prev: u64,
    consumed_curr: u64,
    writes_prev: u64,
    writes_curr: u64,
    errors_prev: u64,
    errors_curr: u64,
    window_secs: u64,
}

impl HealthState {
    fn new(
        grace: Duration,
        err_threshold_per_sec: u64,
        min_writes_per_sec: u64,
        min_consumed_per_sec: u64,
    ) -> Self {
        Self {
            snapshots: Mutex::new(None),
            started_at: Instant::now(),
            grace,
            err_threshold_per_sec,
            min_writes_per_sec,
            min_consumed_per_sec,
        }
    }

    fn evaluate(&self) -> (bool, String) {
        if self.started_at.elapsed() < self.grace {
            return (
                true,
                format!(
                    "ok (grace, {}s remaining)",
                    self.grace.as_secs() - self.started_at.elapsed().as_secs()
                ),
            );
        }

        let snap = match self.snapshots.lock().unwrap().clone() {
            Some(s) => s,
            None => return (true, "ok (warming up, no snapshot yet)".to_string()),
        };

        if snap.window_secs == 0 {
            return (true, "ok (window not yet covered)".to_string());
        }

        let consumed_rate =
            snap.consumed_curr.saturating_sub(snap.consumed_prev) / snap.window_secs;
        let writes_rate = snap.writes_curr.saturating_sub(snap.writes_prev) / snap.window_secs;
        let errors_rate = snap.errors_curr.saturating_sub(snap.errors_prev) / snap.window_secs;

        let unhealthy = errors_rate > self.err_threshold_per_sec
            && writes_rate < self.min_writes_per_sec
            && consumed_rate > self.min_consumed_per_sec;

        let reason = format!(
            "consumed_rate={consumed_rate}/s writes_rate={writes_rate}/s errors_rate={errors_rate}/s \
             (thresholds: err>{} & writes<{} & consumed>{})",
            self.err_threshold_per_sec, self.min_writes_per_sec, self.min_consumed_per_sec,
        );

        if unhealthy {
            (false, format!("UNHEALTHY: {reason}"))
        } else {
            (true, format!("ok: {reason}"))
        }
    }
}

struct CooldownCache {
    map: DashMap<i64, Instant>,
    ttl: Duration,
    max_size: usize,
}

impl CooldownCache {
    fn new(ttl: Duration, max_size: usize) -> Self {
        Self {
            map: DashMap::with_capacity(max_size.min(1_000_000)),
            ttl,
            max_size,
        }
    }

    fn contains_fresh(&self, uid: i64) -> bool {
        match self.map.get(&uid) {
            Some(entry) => entry.elapsed() < self.ttl,
            None => false,
        }
    }

    fn insert(&self, uid: i64) {
        self.map.insert(uid, Instant::now());
    }

    fn prune(&self) {
        let ttl = self.ttl;
        self.map.retain(|_, ts| ts.elapsed() < ttl);

        if self.map.len() > self.max_size {
            let mut entries: Vec<(i64, Instant)> =
                self.map.iter().map(|e| (*e.key(), *e.value())).collect();
            entries.sort_by_key(|(_, ts)| *ts);
            let to_drop = entries.len() - self.max_size;
            for (uid, _) in entries.iter().take(to_drop) {
                self.map.remove(uid);
            }
        }
    }

    fn len(&self) -> usize {
        self.map.len()
    }
}

pub(crate) fn kafka_tls_config(cfg: &mut ClientConfig) {
    cfg.set("enable.ssl.certificate.verification", "true");
    if let Ok(ca) = std::env::var("KAFKA_SSL_CA_LOCATION") {
        let ca = ca.trim();
        if !ca.is_empty() {
            cfg.set("ssl.ca.location", ca);
        }
    }
    if let Ok(algo) = std::env::var("KAFKA_SSL_ENDPOINT_IDENTIFICATION") {
        let algo = algo.trim();
        if !algo.is_empty() {
            cfg.set("ssl.endpoint.identification.algorithm", algo);
        }
    }
}

#[inline(always)]
fn extract_user_id_from_key(key: &[u8]) -> Option<i64> {
    if key.len() == 8 {
        let uid = i64::from_be_bytes(key.try_into().ok()?);
        if uid > 0 {
            Some(uid)
        } else {
            None
        }
    } else if key.len() == 4 {
        let uid = i32::from_be_bytes(key.try_into().ok()?) as i64;
        if uid > 0 {
            Some(uid)
        } else {
            None
        }
    } else {
        None
    }
}

#[inline(always)]
fn extract_user_id(payload: &[u8]) -> Option<i64> {
    if payload.len() < MIN_MSG_SIZE {
        return None;
    }

    if payload[..4] != ARROW_CONTINUATION {
        return None;
    }

    let mut cursor = Cursor::new(&payload[USERID_OFFSET..USERID_OFFSET + 8]);
    let uid = cursor.read_i64::<LittleEndian>().ok()?;

    if uid > 0 {
        Some(uid)
    } else {
        None
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    let args = Args::parse();
    let metrics = Arc::new(Metrics::new());
    let cooldown_cache = Arc::new(CooldownCache::new(
        Duration::from_secs(args.cooldown_cache_ttl_secs),
        args.cooldown_cache_max,
    ));

    let trace_emitter = Arc::new(TraceEmitter::new_active(TraceConfig::from_env()));
    if trace_emitter.enabled {
        info!(
            "Trace emitter ENABLED: sample_rate={}, pod={}",
            trace_emitter.sample_rate, trace_emitter.pod_name
        );
    } else {
        info!("Trace emitter DISABLED (set TRACE_ENABLED=1 to enable)");
    }
    let health_state = Arc::new(HealthState::new(
        Duration::from_secs(args.liveness_grace_secs),
        args.liveness_redis_errors_per_sec,
        args.liveness_min_writes_per_sec,
        args.liveness_min_consumed_per_sec,
    ));
    let start_time = Instant::now();

    let redis_url = format!("redis://{}:{}", args.redis_host, args.redis_port);
    let redis_client = redis::Client::open(redis_url.as_str())?;
    let mut redis_conn = redis::aio::ConnectionManager::new(redis_client).await?;
    info!(
        "Connected to Redis at {}:{} (ConnectionManager: auto-reconnect on broken pipe)",
        args.redis_host, args.redis_port
    );

    let group_instance_id = args
        .group_instance_id
        .clone()
        .or_else(|| std::env::var("HOSTNAME").ok())
        .filter(|s| !s.is_empty());

    let mut cfg = ClientConfig::new();
    cfg.set("bootstrap.servers", &args.kafka_bootstrap)
        .set("group.id", &args.kafka_group)
        .set("auto.offset.reset", "latest")
        .set("enable.auto.commit", "true")
        .set("auto.commit.interval.ms", "5000")
        .set("security.protocol", "SASL_SSL")
        .set("sasl.mechanism", "SCRAM-SHA-512")
        .set("sasl.username", &args.kafka_username)
        .set("sasl.password", &args.kafka_password)
        .set(
            "partition.assignment.strategy",
            &args.kafka_assignment_strategy,
        )
        .set(
            "session.timeout.ms",
            args.kafka_session_timeout_ms.to_string(),
        )
        .set(
            "heartbeat.interval.ms",
            (args.kafka_session_timeout_ms / 6).to_string(),
        )
        .set(
            "max.poll.interval.ms",
            args.kafka_max_poll_interval_ms.to_string(),
        )
        .set("fetch.min.bytes", "524288")
        .set("fetch.wait.max.ms", "100")
        .set("fetch.message.max.bytes", "10485760")
        .set("max.partition.fetch.bytes", "5242880")
        .set("queued.max.messages.kbytes", "2000000")
        .set("queued.min.messages", "1000000")
        .set("socket.receive.buffer.bytes", "4194304");

    kafka_tls_config(&mut cfg);

    if let Some(ref id) = group_instance_id {
        info!("Using static group membership: group.instance.id={id}");
        cfg.set("group.instance.id", id);
    } else {
        warn!(
            "No GROUP_INSTANCE_ID or HOSTNAME set. Falling back to dynamic membership; \
             pod restarts will trigger rebalances."
        );
    }

    let consumer: StreamConsumer = cfg.create()?;

    consumer.subscribe(&[&args.kafka_topic])?;
    info!(
        "Subscribed to {} (group={})",
        args.kafka_topic, args.kafka_group
    );

    let (tx, mut rx) = mpsc::channel::<i64>(100_000);

    let health_metrics = Arc::clone(&metrics);
    let health_cache = Arc::clone(&cooldown_cache);
    let health_state_route = Arc::clone(&health_state);
    let health_port = args.health_port;
    tokio::spawn(async move {
        use axum::http::StatusCode;
        use axum::{routing::get, Router};
        let liveness_state = Arc::clone(&health_state_route);
        let metrics_for_dump = Arc::clone(&health_metrics);
        let cache_for_dump = Arc::clone(&health_cache);
        let app = Router::new()
            .route(
                "/",
                get(move || {
                    let h = Arc::clone(&liveness_state);
                    async move {
                        let (healthy, reason) = h.evaluate();
                        if healthy {
                            (StatusCode::OK, reason)
                        } else {
                            error!("Liveness probe FAILED: {reason}");
                            (StatusCode::SERVICE_UNAVAILABLE, reason)
                        }
                    }
                }),
            )
            .route(
                "/metrics",
                get(move || {
                    let m = Arc::clone(&metrics_for_dump);
                    let c = Arc::clone(&cache_for_dump);
                    async move {
                        format!(
                            "consumed={}\nredis_writes={}\ngate_filtered={}\ndedup_collapsed={}\ncache_skipped={}\nleases_set={}\ncache_size={}\nparse_errors={}\nkafka_errors={}\nredis_errors={}\n",
                            m.consumed.load(Ordering::Relaxed),
                            m.redis_writes.load(Ordering::Relaxed),
                            m.gate_filtered.load(Ordering::Relaxed),
                            m.dedup_collapsed.load(Ordering::Relaxed),
                            m.cache_skipped.load(Ordering::Relaxed),
                            m.leases_set.load(Ordering::Relaxed),
                            c.len(),
                            m.parse_errors.load(Ordering::Relaxed),
                            m.kafka_errors.load(Ordering::Relaxed),
                            m.redis_errors.load(Ordering::Relaxed),
                        )
                    }
                }),
            );
        let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{health_port}"))
            .await
            .expect("health bind failed");
        info!("Health endpoint on :{health_port} (liveness=/ metrics=/metrics)");
        axum::serve(listener, app).await.ok();
    });

    let snap_metrics = Arc::clone(&metrics);
    let snap_state = Arc::clone(&health_state);
    let window_secs = args.liveness_window_secs.max(10);
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(window_secs));
        interval.tick().await;
        let mut prev_consumed = snap_metrics.consumed.load(Ordering::Relaxed);
        let mut prev_writes = snap_metrics.redis_writes.load(Ordering::Relaxed);
        let mut prev_errors = snap_metrics.redis_errors.load(Ordering::Relaxed);
        loop {
            interval.tick().await;
            let curr_consumed = snap_metrics.consumed.load(Ordering::Relaxed);
            let curr_writes = snap_metrics.redis_writes.load(Ordering::Relaxed);
            let curr_errors = snap_metrics.redis_errors.load(Ordering::Relaxed);
            let snap = HealthSnapshot {
                consumed_prev: prev_consumed,
                consumed_curr: curr_consumed,
                writes_prev: prev_writes,
                writes_curr: curr_writes,
                errors_prev: prev_errors,
                errors_curr: curr_errors,
                window_secs,
            };
            *snap_state.snapshots.lock().unwrap() = Some(snap);
            prev_consumed = curr_consumed;
            prev_writes = curr_writes;
            prev_errors = curr_errors;
        }
    });

    let log_metrics = Arc::clone(&metrics);
    let log_cache = Arc::clone(&cooldown_cache);
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(10));
        loop {
            interval.tick().await;
            let elapsed = start_time.elapsed().as_secs();
            let consumed = log_metrics.consumed.load(Ordering::Relaxed);
            let rate = if elapsed > 0 { consumed / elapsed } else { 0 };
            info!(
                "[Accumulator] {}s | consumed: {} ({}/sec) | redis_writes: {} | gate_filtered: {} | dedup_collapsed: {} | cache_skipped: {} | leases_set: {} | cache_size: {} | errors: parse={} kafka={} redis={}",
                elapsed,
                consumed,
                rate,
                log_metrics.redis_writes.load(Ordering::Relaxed),
                log_metrics.gate_filtered.load(Ordering::Relaxed),
                log_metrics.dedup_collapsed.load(Ordering::Relaxed),
                log_metrics.cache_skipped.load(Ordering::Relaxed),
                log_metrics.leases_set.load(Ordering::Relaxed),
                log_cache.len(),
                log_metrics.parse_errors.load(Ordering::Relaxed),
                log_metrics.kafka_errors.load(Ordering::Relaxed),
                log_metrics.redis_errors.load(Ordering::Relaxed),
            );
        }
    });

    let prune_cache = Arc::clone(&cooldown_cache);
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(60));
        loop {
            interval.tick().await;
            prune_cache.prune();
        }
    });

    let flush_metrics = Arc::clone(&metrics);
    let flush_cache = Arc::clone(&cooldown_cache);
    let flush_trace = Arc::clone(&trace_emitter);
    let redis_key = args.redis_key.clone();
    let batch_size = args.batch_size;
    let flush_interval = Duration::from_millis(args.flush_interval_ms);
    let lease_ttl_secs = args.enqueue_lease_ttl_secs;

    tokio::spawn(async move {
        let mut batch: Vec<i64> = Vec::with_capacity(batch_size);
        let mut last_flush = Instant::now();

        loop {
            match tokio::time::timeout(Duration::from_millis(50), rx.recv()).await {
                Ok(Some(uid)) => {
                    batch.push(uid);
                }
                Ok(None) => break,
                Err(_) => {}
            }

            let should_flush = batch.len() >= batch_size
                || (!batch.is_empty() && last_flush.elapsed() >= flush_interval);

            if !should_flush {
                continue;
            }

            let now_ts = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs_f64();

            let raw_count = batch.len();
            let unique: HashSet<i64> = batch.drain(..).collect();
            let unique_count = unique.len();
            if raw_count > unique_count {
                flush_metrics
                    .dedup_collapsed
                    .fetch_add((raw_count - unique_count) as u64, Ordering::Relaxed);
            }

            let mut need_redis: Vec<i64> = Vec::with_capacity(unique_count);
            let mut cache_skipped_now = 0u64;
            for uid in unique {
                if flush_cache.contains_fresh(uid) {
                    cache_skipped_now += 1;
                    if flush_trace.is_traced(uid) {
                        flush_trace.emit_flush_decision(uid, "cache_skipped", Some(now_ts), None);
                    }
                } else {
                    need_redis.push(uid);
                }
            }
            if cache_skipped_now > 0 {
                flush_metrics
                    .cache_skipped
                    .fetch_add(cache_skipped_now, Ordering::Relaxed);
                flush_metrics
                    .gate_filtered
                    .fetch_add(cache_skipped_now, Ordering::Relaxed);
            }

            if need_redis.is_empty() {
                last_flush = Instant::now();
                continue;
            }

            let gate_keys: Vec<String> = need_redis
                .iter()
                .map(|uid| format!("scored:{uid}"))
                .collect();
            let gate_results: Vec<Option<String>> = match redis::cmd("MGET")
                .arg(&gate_keys)
                .query_async::<Vec<Option<String>>>(&mut redis_conn)
                .await
            {
                Ok(r) => r,
                Err(e) => {
                    warn!("Gate MGET failed (fail-open): {e}");
                    flush_metrics.redis_errors.fetch_add(1, Ordering::Relaxed);
                    vec![None; need_redis.len()]
                }
            };

            let mut filtered: Vec<(f64, String)> = Vec::with_capacity(need_redis.len());
            let mut gated = 0u64;
            for (uid, scored) in need_redis.iter().zip(gate_results.iter()) {
                if let Some(val) = scored {
                    flush_cache.insert(*uid);
                    gated += 1;
                    if flush_trace.is_traced(*uid) {
                        let decision = if val == "leased" {
                            "gate_filtered_leased"
                        } else {
                            "gate_filtered_scored"
                        };
                        let detail = serde_json::to_string(&serde_json::json!({
                            "mget_value": val,
                        }))
                        .ok();
                        flush_trace.emit_flush_decision(*uid, decision, Some(now_ts), detail);
                    }
                } else {
                    filtered.push((now_ts, uid.to_string()));
                }
            }
            flush_metrics
                .gate_filtered
                .fetch_add(gated, Ordering::Relaxed);

            if filtered.is_empty() {
                last_flush = Instant::now();
                continue;
            }

            let zadd_result: Result<u64, _> = redis::cmd("ZADD")
                .arg(&redis_key)
                .arg("NX")
                .arg(
                    filtered
                        .iter()
                        .flat_map(|(score, member)| vec![score.to_string(), member.clone()])
                        .collect::<Vec<String>>(),
                )
                .query_async(&mut redis_conn)
                .await;

            match zadd_result {
                Ok(added) => {
                    flush_metrics
                        .redis_writes
                        .fetch_add(added, Ordering::Relaxed);
                    if flush_trace.enabled {
                        for (score, uid_str) in &filtered {
                            if let Ok(uid) = uid_str.parse::<i64>() {
                                if flush_trace.is_traced(uid) {
                                    flush_trace.emit_flush_decision(
                                        uid,
                                        "enqueued",
                                        Some(*score),
                                        None,
                                    );
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("Redis ZADD failed: {e}");
                    flush_metrics.redis_errors.fetch_add(1, Ordering::Relaxed);
                    if flush_trace.enabled {
                        for (_score, uid_str) in &filtered {
                            if let Ok(uid) = uid_str.parse::<i64>() {
                                if flush_trace.is_traced(uid) {
                                    flush_trace.emit_flush_decision(
                                        uid,
                                        "redis_error",
                                        Some(now_ts),
                                        Some(format!("{{\"error\":{:?}}}", e.to_string())),
                                    );
                                }
                            }
                        }
                    }
                }
            }

            if lease_ttl_secs > 0 {
                let mut pipe = redis::pipe();
                for (_score, uid_str) in &filtered {
                    pipe.cmd("SET")
                        .arg(format!("scored:{uid_str}"))
                        .arg("leased")
                        .arg("EX")
                        .arg(lease_ttl_secs)
                        .arg("NX")
                        .ignore();
                }
                match pipe.query_async::<()>(&mut redis_conn).await {
                    Ok(_) => {
                        flush_metrics
                            .leases_set
                            .fetch_add(filtered.len() as u64, Ordering::Relaxed);
                    }
                    Err(e) => {
                        warn!("Lease SET pipeline failed (fail-open): {e}");
                        flush_metrics.redis_errors.fetch_add(1, Ordering::Relaxed);
                    }
                }
            }

            last_flush = Instant::now();
        }
    });

    info!("Starting consumer loop...");
    loop {
        match consumer.recv().await {
            Ok(msg) => {
                metrics.consumed.fetch_add(1, Ordering::Relaxed);

                let uid = msg
                    .key()
                    .and_then(extract_user_id_from_key)
                    .or_else(|| msg.payload().and_then(extract_user_id));

                match uid {
                    Some(uid) => {
                        trace_emitter.maybe_sample_consume(uid, msg.partition(), msg.offset());

                        if tx.try_send(uid).is_err() {
                            if trace_emitter.is_traced(uid) {
                                trace_emitter.emit_try_send_full(
                                    uid,
                                    msg.partition(),
                                    msg.offset(),
                                );
                            }
                        }
                    }
                    None => {
                        metrics.parse_errors.fetch_add(1, Ordering::Relaxed);
                    }
                }

                if metrics
                    .consumed
                    .load(Ordering::Relaxed)
                    .is_multiple_of(10_000)
                {
                    let _ = consumer.commit_message(&msg, CommitMode::Async);
                }
            }
            Err(e) => {
                warn!("Kafka error: {e}");
                metrics.kafka_errors.fetch_add(1, Ordering::Relaxed);
            }
        }
    }
}
