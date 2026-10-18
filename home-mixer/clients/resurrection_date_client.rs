use std::time::{Duration, Instant};

use tonic::async_trait;
use xai_manhattan::{s2s::S2sConfig, NativeManhattanClient, Tenant};
use xai_stats_receiver::{global_stats_receiver, HistogramBuckets};
use xai_x_thrift::user_latest_activity::UserResurrectionRecord;

use crate::clients::s2s::{S2S_CHAIN_PATH, S2S_CRT_PATH, S2S_KEY_PATH};

const MH_CLUSTER: &str = "omega";
const MH_APP_ID: &str = "home_user_latest_activity";
const MH_DATASET: &str = "user_latest_resurrection";
const MH_TIMEOUT: Duration = Duration::from_millis(100);
const METRIC_NAME: &str = "ResurrectionDateClient";

const MIN_RESURRECTION_DORMANCY_MS: i64 = 30 * 24 * 60 * 60 * 1000;

fn tenant() -> Tenant {
    Tenant {
        cluster: MH_CLUSTER.to_string(),
        app_id: MH_APP_ID.to_string(),
        dataset: MH_DATASET.to_string(),
    }
}

fn user_id_to_pkey_bytes(user_id: i64) -> Vec<u8> {
    user_id.to_be_bytes().to_vec()
}

fn decode_resurrection_timestamp_ms(bytes: &[u8]) -> Option<i64> {
    let record = xai_x_thrift::deserialize_binary::<UserResurrectionRecord>(bytes).ok()?;
    match record.previous_visit_timestamp_ms {
        Some(prev) if record.resurrection_timestamp_ms - prev < MIN_RESURRECTION_DORMANCY_MS => {
            None
        }
        _ => Some(record.resurrection_timestamp_ms),
    }
}

pub fn compute_resurrection_fs_fields(
    now_ms: i64,
    resurrection_time_ms: Option<i64>,
) -> (Option<i64>, Option<i64>) {
    let Some(ts) = resurrection_time_ms.filter(|&t| t >= 0) else {
        return (None, None);
    };
    let ms_per_day = 86_400_000i64;
    (Some(ts / ms_per_day), Some((now_ms - ts) / ms_per_day))
}

fn record_stats(result: &str, latency_ms: f64) {
    let Some(receiver) = global_stats_receiver() else {
        return;
    };
    receiver.incr(METRIC_NAME, &[("result", result)], 1);
    receiver.observe(
        METRIC_NAME,
        &[],
        latency_ms,
        HistogramBuckets::Bucket50To500,
    );
}

#[async_trait]
pub trait ResurrectionDateClient: Send + Sync {
    async fn fetch(&self, user_id: u64) -> Result<Option<i64>, String>;
}

pub struct ProdResurrectionDateClient {
    mh: NativeManhattanClient,
}

impl ProdResurrectionDateClient {
    pub async fn new(datacenter: &str) -> anyhow::Result<Self> {
        let s2s = S2sConfig {
            client_cert_path: S2S_CRT_PATH.clone(),
            client_key_path: S2S_KEY_PATH.clone(),
            ca_cert_path: S2S_CHAIN_PATH.clone(),
        };
        let mh = NativeManhattanClient::builder_from_tenant_s2s(&tenant(), datacenter, s2s)
            .timeout(MH_TIMEOUT)
            .retries(1)
            .multiplexed()
            .no_batch()
            .build()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to create ResurrectionDateClient: {e}"))?;
        Ok(Self { mh })
    }
}

#[async_trait]
impl ResurrectionDateClient for ProdResurrectionDateClient {
    async fn fetch(&self, user_id: u64) -> Result<Option<i64>, String> {
        let pkey = vec![user_id_to_pkey_bytes(user_id as i64)];
        let lkey: Vec<Vec<u8>> = vec![];

        let start = Instant::now();
        let item = self.mh.get(tenant(), pkey, lkey).await;
        let latency_ms = start.elapsed().as_millis() as f64;

        match item {
            Ok(Some(item)) => match decode_resurrection_timestamp_ms(item.value().as_bytes()) {
                Some(ts) => {
                    record_stats("hit", latency_ms);
                    Ok(Some(ts))
                }
                None => {
                    record_stats("decode_error", latency_ms);
                    Ok(None)
                }
            },
            Ok(None) => {
                record_stats("miss", latency_ms);
                Ok(None)
            }
            Err(e) => {
                record_stats("error", latency_ms);
                Err(format!("failed to get resurrection date: {e}"))
            }
        }
    }
}

pub struct MockResurrectionDateClient;

#[async_trait]
impl ResurrectionDateClient for MockResurrectionDateClient {
    async fn fetch(&self, _user_id: u64) -> Result<Option<i64>, String> {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DAY_MS: i64 = 86_400_000;

    fn encoded_record(resurrection_ts: i64, previous_visit_ms: Option<i64>) -> Vec<u8> {
        let record = UserResurrectionRecord {
            user_id: 1,
            resurrection_timestamp_ms: resurrection_ts,
            previous_visit_timestamp_ms: previous_visit_ms,
            client_application_id: Some(100),
            country_id: Some("US".to_string()),
            signup_country_id: Some("US".to_string()),
        };
        xai_x_thrift::serialize_binary(&record).unwrap()
    }

    #[test]
    fn counts_when_dormant_at_least_30d() {
        let ts = 1_700_000_000_000i64;
        let bytes = encoded_record(ts, Some(ts - 31 * DAY_MS));
        assert_eq!(decode_resurrection_timestamp_ms(&bytes), Some(ts));
    }

    #[test]
    fn counts_at_exactly_30d_boundary() {
        let ts = 1_700_000_000_000i64;
        let bytes = encoded_record(ts, Some(ts - 30 * DAY_MS));
        assert_eq!(decode_resurrection_timestamp_ms(&bytes), Some(ts));
    }

    #[test]
    fn counts_when_no_previous_visit() {
        let ts = 1_700_000_000_000i64;
        let bytes = encoded_record(ts, None);
        assert_eq!(decode_resurrection_timestamp_ms(&bytes), Some(ts));
    }

    #[test]
    fn skips_when_visit_within_30d() {
        let ts = 1_700_000_000_000i64;
        let bytes = encoded_record(ts, Some(ts - 10 * DAY_MS));
        assert_eq!(decode_resurrection_timestamp_ms(&bytes), None);
    }

    #[test]
    fn incomplete_record_returns_none() {
        let bytes: &[u8] = &[
            0x0a, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00,
        ];
        assert_eq!(decode_resurrection_timestamp_ms(bytes), None);
    }

    #[test]
    fn fs_fields_from_timestamp() {
        let ms_per_day = 86_400_000i64;
        let ts = 10 * ms_per_day;
        let now = ts + 3 * ms_per_day;
        assert_eq!(
            compute_resurrection_fs_fields(now, Some(ts)),
            (Some(10), Some(3))
        );
        assert_eq!(compute_resurrection_fs_fields(now, None), (None, None));
    }
}
