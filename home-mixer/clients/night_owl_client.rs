use std::time::Duration;

use night_owl_proto as night_owl;
use night_owl_proto::night_owl_search_service_client::NightOwlSearchServiceClient;
use tokio::time::timeout;
use tonic::async_trait;
use tonic::transport::Channel;
use xai_x_rpc::endpoint_source::PollingEndpointSource;
use xai_x_rpc::grpc_client::{ChannelBuilder, TlsMode};
use xai_x_rpc::wily_lookup_service::WilyNsLookup;

const WILY_PATH: &str = "/s/search-ai/night-owl";
const AUTHORITY: &str = "search-ai.night-owl.prod.{datacenter}.s2s.twttr.net";
pub const REQUEST_TIMEOUT_MS: u64 = 700;
const CONNECT_TIMEOUT_MS: u64 = 1000;
const DNS_POLL_INTERVAL_SECS: u64 = 5;
const NUM_ENDPOINTS: usize = 12;
const MAX_DECODING_MESSAGE_SIZE: usize = 20 * 1024 * 1024;

#[async_trait]
pub trait NightOwlClient: Send + Sync {
    async fn search(
        &self,
        request: night_owl::SearchRequest,
        timeout_ms: u64,
    ) -> Result<night_owl::SearchResponse, tonic::Status>;
}

pub struct ProdNightOwlClient {
    client: NightOwlSearchServiceClient<Channel>,
}

impl ProdNightOwlClient {
    pub async fn new(datacenter: &str) -> Result<Self, tonic::Status> {
        let wilyns = xai_wily::WilyNs::new(xai_wily::WilyConfig {
            zone: datacenter.to_string(),
            ..Default::default()
        })
        .map_err(|e| tonic::Status::internal(format!("WilyNS init failed: {e}")))?;
        let lookup = WilyNsLookup {
            wilyns,
            wily_path: WILY_PATH.to_string(),
            num_endpoints: Some(NUM_ENDPOINTS),
            shard_coordinate: None,
            filter_to_k8s_only: false,
        };
        let source =
            PollingEndpointSource::new(lookup, Duration::from_secs(DNS_POLL_INTERVAL_SECS));
        let tls = TlsMode::mtls_from_env()
            .map(|t| t.with_domain_override(AUTHORITY.replace("{datacenter}", datacenter)))
            .map_err(|e| tonic::Status::internal(format!("mTLS config failed: {e}")))?;
        let channel = ChannelBuilder::new("night-owl")
            .tls(tls)
            .connect_timeout(Duration::from_millis(CONNECT_TIMEOUT_MS))
            .request_timeout(Duration::from_millis(REQUEST_TIMEOUT_MS))
            .endpoint_source(source)
            .build()
            .await
            .map_err(|e| {
                tonic::Status::unavailable(format!("NightOwl channel build failed: {e}"))
            })?;
        Ok(Self {
            client: NightOwlSearchServiceClient::new(channel)
                .max_decoding_message_size(MAX_DECODING_MESSAGE_SIZE),
        })
    }
}

#[async_trait]
impl NightOwlClient for ProdNightOwlClient {
    async fn search(
        &self,
        request: night_owl::SearchRequest,
        timeout_ms: u64,
    ) -> Result<night_owl::SearchResponse, tonic::Status> {
        let timeout_ms = timeout_ms.min(REQUEST_TIMEOUT_MS);
        let mut client = self.client.clone();
        timeout(Duration::from_millis(timeout_ms), client.search(request))
            .await
            .map_err(|_| tonic::Status::deadline_exceeded("NightOwl search timed out"))?
            .map(|r| r.into_inner())
    }
}

pub struct MockNightOwlClient;

#[async_trait]
impl NightOwlClient for MockNightOwlClient {
    async fn search(
        &self,
        _request: night_owl::SearchRequest,
        _timeout_ms: u64,
    ) -> Result<night_owl::SearchResponse, tonic::Status> {
        Ok(night_owl::SearchResponse::default())
    }
}
