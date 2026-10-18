use std::sync::Arc;
use std::time::Duration;

use tonic::async_trait;
use xai_cache::discovery::WilyDiscovery;
use xai_cache::{CacheClient, ClientBuilder, ClientConfig, Key, Protocol, TlsConfig};
use xai_x_thrift::impression_store::ImpressionList;

use crate::clients::s2s::{S2S_CHAIN_PATH, S2S_CRT_PATH, S2S_KEY_PATH};

const WILY_PATH: &str = "/s/cache/timelines_impressionstore:twemcaches";
const CLIENT_NAME: &str = "home-mixer";
const REQUEST_TIMEOUT: Duration = Duration::from_millis(300);

#[async_trait]
pub trait ImpressedPostsClient: Send + Sync {
    async fn get(&self, user_id: u64) -> Result<Vec<u64>, String>;
}

pub struct ProdImpressedPostsClient {
    client: Arc<CacheClient>,
}

impl ProdImpressedPostsClient {
    pub async fn new(datacenter: &str) -> anyhow::Result<Self> {
        let discovery = Arc::new(
            WilyDiscovery::new(WILY_PATH, CLIENT_NAME, datacenter)
                .await
                .map_err(|e| anyhow::anyhow!("ImpressedPosts cache discovery failed: {e}"))?,
        );

        let config = ClientConfig::builder()
            .request_timeout(REQUEST_TIMEOUT)
            .build();

        let client = ClientBuilder::new(Protocol::Memcached, discovery, config)
            .with_tls(TlsConfig {
                ca_cert_path: S2S_CHAIN_PATH.clone(),
                client_cert_path: S2S_CRT_PATH.clone(),
                client_key_path: S2S_KEY_PATH.clone(),
            })
            .build()
            .await
            .map_err(|e| anyhow::anyhow!("ImpressedPosts cache client build failed: {e}"))?;

        tracing::info!("ImpressedPosts cache client connected to {WILY_PATH}");
        Ok(Self {
            client: Arc::new(client),
        })
    }
}

fn cache_key(user_id: u64) -> Result<Key, String> {
    let key_bytes = (user_id as i64).to_be_bytes().to_vec();
    Key::new(key_bytes).map_err(|e| format!("invalid cache key: {e}"))
}

#[async_trait]
impl ImpressedPostsClient for ProdImpressedPostsClient {
    async fn get(&self, user_id: u64) -> Result<Vec<u64>, String> {
        let key = cache_key(user_id)?;

        let bytes = self
            .client
            .get(&key)
            .await
            .map_err(|e| format!("ImpressedPosts cache get failed: {e}"))?;

        let Some(bytes) = bytes else {
            return Ok(Vec::new());
        };

        let impression_list: ImpressionList = xai_x_thrift::deserialize_compact(&bytes)
            .map_err(|e| format!("failed to deserialize ImpressionList: {e}"))?;

        let post_ids = impression_list
            .impressions
            .unwrap_or_default()
            .iter()
            .map(|impression| impression.tweet_id as u64)
            .collect();

        Ok(post_ids)
    }
}

#[derive(Default)]
pub struct MockImpressedPostsClient {
    pub post_ids: Vec<u64>,
}

#[async_trait]
impl ImpressedPostsClient for MockImpressedPostsClient {
    async fn get(&self, _user_id: u64) -> Result<Vec<u64>, String> {
        Ok(self.post_ids.clone())
    }
}
