use std::time::Duration;
use tonic::async_trait;
use xai_core_entities::s2s::{S2S_CHAIN_PATH, S2S_CRT_PATH, S2S_KEY_PATH};
use xai_manhattan::s2s::S2sConfig;
use xai_manhattan::{NativeManhattanClient, Tenant};
use xai_x_thrift::non_polling_timestamps::NonPollingTimestamps;

const CLUSTER: &str = "omega";
const APP_ID: &str = "timelineservice_user_session_store";
const DATASET: &str = "tls_user_session_store";
const DEFAULT_TIMEOUT: Duration = Duration::from_millis(500);
const DEFAULT_RETRIES: usize = 1;

const NON_POLLING_TIMES_DATASET_ID: i32 = 1;
const VERSION_NUM_V1: i32 = 1;

fn tenant() -> Tenant {
    Tenant {
        cluster: CLUSTER.to_string(),
        app_id: APP_ID.to_string(),
        dataset: DATASET.to_string(),
    }
}

fn lkey() -> Vec<Vec<u8>> {
    vec![
        NON_POLLING_TIMES_DATASET_ID.to_be_bytes().to_vec(),
        VERSION_NUM_V1.to_be_bytes().to_vec(),
    ]
}

fn pkey(user_id: u64) -> Vec<Vec<u8>> {
    vec![(user_id as i64).to_be_bytes().to_vec()]
}

#[async_trait]
pub trait PastRequestTimestampsClient: Send + Sync {
    async fn get(&self, user_id: u64) -> Result<Option<NonPollingTimestamps>, String>;
    async fn put(&self, user_id: u64, npt: &NonPollingTimestamps) -> Result<(), String>;
}

pub struct ProdPastRequestTimestampsClient {
    client: NativeManhattanClient,
}

impl ProdPastRequestTimestampsClient {
    pub async fn new(dc: &str) -> anyhow::Result<Self> {
        let s2s = S2sConfig {
            client_cert_path: S2S_CRT_PATH.clone(),
            client_key_path: S2S_KEY_PATH.clone(),
            ca_cert_path: S2S_CHAIN_PATH.clone(),
        };
        let client = NativeManhattanClient::builder_from_tenant_s2s(&tenant(), dc, s2s)
            .timeout(DEFAULT_TIMEOUT)
            .retries(DEFAULT_RETRIES)
            .build()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to create PastRequestTimestampsClient: {e}"))?;
        Ok(Self { client })
    }
}

#[async_trait]
impl PastRequestTimestampsClient for ProdPastRequestTimestampsClient {
    async fn get(&self, user_id: u64) -> Result<Option<NonPollingTimestamps>, String> {
        let item = self
            .client
            .get(tenant(), pkey(user_id), lkey())
            .await
            .map_err(|e| format!("failed to get non_polling_timestamps: {e}"))?;

        match item {
            Some(it) => {
                let npt = xai_x_thrift::deserialize_compact::<NonPollingTimestamps>(
                    it.value().as_bytes(),
                )
                .map_err(|e| format!("failed to deserialize NonPollingTimestamps: {e}"))?;
                Ok(Some(npt))
            }
            None => Ok(None),
        }
    }

    async fn put(&self, user_id: u64, npt: &NonPollingTimestamps) -> Result<(), String> {
        let value = xai_x_thrift::serialize_compact(npt)
            .map_err(|e| format!("failed to serialize NonPollingTimestamps: {e}"))?;

        self.client
            .put(tenant(), pkey(user_id), lkey(), value)
            .await
            .map_err(|e| format!("failed to put non_polling_timestamps: {e}"))
    }
}

pub struct MockPastRequestTimestampsClient;

#[async_trait]
impl PastRequestTimestampsClient for MockPastRequestTimestampsClient {
    async fn get(&self, _user_id: u64) -> Result<Option<NonPollingTimestamps>, String> {
        Ok(None)
    }

    async fn put(&self, _user_id: u64, _npt: &NonPollingTimestamps) -> Result<(), String> {
        Ok(())
    }
}
