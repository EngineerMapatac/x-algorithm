use prometheus::{CounterVec, Gauge, HistogramOpts, HistogramVec, Opts, Registry};

pub const STATUS_TOO_FRESH: &str = "too_fresh";
pub const STATUS_HIT: &str = "hit";
pub const STATUS_MISS: &str = "miss";
pub const STATUS_ERROR: &str = "error";

pub const BACKFILL_RESOLVED: &str = "resolved";
pub const BACKFILL_STILL_MISSING: &str = "still_missing";

pub const LATENCY_SUCCESS: &str = "success";
pub const LATENCY_ERROR: &str = "error";

#[derive(Clone)]
pub struct SidMetrics {
    pub lookup_count: CounterVec,
    pub lookup_latency_seconds: HistogramVec,
    pub backfill_total: CounterVec,
    pub missing_count: Gauge,
}

impl SidMetrics {
    pub fn new(registry: &Registry) -> anyhow::Result<Self> {
        let lookup_count = CounterVec::new(
            Opts::new(
                "phoenix_sid_lookup_count",
                "Per-post SID lookup outcomes from xai-recsys-sid",
            ),
            &["status"],
        )?;
        registry.register(Box::new(lookup_count.clone()))?;

        let lookup_latency_seconds = HistogramVec::new(
            HistogramOpts::new(
                "phoenix_sid_lookup_latency_seconds",
                "Latency of one bulk SID gRPC call",
            ),
            &["status"],
        )?;
        registry.register(Box::new(lookup_latency_seconds.clone()))?;

        let backfill_total = CounterVec::new(
            Opts::new(
                "phoenix_sid_backfill_total",
                "Posts re-looked-up for missing SIDs",
            ),
            &["status"],
        )?;
        registry.register(Box::new(backfill_total.clone()))?;

        let missing_count = Gauge::with_opts(Opts::new(
            "phoenix_sid_missing_count",
            "Current count of posts with empty SID in store",
        ))?;
        registry.register(Box::new(missing_count.clone()))?;

        Ok(Self {
            lookup_count,
            lookup_latency_seconds,
            backfill_total,
            missing_count,
        })
    }

    #[cfg(test)]
    pub fn for_tests() -> Self {
        Self::new(&Registry::new()).expect("metrics for tests")
    }
}
