use std::time::{Duration, Instant};
use tokio::time::timeout;
use tonic::transport::Channel;
use tonic::{async_trait, Status};
use tracing::info;
use xai_prompts_thrift::injection::Injection;
use xai_prompts_thrift::injection_service::{GetInjectionsRequest, GetInjectionsResponse};
use xai_proto::generic_thrift::generic_thrift_service_client::GenericThriftServiceClient;
use xai_proto::generic_thrift::{GenericThriftRequest, GenericThriftResponse};
use xai_stats_receiver::{global_stats_receiver, HistogramBuckets};
use xai_wily::WilyNs;
use xai_x_rpc::endpoint_source::PollingEndpointSource;
use xai_x_rpc::grpc_client::{ChannelBuilder, TlsMode};
use xai_x_rpc::lookup_service::LookupService;
use xai_x_rpc::retry::{RetryConfig, RetryingChannel};
use xai_x_rpc::wily_lookup_service::WilyNsLookup;

const WILY_PATH: &str = "/s/onboarding-task-service/onboarding-task-service:grpc";
const TIMEOUT_MS: u64 = 500;
const NUM_ENDPOINTS: usize = 10;
const DNS_POLL_INTERVAL_MS: u64 = 30_000;
const METRIC_NAME: &str = "PromptsClient.get_injections";

#[async_trait]
pub trait PromptsClient: Send + Sync {
    async fn get_injections(&self, request: GetInjectionsRequest)
        -> Result<Vec<Injection>, Status>;
}

pub struct ProdPromptsClient {
    client: GenericThriftServiceClient<RetryingChannel<Channel>>,
}

impl ProdPromptsClient {
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
            Status::unavailable(format!("PromptsClient: Wily DNS resolution failed: {e}"))
        })?;
        if endpoints.is_empty() {
            return Err(Status::unavailable(
                "PromptsClient: Wily DNS returned 0 endpoints",
            ));
        }
        info!(
            endpoint_count = endpoints.len(),
            "PromptsClient: Wily DNS resolution succeeded"
        );

        let source =
            PollingEndpointSource::new(lookup, Duration::from_millis(DNS_POLL_INTERVAL_MS));

        let sni =
            format!("onboarding-task-service.onboarding-task-service.prod.{dc}.s2s.twttr.net");
        let tls = TlsMode::mtls_from_env()
            .map_err(|e| Status::internal(format!("failed to load mTLS certs: {e}")))?
            .with_domain_override(&sni);

        let channel = ChannelBuilder::new("onboarding-task-service")
            .connect_timeout(Duration::from_secs(2))
            .request_timeout(Duration::from_millis(TIMEOUT_MS))
            .tls(tls)
            .endpoint_source(source)
            .tcp_health_probe()
            .build()
            .await
            .map_err(|e| {
                Status::unavailable(format!("failed to connect to onboarding-task-service: {e}"))
            })?;

        let channel = RetryingChannel::new(channel, RetryConfig::for_idempotent());
        let client = GenericThriftServiceClient::new(channel);

        Ok(Self { client })
    }
}

#[async_trait]
impl PromptsClient for ProdPromptsClient {
    async fn get_injections(
        &self,
        request: GetInjectionsRequest,
    ) -> Result<Vec<Injection>, Status> {
        let payload = xai_prompts_thrift::serialize_binary(&request)
            .map_err(|e| Status::internal(format!("thrift serialization failed: {e}")))?;

        let grpc_request = GenericThriftRequest {
            payload: payload.into(),
        };

        let start = Instant::now();
        let result = timeout(
            Duration::from_millis(TIMEOUT_MS),
            self.client.clone().get_request(grpc_request),
        )
        .await
        .map_err(|_| Status::deadline_exceeded("PromptsClient: deadline exceeded"))?
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

        let grpc_response: GenericThriftResponse = result?;
        let parsed: GetInjectionsResponse =
            xai_prompts_thrift::deserialize_binary(&grpc_response.payload)
                .map_err(|e| Status::internal(format!("thrift deserialization failed: {e}")))?;

        Ok(parsed.injections)
    }
}

pub struct MockPromptsClient;

#[async_trait]
impl PromptsClient for MockPromptsClient {
    async fn get_injections(
        &self,
        _request: GetInjectionsRequest,
    ) -> Result<Vec<Injection>, Status> {
        Ok(vec![])
    }
}
