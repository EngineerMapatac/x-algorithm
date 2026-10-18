use std::collections::HashMap;
use std::sync::Arc;
use std::sync::LazyLock;
use std::time::Duration;
use std::time::Instant;
use strum::{EnumString, VariantArray};
use tonic::codec::CompressionEncoding;
use tonic::{async_trait, Status};
use tracing::info;
use xai_candidate_pipeline::component_library::utils::SuccessRateRouter;
use xai_recsys_proto::{
    user_action_aggregation_service_client::UserActionAggregationServiceClient,
    FetchAggregatedUserActionRequest, FetchAggregatedUserActionRequests, ResponseFormat,
    UaasModelType, UaasProductSurface, UserActionAggregationType, UserActionSequence,
    UserActionSequenceSourceDataType,
};
use xai_stats_receiver::{global_stats_receiver, HistogramBuckets};
use xai_x_rpc::balanced_channel::{LbPolicy, LoadBalancedChannel};
use xai_x_rpc::grpc_client::{ChannelBuilder, TlsMode};
use xai_x_rpc::xds_endpoint_source::XdsEndpointSource;
use xai_xds_client::{StartFrom, XdsClient};

use super::rotating_channel::RotatingChannel;

pub struct AggregatedSequenceResult {
    pub sequence: UserActionSequence,
    pub columnar_bytes: Option<bytes::Bytes>,
}

const TIMEOUT_MS: u64 = 200;
const MAX_RETRIES: u32 = 2;
const METRIC_NAME: &str = "UserActionAggregationClient.fetch";

const UAS_XDS_LISTENER: &str = "recsys-user-action-aggregation.prod.recsys-infra:grpc";
const UAS_XDS_APERTURE_SIZE: usize = 16;
const REFRESH_INTERVAL: Duration = Duration::from_secs(60);

static SR_ROUTER: LazyLock<SuccessRateRouter<UserActionsCluster>> =
    LazyLock::new(|| SuccessRateRouter::new(UserActionsCluster::Fou, UserActionsCluster::Amp));

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, strum::Display, EnumString, VariantArray)]
#[strum(serialize_all = "lowercase", ascii_case_insensitive)]
pub enum UserActionsCluster {
    Fou,
    Amp,
}

impl UserActionsCluster {
    pub fn endpoint(&self) -> String {
        let dc = self.to_string();
        format!("https://xai-recsys-user-action-aggregation-proxy.prod.{dc}.twitter.biz:443")
    }

    pub fn parse(s: &str) -> Self {
        s.parse().unwrap_or(Self::VARIANTS[0])
    }
}

#[async_trait]
pub trait UserActionAggregationClient {
    async fn fetch_aggregated_sequence(
        &self,
        cluster: UserActionsCluster,
        user_id: u64,
        window_time_ms: u32,
        max_sequence_length: u32,
        aggregation_type: UserActionAggregationType,
        source_data_type: UserActionSequenceSourceDataType,
        response_format: ResponseFormat,
        request_id: Option<i64>,
        use_xds: bool,
        model_type: UaasModelType,
    ) -> Result<AggregatedSequenceResult, Status>;
}

pub struct ProdUserActionAggregationClient {
    envoy_channels: HashMap<UserActionsCluster, Arc<RotatingChannel>>,
    xds_client: UserActionAggregationServiceClient<LoadBalancedChannel>,
}

impl ProdUserActionAggregationClient {
    pub async fn new() -> Result<Self, Status> {
        let (envoy_channels, xds_client) =
            tokio::try_join!(Self::build_envoy_channels(), Self::build_xds_client())?;

        Ok(Self {
            envoy_channels,
            xds_client,
        })
    }

    async fn build_envoy_channels(
    ) -> Result<HashMap<UserActionsCluster, Arc<RotatingChannel>>, Status> {
        info!("UAS client: building envoy proxy channels");
        let builds = UserActionsCluster::VARIANTS
            .iter()
            .map(|&cluster| async move {
                let ch = RotatingChannel::new(
                    cluster.endpoint(),
                    Duration::from_millis(TIMEOUT_MS),
                    REFRESH_INTERVAL,
                )
                .await?;
                Ok::<_, Status>((cluster, ch))
            });
        futures::future::join_all(builds)
            .await
            .into_iter()
            .collect()
    }

    async fn build_xds_client(
    ) -> Result<UserActionAggregationServiceClient<LoadBalancedChannel>, Status> {
        info!(
            xds_listener = UAS_XDS_LISTENER,
            aperture_size = UAS_XDS_APERTURE_SIZE,
            lb_policy = "penalized_peak_ewma",
            "UAS client: building XDS channel"
        );

        let xds_client = XdsClient::from_bootstrap()
            .map_err(|e| {
                Status::failed_precondition(format!("UAS XDS: bootstrap not available: {e}"))
            })?
            .start_from(StartFrom::Lds(UAS_XDS_LISTENER.to_string()))
            .build()
            .await
            .map_err(|e| {
                Status::unavailable(format!("UAS XDS: failed to build XDS client: {e}"))
            })?;

        let source = XdsEndpointSource::new(xds_client);

        let channel = ChannelBuilder::new("user-action-aggregation-xds")
            .tls(TlsMode::Plaintext)
            .request_timeout(Duration::from_millis(TIMEOUT_MS))
            .endpoint_source(source)
            .aperture(UAS_XDS_APERTURE_SIZE)
            .deterministic()
            .build_load_balanced(LbPolicy::penalized_peak_ewma())
            .await
            .map_err(|e| Status::unavailable(format!("UAS XDS: failed to build channel: {e}")))?;

        let client = UserActionAggregationServiceClient::new(channel)
            .send_compressed(CompressionEncoding::Zstd)
            .accept_compressed(CompressionEncoding::Zstd);

        Self::warm_connection(&client).await;
        info!("UAS XDS: channel ready");
        Ok(client)
    }

    async fn warm_connection(client: &UserActionAggregationServiceClient<LoadBalancedChannel>) {
        let request = FetchAggregatedUserActionRequests {
            requests: vec![FetchAggregatedUserActionRequest {
                user_id: 0,
                window_time_ms: 1,
                max_sequence_length: 1,
                aggregation_type: UserActionAggregationType::Dense as i32,
                source_data_type: UserActionSequenceSourceDataType::Arrow as i32,
                response_format: ResponseFormat::Proto as i32,
                ..Default::default()
            }],
            client_id: "xai-home-mixer-warmup".to_string(),
            model_type: UaasModelType::Unspecified as i32,
            product_surface: UaasProductSurface::Home as i32,
            ..Default::default()
        };

        let start = Instant::now();
        match client.clone().fetch_aggregated_user_actions(request).await {
            Ok(_) => {
                info!(
                    latency_ms = start.elapsed().as_millis() as u64,
                    "UAS XDS: warmup request succeeded"
                );
            }
            Err(e) => {
                tracing::warn!(
                    latency_ms = start.elapsed().as_millis() as u64,
                    error = %e,
                    "UAS XDS: warmup request failed (non-fatal)"
                );
            }
        }
    }

    fn record_stats(cluster: UserActionsCluster, success: bool, latency_ms: f64) {
        SR_ROUTER.record(cluster, success);

        if let Some(receiver) = global_stats_receiver() {
            let cluster_label = format!("{:?}", cluster);
            let result_label = if success { "success" } else { "failure" };
            let scope = [
                ("user_actions_cluster", cluster_label.as_str()),
                ("result", result_label),
            ];
            receiver.incr(METRIC_NAME, &scope, 1);
            receiver.observe(
                METRIC_NAME,
                &[("user_actions_cluster", cluster_label.as_str())],
                latency_ms,
                HistogramBuckets::Bucket500To1000,
            );
        }
    }
}

#[async_trait]
impl UserActionAggregationClient for ProdUserActionAggregationClient {
    async fn fetch_aggregated_sequence(
        &self,
        cluster: UserActionsCluster,
        user_id: u64,
        window_time_ms: u32,
        max_sequence_length: u32,
        aggregation_type: UserActionAggregationType,
        source_data_type: UserActionSequenceSourceDataType,
        response_format: ResponseFormat,
        request_id: Option<i64>,
        use_xds: bool,
        model_type: UaasModelType,
    ) -> Result<AggregatedSequenceResult, Status> {
        let (envoy_client, cluster) = if use_xds {
            (None, cluster)
        } else {
            let routed = SR_ROUTER.route(cluster);
            let ch = self
                .envoy_channels
                .get(&routed)
                .ok_or_else(|| Status::not_found(format!("No channel for cluster: {routed:?}")))?;
            let channel = ch.next();
            let c = UserActionAggregationServiceClient::new((*channel).clone())
                .send_compressed(CompressionEncoding::Zstd)
                .accept_compressed(CompressionEncoding::Zstd);
            (Some(c), routed)
        };

        let request = FetchAggregatedUserActionRequests {
            requests: vec![FetchAggregatedUserActionRequest {
                user_id: user_id as i64,
                window_time_ms,
                aggregation_type: aggregation_type as i32,
                max_sequence_length,
                source_data_type: source_data_type as i32,
                response_format: response_format as i32,
                request_id,
                seed_post_id: None,
            }],
            client_id: "xai-home-mixer".to_string(),
            model_type: model_type as i32,
            product_surface: UaasProductSurface::Home as i32,
            ..Default::default()
        };

        let mut last_err = None;
        for attempt in 0..=MAX_RETRIES {
            let start = Instant::now();
            let result = if use_xds {
                self.xds_client
                    .clone()
                    .fetch_aggregated_user_actions(request.clone())
                    .await
            } else {
                envoy_client
                    .as_ref()
                    .expect("envoy client set when !use_xds")
                    .clone()
                    .fetch_aggregated_user_actions(request.clone())
                    .await
            }
            .map(|response| response.into_inner());
            let latency_ms = start.elapsed().as_millis() as f64;

            Self::record_stats(cluster, result.is_ok(), latency_ms);

            match result {
                Ok(mut response) => {
                    let columnar_bytes = match response_format {
                        ResponseFormat::Arrow => {
                            response.columnar_sequences.pop().filter(|b| !b.is_empty())
                        }
                        _ => None,
                    };
                    let sequence = response.sequences.into_iter().next().unwrap_or_default();
                    return Ok(AggregatedSequenceResult {
                        sequence,
                        columnar_bytes,
                    });
                }
                Err(e) => {
                    tracing::warn!(
                        attempt = attempt + 1,
                        max_retries = MAX_RETRIES,
                        error = %e,
                        "UserActionAggregationClient fetch failed, {}",
                        if attempt < MAX_RETRIES { "retrying" } else { "no more retries" }
                    );
                    last_err = Some(e);
                }
            }
        }

        Err(last_err.unwrap_or_else(|| Status::internal("All retries exhausted")))
    }
}

pub struct MockUserActionAggregationClient;

#[async_trait]
impl UserActionAggregationClient for MockUserActionAggregationClient {
    async fn fetch_aggregated_sequence(
        &self,
        _cluster: UserActionsCluster,
        _user_id: u64,
        _window_time_ms: u32,
        _max_sequence_length: u32,
        _aggregation_type: UserActionAggregationType,
        _source_data_type: UserActionSequenceSourceDataType,
        _response_format: ResponseFormat,
        _request_id: Option<i64>,
        _use_xds: bool,
        _model_type: UaasModelType,
    ) -> Result<AggregatedSequenceResult, Status> {
        Ok(AggregatedSequenceResult {
            sequence: UserActionSequence::default(),
            columnar_bytes: None,
        })
    }
}
