use std::sync::Arc;
use std::time::{Duration, Instant};

use log::{info, warn};
use tokio_util::sync::CancellationToken;

use super::client::SidClient;
use super::metrics::{BACKFILL_RESOLVED, BACKFILL_STILL_MISSING, SidMetrics};
use crate::store::base::BaseSnapshotStore;

#[derive(Debug, Clone)]
pub struct BackfillConfig {
    pub interval: Duration,
    pub batch_size: usize,
            pub max_per_cycle: usize,
        pub max_age_hours: f64,
}

impl Default for BackfillConfig {
    fn default() -> Self {
        Self {
            interval: Duration::from_secs(30),
            batch_size: 5000,
            max_per_cycle: 800_000,
            max_age_hours: 1.0,
        }
    }
}

pub fn spawn_backfill_loop(
    store: Arc<BaseSnapshotStore>,
    sid_client: Arc<SidClient>,
    metrics: SidMetrics,
    config: BackfillConfig,
    cancel: CancellationToken,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        run_backfill_loop(store, sid_client, metrics, config, cancel).await;
    })
}

pub async fn run_backfill_loop(
    store: Arc<BaseSnapshotStore>,
    sid_client: Arc<SidClient>,
    metrics: SidMetrics,
    config: BackfillConfig,
    cancel: CancellationToken,
) {
    info!(
        "SID backfill loop starting: interval={:?}, batch_size={}, max_per_cycle={}, max_age_hours={}",
        config.interval, config.batch_size, config.max_per_cycle, config.max_age_hours
    );

    if config.batch_size == 0 {
        info!("SID backfill disabled (batch_size = 0)");
        return;
    }

    loop {
        tokio::select! {
            _ = tokio::time::sleep(config.interval) => {}
            _ = cancel.cancelled() => {
                info!("SID backfill loop cancelled");
                return;
            }
        }

        let cycle_start = Instant::now();

        let missing_ids = store.get_missing_sid_post_ids(config.max_age_hours).await;
        metrics.missing_count.set(missing_ids.len() as f64);
        if missing_ids.is_empty() {
            continue;
        }

        let total_to_send = missing_ids.len().min(config.max_per_cycle);
        let mut total_resolved = 0usize;
        let mut total_sent = 0usize;

        for chunk in missing_ids[..total_to_send].chunks(config.batch_size) {
            if cancel.is_cancelled() {
                return;
            }
            let resolved = sid_client.lookup_sids(chunk, None).await;
            let mut resolved_map = std::collections::HashMap::with_capacity(resolved.len());
            for &pid in chunk {
                match resolved.get(&pid) {
                    Some(codes) if !codes.is_empty() => {
                        resolved_map.insert(pid, codes.clone());
                        metrics
                            .backfill_total
                            .with_label_values(&[BACKFILL_RESOLVED])
                            .inc();
                    }
                    _ => {
                        metrics
                            .backfill_total
                            .with_label_values(&[BACKFILL_STILL_MISSING])
                            .inc();
                    }
                }
            }
            if !resolved_map.is_empty()
                && let Err(e) = store.update_sids(resolved_map.clone()).await
            {
                warn!("update_sids failed (will retry next cycle): {e}");
            }
            total_resolved += resolved_map.len();
            total_sent += chunk.len();
        }

        let elapsed = cycle_start.elapsed();
        if total_sent >= 1000 {
            let qps = total_sent as f64 / elapsed.as_secs_f64().max(0.001);
            info!(
                "SID backfill: resolved {total_resolved}/{total_sent} in {elapsed:?} ({qps:.0} qps) (total missing: {})",
                missing_ids.len()
            );
        }
    }
}
