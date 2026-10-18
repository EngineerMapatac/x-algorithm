use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use tracing::{info, warn};
use xai_manhattan::{Key, LkeySelector, NativeManhattanClient, Tenant};

use crate::limiter::LimiterClient;
use crate::manhattan::ttl_nanos_from_secs;
use crate::metrics;

pub const USER_SNAP_PKEY: &str = "global_rate_limit";
pub const POST_SNAP_PKEY: &str = "global_rate_limit:post";
const WINDOW_SECS: u64 = 86_400;
const SNAPSHOT_INTERVAL_SECS: u64 = 60;
const SNAPSHOT_TTL_SECS: u64 = WINDOW_SECS + 3_600;
const SCAN_LIMIT: u32 = 200_000;

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn snapshot_lkey(ts_secs: u64, interval: u64) -> String {
    format!("{:020}", (ts_secs / interval) * interval)
}

fn window_usage(counts: &[i64]) -> Option<(i64, i64)> {
    let first = *counts.first()?;
    let baseline = counts.iter().copied().max().unwrap_or(first);
    Some((baseline - first, baseline))
}

fn remaining(cap: i64, used: i64) -> i64 {
    (cap - used).max(0)
}

#[async_trait]
trait CounterSource: Send + Sync {
    async fn increment(&self) -> Option<i64>;
}

#[async_trait]
impl CounterSource for LimiterClient {
    async fn increment(&self) -> Option<i64> {
        self.increment_count().await
    }
}

#[async_trait]
trait SnapshotStore: Send + Sync {
    async fn scan(
        &self,
        pkey: &str,
        from: Option<&str>,
        to: Option<&str>,
        limit: u32,
    ) -> anyhow::Result<Vec<(String, String)>>;
    async fn put_with_ttl(
        &self,
        pkey: &str,
        lkey: &str,
        value: String,
        ttl_nanos: i64,
    ) -> anyhow::Result<()>;
    async fn delete(&self, pkey: &str, lkey: &str) -> anyhow::Result<()>;
}

struct ManhattanSnapshotStore {
    mh: Arc<NativeManhattanClient>,
    tenant: Tenant,
}

#[async_trait]
impl SnapshotStore for ManhattanSnapshotStore {
    async fn scan(
        &self,
        pkey: &str,
        from: Option<&str>,
        to: Option<&str>,
        limit: u32,
    ) -> anyhow::Result<Vec<(String, String)>> {
        let items = self
            .mh
            .scan(
                self.tenant.clone(),
                [pkey],
                LkeySelector::Range {
                    from: from.map(|f| Key::from_parts([f])),
                    to: to.map(|t| Key::from_parts([t])),
                },
                limit,
            )
            .await
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        Ok(items
            .iter()
            .filter_map(|it| {
                let lkey =
                    String::from_utf8(it.lkey().into_byte_vecs().into_iter().next()?).ok()?;
                let value = String::from_utf8(it.value().into_bytes()).ok()?;
                Some((lkey, value))
            })
            .collect())
    }

    async fn put_with_ttl(
        &self,
        pkey: &str,
        lkey: &str,
        value: String,
        ttl_nanos: i64,
    ) -> anyhow::Result<()> {
        self.mh
            .put_with_ttl(self.tenant.clone(), [pkey], [lkey], value, ttl_nanos)
            .await
            .map_err(|e| anyhow::anyhow!("{e}"))
    }

    async fn delete(&self, pkey: &str, lkey: &str) -> anyhow::Result<()> {
        self.mh
            .delete(self.tenant.clone(), [pkey], [lkey])
            .await
            .map_err(|e| anyhow::anyhow!("{e}"))
    }
}

#[derive(Clone, Copy, Default)]
struct Cached {
    rolling_tail: i64,
    baseline: i64,
}

#[derive(Clone)]
pub struct SlidingWindowLimiter {
    counter: Arc<dyn CounterSource>,
    store: Arc<dyn SnapshotStore>,
    snap_pkey: String,
    metric_label: &'static str,
    last_count: Arc<AtomicI64>,
    cached: Arc<RwLock<Cached>>,
    seeded: Arc<AtomicBool>,
    dirty: Arc<AtomicBool>,
    snapshots_present: Arc<AtomicBool>,
    fail_open: bool,
}

impl SlidingWindowLimiter {
    pub fn new(
        limiter: LimiterClient,
        mh: Arc<NativeManhattanClient>,
        tenant: Tenant,
        fail_open: bool,
        snap_pkey: impl Into<String>,
        metric_label: &'static str,
    ) -> Self {
        Self::from_parts(
            Arc::new(limiter),
            Arc::new(ManhattanSnapshotStore { mh, tenant }),
            fail_open,
            snap_pkey,
            metric_label,
        )
    }

    fn from_parts(
        counter: Arc<dyn CounterSource>,
        store: Arc<dyn SnapshotStore>,
        fail_open: bool,
        snap_pkey: impl Into<String>,
        metric_label: &'static str,
    ) -> Self {
        Self {
            counter,
            store,
            snap_pkey: snap_pkey.into(),
            metric_label,
            last_count: Arc::new(AtomicI64::new(0)),
            cached: Arc::new(RwLock::new(Cached::default())),
            seeded: Arc::new(AtomicBool::new(false)),
            dirty: Arc::new(AtomicBool::new(false)),
            snapshots_present: Arc::new(AtomicBool::new(false)),
            fail_open,
        }
    }

    pub async fn try_record(&self, cap: i64) -> bool {
        match self.increment_and_rolling().await {
            Some(rolling) => {
                metrics::RATE_LIMIT_REMAINING
                    .with_label_values(&[self.metric_label])
                    .set(remaining(cap, rolling));
                if !self.seeded.load(Ordering::Relaxed) {
                    return true;
                }
                rolling < cap
            }
            None => self.fail_open,
        }
    }

    #[cfg(test)]
    pub(crate) fn seed_for_test(&self) {
        self.seeded.store(true, Ordering::Relaxed);
    }

    pub fn stats(&self, cap: i64) -> (u32, u32, u32) {
        let used = self.rolling_cached();
        (
            used.max(0) as u32,
            cap.max(0) as u32,
            remaining(cap, used) as u32,
        )
    }

    pub async fn record_report(&self, cap: i64) -> Option<serde_json::Value> {
        let rolling = self.increment_and_rolling().await?;
        metrics::RATE_LIMIT_REMAINING
            .with_label_values(&[self.metric_label])
            .set(remaining(cap, rolling));
        let allowed = !self.seeded.load(Ordering::Relaxed) || rolling < cap;
        Some(serde_json::json!({
            "allowed": allowed,
            "used": rolling,
            "cap": cap,
            "remaining": remaining(cap, rolling),
        }))
    }

    pub async fn reset(&self) -> Option<u64> {
        let items = match self
            .store
            .scan(self.snap_pkey.as_str(), None, None, SCAN_LIMIT)
            .await
        {
            Ok(items) => items,
            Err(e) => {
                metrics::MANHATTAN_ERRORS_TOTAL.inc();
                warn!(pkey = %self.snap_pkey, "sliding-window reset scan failed: {e}");
                return None;
            }
        };

        let mut deleted = 0u64;
        for (lkey, _) in &items {
            if let Err(e) = self.store.delete(self.snap_pkey.as_str(), lkey).await {
                metrics::MANHATTAN_ERRORS_TOTAL.inc();
                warn!(pkey = %self.snap_pkey, "sliding-window reset delete failed for lkey={lkey}: {e}");
            } else {
                deleted += 1;
            }
        }

        let baseline = self.last_count.load(Ordering::Relaxed);
        *self.cached.write().unwrap_or_else(|e| e.into_inner()) = Cached {
            rolling_tail: 0,
            baseline,
        };
        self.seeded.store(true, Ordering::Relaxed);
        warn!(pkey = %self.snap_pkey, deleted, baseline, "sliding-window reset");
        Some(deleted)
    }

    pub fn spawn_background(&self) {
        let this = self.clone();
        tokio::spawn(async move {
            this.refresh().await;
            let mut tick = tokio::time::interval(Duration::from_secs(SNAPSHOT_INTERVAL_SECS));
            loop {
                tick.tick().await;
                this.write_snapshot().await;
                this.refresh().await;
            }
        });
    }

    async fn increment_and_rolling(&self) -> Option<i64> {
        let c_now = self.counter.increment().await?;
        let prev = self.last_count.swap(c_now, Ordering::Relaxed);
        self.dirty.store(true, Ordering::Relaxed);
        if c_now < prev {
            metrics::LIMITER_COUNTER_RESET_TOTAL
                .with_label_values(&[self.metric_label])
                .inc();
        }
        if !self.snapshots_present.load(Ordering::Relaxed) {
            self.cached
                .write()
                .unwrap_or_else(|e| e.into_inner())
                .baseline = c_now;
        }
        let cached = *self.cached.read().unwrap_or_else(|e| e.into_inner());
        let head = (c_now - cached.baseline).max(0);
        Some(cached.rolling_tail + head)
    }

    fn rolling_cached(&self) -> i64 {
        let cached = *self.cached.read().unwrap_or_else(|e| e.into_inner());
        let head = (self.last_count.load(Ordering::Relaxed) - cached.baseline).max(0);
        cached.rolling_tail + head
    }

    async fn write_snapshot(&self) {
        if !self.dirty.swap(false, Ordering::Relaxed) {
            return;
        }
        let c = self.last_count.load(Ordering::Relaxed);
        let lkey = snapshot_lkey(now_secs(), SNAPSHOT_INTERVAL_SECS);
        let ttl = ttl_nanos_from_secs(SNAPSHOT_TTL_SECS);
        if let Err(e) = self
            .store
            .put_with_ttl(self.snap_pkey.as_str(), lkey.as_str(), c.to_string(), ttl)
            .await
        {
            metrics::MANHATTAN_ERRORS_TOTAL.inc();
            warn!(pkey = %self.snap_pkey, "sliding-window snapshot write failed: {e}");
            self.dirty.store(true, Ordering::Relaxed);
        }
    }

    async fn refresh(&self) {
        let now = now_secs();
        let from = snapshot_lkey(now.saturating_sub(WINDOW_SECS), SNAPSHOT_INTERVAL_SECS);
        let to = snapshot_lkey(now, SNAPSHOT_INTERVAL_SECS);
        let items = match self
            .store
            .scan(
                self.snap_pkey.as_str(),
                Some(from.as_str()),
                Some(to.as_str()),
                SCAN_LIMIT,
            )
            .await
        {
            Ok(items) => items,
            Err(e) => {
                metrics::MANHATTAN_ERRORS_TOTAL.inc();
                warn!(pkey = %self.snap_pkey, "sliding-window scan failed: {e}");
                return;
            }
        };
        let counts: Vec<i64> = items
            .iter()
            .filter_map(|(_, v)| v.trim().parse().ok())
            .collect();
        let (rolling_tail, baseline) = match window_usage(&counts) {
            Some(usage) => usage,
            None => {
                if !self.seeded.load(Ordering::Relaxed) {
                    info!(
                        pkey = %self.snap_pkey,
                        "sliding-window: cold start (no snapshots), baselining at C={}",
                        self.last_count.load(Ordering::Relaxed)
                    );
                }
                (0, self.last_count.load(Ordering::Relaxed))
            }
        };
        self.snapshots_present
            .store(!counts.is_empty(), Ordering::Relaxed);
        *self.cached.write().unwrap_or_else(|e| e.into_inner()) = Cached {
            rolling_tail,
            baseline,
        };
        self.seeded.store(true, Ordering::Relaxed);
        warn!(
            pkey = %self.snap_pkey,
            from = %from,
            to = %to,
            snapshots = counts.len(),
            rolling_tail,
            baseline,
            "sliding-window refresh"
        );
    }
}

#[cfg(test)]
pub(crate) fn counting_limiter_for_test() -> (SlidingWindowLimiter, Arc<AtomicI64>) {
    struct Counting(Arc<AtomicI64>);
    #[async_trait]
    impl CounterSource for Counting {
        async fn increment(&self) -> Option<i64> {
            Some(self.0.fetch_add(1, Ordering::Relaxed) + 1)
        }
    }
    struct NoStore;
    #[async_trait]
    impl SnapshotStore for NoStore {
        async fn scan(
            &self,
            _pkey: &str,
            _from: Option<&str>,
            _to: Option<&str>,
            _limit: u32,
        ) -> anyhow::Result<Vec<(String, String)>> {
            Ok(Vec::new())
        }
        async fn put_with_ttl(
            &self,
            _pkey: &str,
            _lkey: &str,
            _value: String,
            _ttl_nanos: i64,
        ) -> anyhow::Result<()> {
            Ok(())
        }
        async fn delete(&self, _pkey: &str, _lkey: &str) -> anyhow::Result<()> {
            Ok(())
        }
    }
    let tickets = Arc::new(AtomicI64::new(0));
    let limiter = SlidingWindowLimiter::from_parts(
        Arc::new(Counting(tickets.clone())),
        Arc::new(NoStore),
        true,
        USER_SNAP_PKEY,
        "user",
    );
    (limiter, tickets)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::collections::VecDeque;
    use std::sync::Mutex;

    use super::*;

    #[test]
    fn window_usage_monotone_equals_last_minus_first() {
        assert_eq!(window_usage(&[10, 13, 20, 100]), Some((90, 100)));
    }

    #[test]
    fn window_usage_dip_and_recover_adds_no_phantom_usage() {
        let incident = [4_650_000, 4_800_000, 5_000_000, 12, 5_000_000, 40];
        assert_eq!(window_usage(&incident), Some((350_000, 5_000_000)));
    }

    #[test]
    fn window_usage_true_reset_undercounts_never_overcounts() {
        assert_eq!(window_usage(&[10, 100, 5, 30]), Some((90, 100)));
    }

    #[test]
    fn window_usage_oscillation_bounded_by_range() {
        assert_eq!(window_usage(&[50, 10, 60, 20, 55]), Some((10, 60)));
    }

    #[test]
    fn window_usage_matches_old_math_on_monotone_series() {
        let series: &[&[i64]] = &[
            &[0, 1, 2, 3],
            &[42],
            &[10, 13, 20, 100],
            &[4_650_000, 4_800_000, 5_000_000],
            &[7, 7, 7],
        ];
        for counts in series {
            let old_tail: i64 = counts.windows(2).map(|w| (w[1] - w[0]).max(0)).sum();
            let old_baseline = *counts.last().unwrap();
            assert_eq!(
                window_usage(counts),
                Some((old_tail, old_baseline)),
                "diverged from old math on healthy series {counts:?}"
            );
        }
    }

    #[test]
    fn window_usage_empty_and_single() {
        assert_eq!(window_usage(&[]), None);
        assert_eq!(window_usage(&[42]), Some((0, 42)));
    }

    #[test]
    fn remaining_clamps_at_zero() {
        assert_eq!(remaining(500_000, 100), 499_900);
        assert_eq!(remaining(500_000, 500_000), 0);
        assert_eq!(remaining(500_000, 600_000), 0);
    }

    #[test]
    fn snapshot_lkey_aligns_to_interval() {
        assert_eq!(snapshot_lkey(125, 60), snapshot_lkey(179, 60));
        assert_ne!(snapshot_lkey(125, 60), snapshot_lkey(185, 60));
    }

    #[test]
    fn snapshot_lkey_is_fixed_width_and_lexically_ordered() {
        let a = snapshot_lkey(120, 60);
        let b = snapshot_lkey(1_000_000, 60);
        assert_eq!(a.len(), 20);
        assert_eq!(b.len(), 20);
        assert!(a < b);
    }

    #[derive(Default)]
    struct MemStore {
        rows: Mutex<BTreeMap<(String, String), String>>,
    }

    #[async_trait]
    impl SnapshotStore for MemStore {
        async fn scan(
            &self,
            pkey: &str,
            from: Option<&str>,
            to: Option<&str>,
            _limit: u32,
        ) -> anyhow::Result<Vec<(String, String)>> {
            Ok(self
                .rows
                .lock()
                .unwrap()
                .iter()
                .filter(|((p, l), _)| {
                    p == pkey
                        && from.is_none_or(|f| l.as_str() >= f)
                        && to.is_none_or(|t| l.as_str() <= t)
                })
                .map(|((_, l), v)| (l.clone(), v.clone()))
                .collect())
        }

        async fn put_with_ttl(
            &self,
            pkey: &str,
            lkey: &str,
            value: String,
            _ttl_nanos: i64,
        ) -> anyhow::Result<()> {
            self.rows
                .lock()
                .unwrap()
                .insert((pkey.to_string(), lkey.to_string()), value);
            Ok(())
        }

        async fn delete(&self, pkey: &str, lkey: &str) -> anyhow::Result<()> {
            self.rows
                .lock()
                .unwrap()
                .remove(&(pkey.to_string(), lkey.to_string()));
            Ok(())
        }
    }

    struct ScriptedCounter {
        script: Mutex<VecDeque<i64>>,
    }

    impl ScriptedCounter {
        fn new(values: impl IntoIterator<Item = i64>) -> Arc<Self> {
            Arc::new(Self {
                script: Mutex::new(values.into_iter().collect()),
            })
        }
    }

    #[async_trait]
    impl CounterSource for ScriptedCounter {
        async fn increment(&self) -> Option<i64> {
            self.script.lock().unwrap().pop_front()
        }
    }

    const CAP: i64 = 2_000_000;
    const PKEY: &str = "global_rate_limit:post";

    fn pod(counter: Arc<ScriptedCounter>, store: Arc<MemStore>) -> SlidingWindowLimiter {
        SlidingWindowLimiter::from_parts(counter, store, true, PKEY, "post")
    }

    async fn seed(store: &MemStore, age_secs: u64, value: i64) {
        let lkey = snapshot_lkey(now_secs().saturating_sub(age_secs), SNAPSHOT_INTERVAL_SECS);
        store
            .put_with_ttl(PKEY, &lkey, value.to_string(), 0)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn incident_replay_poisoned_series_does_not_wedge_the_cap() {
        let store = Arc::new(MemStore::default());
        seed(&store, 300, 4_650_000).await;
        seed(&store, 240, 4_800_000).await;
        seed(&store, 180, 5_000_000).await;
        seed(&store, 120, 12).await;
        seed(&store, 60, 5_000_000).await;

        let pod_a = pod(ScriptedCounter::new([]), store.clone());
        pod_a.refresh().await;

        let (used, cap, rem) = pod_a.stats(CAP);
        assert_eq!(used, 350_000, "phantom usage from the dip-recover");
        assert_eq!(cap, 2_000_000);
        assert_eq!(rem, 1_650_000, "remaining must not cliff to 0");
    }

    #[tokio::test]
    async fn post_reset_observation_keeps_allowing_and_counts_reset() {
        let store = Arc::new(MemStore::default());
        seed(&store, 240, 4_650_000).await;
        seed(&store, 120, 5_000_000).await;

        let pod_a = pod(ScriptedCounter::new([5_000_010, 12]), store.clone());
        pod_a.refresh().await;

        assert!(pod_a.try_record(CAP).await, "pre-reset: well under cap");
        let before = metrics::LIMITER_COUNTER_RESET_TOTAL
            .with_label_values(&["post"])
            .get();
        assert!(
            pod_a.try_record(CAP).await,
            "post-reset observation must not deny"
        );
        let after = metrics::LIMITER_COUNTER_RESET_TOTAL
            .with_label_values(&["post"])
            .get();
        assert_eq!(after - before, 1, "backwards step must tick the metric");

        let (used, _, rem) = pod_a.stats(CAP);
        assert_eq!(used, 350_000);
        assert_eq!(rem, 1_650_000);
    }

    #[tokio::test]
    async fn stale_pod_stops_writing_snapshots() {
        let store = Arc::new(MemStore::default());
        let stale = pod(ScriptedCounter::new([5_000_000]), store.clone());

        stale.write_snapshot().await;
        assert!(
            store.scan(PKEY, None, None, 100).await.unwrap().is_empty(),
            "fresh pod must not persist its placeholder"
        );

        stale.increment_and_rolling().await.unwrap();
        stale.write_snapshot().await;
        assert_eq!(store.scan(PKEY, None, None, 100).await.unwrap().len(), 1);

        let fresh = pod(ScriptedCounter::new([12]), store.clone());
        fresh.refresh().await;
        fresh.increment_and_rolling().await.unwrap();
        fresh.write_snapshot().await;

        let rows_before = store.scan(PKEY, None, None, 100).await.unwrap();
        stale.write_snapshot().await;
        let rows_after = store.scan(PKEY, None, None, 100).await.unwrap();
        assert_eq!(
            rows_before, rows_after,
            "stale pod re-injected a snapshot without a fresh observation"
        );
    }

    #[tokio::test]
    async fn refresh_does_not_resurrect_stale_high_into_last_count() {
        let store = Arc::new(MemStore::default());
        seed(&store, 120, 5_000_000).await;

        let pod_a = pod(ScriptedCounter::new([12]), store.clone());
        pod_a.refresh().await;
        pod_a.increment_and_rolling().await.unwrap();
        pod_a.refresh().await;

        assert_eq!(
            pod_a.last_count.load(Ordering::Relaxed),
            12,
            "scanned stale high must not overwrite an own observation"
        );
        pod_a.write_snapshot().await;
        let rows = store.scan(PKEY, None, None, 100).await.unwrap();
        assert!(
            rows.iter().any(|(_, v)| v == "12"),
            "pod must persist its own fresh observation"
        );
        assert_eq!(
            rows.iter().filter(|(_, v)| v == "5000000").count(),
            1,
            "no additional stale-high rows may appear"
        );
    }

    #[tokio::test]
    async fn true_reset_counts_fresh_after_window_slides() {
        let store = Arc::new(MemStore::default());
        seed(&store, WINDOW_SECS + 600, 5_000_000).await;
        seed(&store, 180, 12).await;
        seed(&store, 60, 40).await;

        let pod_a = pod(ScriptedCounter::new([]), store.clone());
        pod_a.refresh().await;
        let (used, _, rem) = pod_a.stats(CAP);
        assert_eq!(used, 28, "post-reset ramp counts once highs age out");
        assert_eq!(rem as i64, CAP - 28);
    }

    #[tokio::test]
    async fn reset_clears_poisoned_series_and_rebaselines() {
        let store = Arc::new(MemStore::default());
        seed(&store, 180, 5_000_000).await;
        seed(&store, 120, 12).await;
        seed(&store, 60, 5_000_000).await;

        let pod_a = pod(ScriptedCounter::new([5_000_001]), store.clone());
        pod_a.refresh().await;
        pod_a.increment_and_rolling().await.unwrap();

        let deleted = pod_a.reset().await.expect("reset must succeed");
        assert_eq!(deleted, 3);
        assert!(store.scan(PKEY, None, None, 100).await.unwrap().is_empty());
        let (used, _, rem) = pod_a.stats(CAP);
        assert_eq!(used, 0);
        assert_eq!(rem as i64, CAP);
    }
}
