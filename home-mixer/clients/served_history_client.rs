use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use tonic::async_trait;
use xai_manhattan::s2s::S2sConfig;
use xai_manhattan::{ConsistencyLevel, LkeySelector, NativeManhattanClient, Tenant};
use xai_x_thrift::served_history::ServedHistory;

use crate::clients::s2s::{S2S_CHAIN_PATH, S2S_CRT_PATH, S2S_KEY_PATH};
use crate::models::query::RequestType;

const MH_TIMEOUT: Duration = Duration::from_millis(300);
const SCAN_LIMIT: u32 = 100;

#[derive(Debug, Clone, Copy, strum::Display)]
#[strum(serialize_all = "lowercase")]
pub enum TimelineType {
    Home,
    #[strum(serialize = "homelatest")]
    HomeLatest,
    #[strum(serialize = "rankedfollowing")]
    RankedFollowing,
    Communities,
    Reply,
}

impl TimelineType {
    pub fn for_request(request_type: RequestType) -> Self {
        match request_type {
            RequestType::RankedFollowing => TimelineType::RankedFollowing,
            RequestType::Following => TimelineType::HomeLatest,
            _ => TimelineType::Home,
        }
    }
}

fn timeline_pkey(timeline_type: TimelineType, user_id: u64, client_platform: i32) -> Vec<Vec<u8>> {
    let timeline_id = format!("{timeline_type}-{user_id}");
    vec![
        timeline_id.into_bytes(),
        client_platform.to_be_bytes().to_vec(),
    ]
}

#[async_trait]
pub trait ServedHistoryClient: Send + Sync {
    async fn get_recent(
        &self,
        user_id: u64,
        timeline_type: TimelineType,
        client_platform: i32,
    ) -> Result<Vec<ServedHistory>>;

    async fn put(
        &self,
        user_id: u64,
        timeline_type: TimelineType,
        client_platform: i32,
        served_time_ms: i64,
        history: &ServedHistory,
    ) -> Result<()>;

    async fn delete(
        &self,
        user_id: u64,
        timeline_type: TimelineType,
        client_platform: i32,
        served_times_ms: &[i64],
    ) -> Result<()>;
}

pub struct ProdServedHistoryClient {
    client: Arc<NativeManhattanClient>,
    tenant: Tenant,
}

impl ProdServedHistoryClient {
    pub async fn new(datacenter: &str) -> Result<Self> {
        let tenant = Tenant {
            cluster: "omega".to_string(),
            app_id: "timeline_persistence".to_string(),
            dataset: "timeline_response_batches_v5".to_string(),
        };
        let s2s = S2sConfig {
            client_cert_path: S2S_CRT_PATH.clone(),
            client_key_path: S2S_KEY_PATH.clone(),
            ca_cert_path: S2S_CHAIN_PATH.clone(),
        };
        let client = Arc::new(
            NativeManhattanClient::builder_from_tenant_s2s(&tenant, datacenter, s2s)
                .timeout(MH_TIMEOUT)
                .early_terminate_empty_scan(true)
                .build()
                .await
                .map_err(|e| {
                    anyhow::anyhow!("Failed to create native MH client for served history: {e}")
                })?,
        );
        Ok(Self { client, tenant })
    }
}

#[async_trait]
impl ServedHistoryClient for ProdServedHistoryClient {
    async fn get_recent(
        &self,
        user_id: u64,
        timeline_type: TimelineType,
        client_platform: i32,
    ) -> Result<Vec<ServedHistory>> {
        let pkey = timeline_pkey(timeline_type, user_id, client_platform);
        let range = LkeySelector::Range {
            from: None,
            to: None,
        };

        let items = self
            .client
            .scan_with_consistency(
                self.tenant.clone(),
                pkey,
                range,
                SCAN_LIMIT,
                ConsistencyLevel::SoftDcReadMyWrites,
            )
            .await
            .map_err(|e| anyhow::anyhow!("MH scan failed: {e}"))?;

        let mut entries = Vec::with_capacity(items.len());
        for item in &items {
            let value = item.value();
            let bytes = value.as_bytes();
            match xai_x_thrift::deserialize_compact::<ServedHistory>(bytes) {
                Ok(mut history) => {
                    let lkey_parts = item.lkey().into_byte_vecs();
                    if let Some(part) = lkey_parts.first()
                        && part.len() >= 8
                    {
                        let inverted = i64::from_be_bytes(part[..8].try_into().unwrap());
                        history.served_time_ms = Some(i64::MAX - inverted);
                    }
                    entries.push(history);
                }
                Err(e) => {
                    tracing::warn!("Failed to deserialize ServedHistory: {e}");
                }
            }
        }

        Ok(entries)
    }

    async fn put(
        &self,
        user_id: u64,
        timeline_type: TimelineType,
        client_platform: i32,
        served_time_ms: i64,
        history: &ServedHistory,
    ) -> Result<()> {
        let pkey = timeline_pkey(timeline_type, user_id, client_platform);
        let lkey = vec![(i64::MAX - served_time_ms).to_be_bytes().to_vec()];
        let value = xai_x_thrift::serialize_compact(history)
            .map_err(|e| anyhow::anyhow!("failed to serialize ServedHistory: {e}"))?;

        self.client
            .put(self.tenant.clone(), pkey, lkey, value)
            .await
            .map_err(|e| anyhow::anyhow!("MH put failed: {e}"))
    }

    async fn delete(
        &self,
        user_id: u64,
        timeline_type: TimelineType,
        client_platform: i32,
        served_times_ms: &[i64],
    ) -> Result<()> {
        let pkey = timeline_pkey(timeline_type, user_id, client_platform);
        for &ts in served_times_ms {
            let lkey = vec![(i64::MAX - ts).to_be_bytes().to_vec()];
            self.client
                .delete(self.tenant.clone(), pkey.clone(), lkey)
                .await
                .map_err(|e| anyhow::anyhow!("MH delete failed: {e}"))?;
        }
        Ok(())
    }
}

pub struct MockServedHistoryClient;

#[async_trait]
impl ServedHistoryClient for MockServedHistoryClient {
    async fn get_recent(
        &self,
        _user_id: u64,
        _timeline_type: TimelineType,
        _client_platform: i32,
    ) -> Result<Vec<ServedHistory>> {
        Ok(vec![])
    }

    async fn put(
        &self,
        _user_id: u64,
        _timeline_type: TimelineType,
        _client_platform: i32,
        _served_time_ms: i64,
        _history: &ServedHistory,
    ) -> Result<()> {
        Ok(())
    }

    async fn delete(
        &self,
        _user_id: u64,
        _timeline_type: TimelineType,
        _client_platform: i32,
        _served_times_ms: &[i64],
    ) -> Result<()> {
        Ok(())
    }
}
