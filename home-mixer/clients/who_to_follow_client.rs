use std::time::{Duration, Instant};
use tokio::time::timeout;
use tonic::transport::Channel;
use tonic::{async_trait, Status};
use tracing::info;
use xai_account_recommendations_mixer_proto::account_recommendations_mixer_client::AccountRecommendationsMixerClient;
use xai_account_recommendations_mixer_proto::{
    AccountRecommendationsMixerRequest, WhoToFollowResponse,
};
use xai_stats_receiver::{global_stats_receiver, HistogramBuckets};
use xai_wily::WilyNs;
use xai_x_rpc::endpoint_source::PollingEndpointSource;
use xai_x_rpc::grpc_client::{ChannelBuilder, TlsMode};
use xai_x_rpc::lookup_service::LookupService;
use xai_x_rpc::retry::{RetryConfig, RetryingChannel};
use xai_x_rpc::wily_lookup_service::WilyNsLookup;

const WILY_PATH: &str = "/s/account-recs-mixer/account-recs-mixer:grpc";
const TIMEOUT_MS: u64 = 600;
const NUM_ENDPOINTS: usize = 15;
const DNS_POLL_INTERVAL_MS: u64 = 30_000;
const METRIC_NAME: &str = "WhoToFollowClient.get_wtf_recommendations";

#[async_trait]
pub trait WhoToFollowClient: Send + Sync {
    async fn get_wtf_recommendations(
        &self,
        request: AccountRecommendationsMixerRequest,
    ) -> Result<WhoToFollowResponse, Status>;
}

pub struct ProdWhoToFollowClient {
    client: AccountRecommendationsMixerClient<RetryingChannel<Channel>>,
}

impl ProdWhoToFollowClient {
    pub async fn new(dc: &str) -> Result<Self, Status> {
        let wilyns = WilyNs::new(xai_wily::WilyConfig::local_zone(dc))
            .map_err(|e| Status::internal(format!("failed to create WilyNs: {e}")))?;
        let lookup = WilyNsLookup {
            wilyns,
            wily_path: WILY_PATH.to_string(),
            num_endpoints: Some(NUM_ENDPOINTS),
            shard_coordinate: None,
            filter_to_k8s_only: false,
        };

        let endpoints = lookup.resolve_service_endpoints().await.map_err(|e| {
            Status::unavailable(format!(
                "WhoToFollowClient: Wily DNS resolution failed: {e}"
            ))
        })?;
        if endpoints.is_empty() {
            return Err(Status::unavailable(
                "WhoToFollowClient: Wily DNS returned 0 endpoints",
            ));
        }
        info!(
            endpoint_count = endpoints.len(),
            "WhoToFollowClient: Wily DNS resolution succeeded"
        );

        let source =
            PollingEndpointSource::new(lookup, Duration::from_millis(DNS_POLL_INTERVAL_MS));

        let sni = format!("account-recs-mixer.account-recs-mixer.prod.{dc}.s2s.twttr.net");
        let tls = TlsMode::mtls_from_env()
            .map_err(|e| Status::internal(format!("failed to load mTLS certs: {e}")))?
            .with_domain_override(&sni);

        let channel = ChannelBuilder::new("account-recommendations-mixer")
            .connect_timeout(Duration::from_secs(5))
            .request_timeout(Duration::from_millis(TIMEOUT_MS))
            .tls(tls)
            .endpoint_source(source)
            .tcp_health_probe()
            .build()
            .await
            .map_err(|e| {
                Status::unavailable(format!(
                    "failed to connect to account-recommendations-mixer: {e}"
                ))
            })?;

        let channel = RetryingChannel::new(channel, RetryConfig::for_idempotent());
        let client = AccountRecommendationsMixerClient::new(channel);

        Ok(Self { client })
    }
}

#[async_trait]
impl WhoToFollowClient for ProdWhoToFollowClient {
    async fn get_wtf_recommendations(
        &self,
        request: AccountRecommendationsMixerRequest,
    ) -> Result<WhoToFollowResponse, Status> {
        let start = Instant::now();
        let result = timeout(
            Duration::from_millis(TIMEOUT_MS),
            self.client.clone().get_wtf_recommendations(request),
        )
        .await
        .map_err(|_| Status::deadline_exceeded("WhoToFollowClient: deadline exceeded"))?
        .map(|r| r.into_inner());
        let latency_ms = start.elapsed().as_millis() as f64;

        if let Some(receiver) = global_stats_receiver() {
            let result_label = if result.is_ok() { "success" } else { "failure" };
            receiver.incr(METRIC_NAME, &[("result", result_label)], 1);
            receiver.observe(
                METRIC_NAME,
                &[],
                latency_ms,
                HistogramBuckets::Bucket500To1000,
            );
        }

        result
    }
}

pub struct MockWhoToFollowClient;

#[async_trait]
impl WhoToFollowClient for MockWhoToFollowClient {
    async fn get_wtf_recommendations(
        &self,
        _request: AccountRecommendationsMixerRequest,
    ) -> Result<WhoToFollowResponse, Status> {
        Ok(WhoToFollowResponse {
            header: None,
            footer: None,
            user_recommendations: vec![],
            display_options: None,
        })
    }
}
