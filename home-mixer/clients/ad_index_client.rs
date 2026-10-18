use std::sync::LazyLock;
use std::time::{Duration, Instant};
use tokio::time::timeout;
use tonic::codec::CompressionEncoding;
use tonic::transport::Channel;
use tonic::{async_trait, Status};
use tracing::{info, warn};
use xai_candidate_pipeline::component_library::utils::SuccessRateRouter;
use xai_recsys_proto::ad_index_service_client::AdIndexServiceClient;
use xai_recsys_proto::{AdIndexRequest, AdIndexResponse};
use xai_stats_receiver::{global_stats_receiver, HistogramBuckets};
use xai_wily::WilyNs;
use xai_x_rpc::endpoint_source::PollingEndpointSource;
use xai_x_rpc::grpc_client::{ChannelBuilder, TlsMode};
use xai_x_rpc::lookup_service::LookupService;
use xai_x_rpc::wily_lookup_service::WilyNsLookup;

const WILY_PATH: &str = "/s/ads/xai-ad-index:grpc";
const TIMEOUT_MS: u64 = 1200;
const NUM_ENDPOINTS: usize = 5;
const DNS_POLL_INTERVAL_MS: u64 = 30_000;
const METRIC_NAME: &str = "AdIndexClient.get_eligible_ads";

fn app_env() -> String {
    std::env::var("APP_ENV").unwrap_or_else(|_| "prod".to_string())
}

fn ad_index_wily_path(dc: &str, env: &str) -> String {
    if env == "prod" {
        WILY_PATH.to_string()
    } else {
        format!("/srv%23/{env}/{dc}/ads/xai-ad-index:grpc")
    }
}

fn ad_index_sni(dc: &str, env: &str) -> String {
    format!("ads.ad-index.{env}.{dc}.s2s.twttr.net")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AdIndexCluster {
    Local,
    Remote,
}

impl AdIndexCluster {
    fn as_label(&self) -> &'static str {
        match self {
            AdIndexCluster::Local => "local",
            AdIndexCluster::Remote => "remote",
        }
    }
}

static SR_ROUTER: LazyLock<SuccessRateRouter<AdIndexCluster>> =
    LazyLock::new(|| SuccessRateRouter::new(AdIndexCluster::Local, AdIndexCluster::Remote));

#[async_trait]
pub trait AdIndexClient: Send + Sync {
    async fn get_eligible_ads(&self, request: AdIndexRequest) -> Result<AdIndexResponse, Status>;
}

pub struct ProdAdIndexClient {
    local_client: AdIndexServiceClient<Channel>,
    remote_client: Option<AdIndexServiceClient<Channel>>,
}

impl ProdAdIndexClient {
    pub async fn new(dc: &str) -> Result<Self, Status> {
        let (local_result, remote_client) = tokio::join!(Self::build_client(dc), async {
            match peer_dc(dc) {
                Some(remote_dc) => match Self::build_client(remote_dc).await {
                    Ok(client) => {
                        info!(remote_dc, "AdIndexClient: remote DC client built");
                        Some(client)
                    }
                    Err(e) => {
                        warn!(
                            remote_dc,
                            error = %e,
                            "AdIndexClient: failed to build remote DC client; SR router disabled"
                        );
                        None
                    }
                },
                None => {
                    warn!(
                        local_dc = dc,
                        "AdIndexClient: unknown local DC, no peer to fail over to; SR router disabled"
                    );
                    None
                }
            }
        });

        Ok(Self {
            local_client: local_result?,
            remote_client,
        })
    }

    async fn build_client(dc: &str) -> Result<AdIndexServiceClient<Channel>, Status> {
        let env = app_env();
        let wily_path = ad_index_wily_path(dc, &env);
        info!(dc, %env, %wily_path, "AdIndexClient: building client");
        let wilyns = WilyNs::new(xai_wily::WilyConfig::local_zone(dc))
            .map_err(|e| Status::internal(format!("failed to create WilyNs for {dc}: {e}")))?;
        let lookup = WilyNsLookup {
            wilyns,
            wily_path,
            num_endpoints: Some(NUM_ENDPOINTS),
            shard_coordinate: None,
            filter_to_k8s_only: false,
        };

        match lookup.resolve_service_endpoints().await {
            Ok(endpoints) => {
                info!(
                    dc,
                    endpoint_count = endpoints.len(),
                    "AdIndexClient: Wily DNS resolution succeeded"
                );
                if endpoints.is_empty() {
                    warn!(
                        dc,
                        "AdIndexClient: Wily DNS returned 0 endpoints! Requests may hang."
                    );
                }
            }
            Err(e) => {
                warn!(dc, error = %e, "AdIndexClient: Wily DNS resolution failed");
            }
        }

        let source =
            PollingEndpointSource::new(lookup, Duration::from_millis(DNS_POLL_INTERVAL_MS));

        let sni = ad_index_sni(dc, &env);
        let tls = TlsMode::mtls_from_env()
            .map_err(|e| Status::internal(format!("failed to load mTLS certs: {e}")))?
            .with_domain_override(&sni);

        let channel = ChannelBuilder::new("ad-index")
            .request_timeout(Duration::from_millis(TIMEOUT_MS))
            .tls(tls)
            .endpoint_source(source)
            .build()
            .await
            .map_err(|e| {
                Status::unavailable(format!("failed to connect to ad-index ({dc}): {e}"))
            })?;

        Ok(AdIndexServiceClient::new(channel)
            .send_compressed(CompressionEncoding::Zstd)
            .accept_compressed(CompressionEncoding::Zstd))
    }

    fn pick_cluster_and_client(&self) -> (AdIndexCluster, AdIndexServiceClient<Channel>) {
        match &self.remote_client {
            Some(remote) => {
                let cluster = SR_ROUTER.route(AdIndexCluster::Local);
                let client = match cluster {
                    AdIndexCluster::Local => self.local_client.clone(),
                    AdIndexCluster::Remote => remote.clone(),
                };
                (cluster, client)
            }
            None => (AdIndexCluster::Local, self.local_client.clone()),
        }
    }

    fn record_stats(&self, cluster: AdIndexCluster, success: bool, latency_ms: f64) {
        if self.remote_client.is_some() {
            SR_ROUTER.record(cluster, success);
        }

        if let Some(receiver) = global_stats_receiver() {
            let cluster_label = cluster.as_label();
            let result_label = if success { "success" } else { "failure" };
            receiver.incr(
                METRIC_NAME,
                &[
                    ("ad_index_cluster", cluster_label),
                    ("result", result_label),
                ],
                1,
            );
            receiver.observe(
                METRIC_NAME,
                &[("ad_index_cluster", cluster_label)],
                latency_ms,
                HistogramBuckets::Bucket500To1000,
            );
        }
    }
}

fn peer_dc(dc: &str) -> Option<&'static str> {
    match dc {
        "atla" => Some("pdxa"),
        "pdxa" => Some("atla"),
        _ => None,
    }
}

#[async_trait]
impl AdIndexClient for ProdAdIndexClient {
    async fn get_eligible_ads(&self, request: AdIndexRequest) -> Result<AdIndexResponse, Status> {
        let (cluster, client) = self.pick_cluster_and_client();

        let mut grpc_request = tonic::Request::new(request);
        grpc_request.set_timeout(Duration::from_millis(TIMEOUT_MS));

        let start = Instant::now();
        let result = timeout(
            Duration::from_millis(TIMEOUT_MS),
            client.clone().get_eligible_ads(grpc_request),
        )
        .await
        .map_err(|_| Status::deadline_exceeded("AdIndexClient: deadline exceeded"))?
        .map(|r| r.into_inner());
        let latency_ms = start.elapsed().as_millis() as f64;

        self.record_stats(cluster, result.is_ok(), latency_ms);

        result
    }
}

pub struct MockAdIndexClient;

#[async_trait]
impl AdIndexClient for MockAdIndexClient {
    async fn get_eligible_ads(&self, _request: AdIndexRequest) -> Result<AdIndexResponse, Status> {
        Ok(AdIndexResponse {
            ad_info: vec![],
            ..Default::default()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn peer_dc_atla_pairs_with_pdxa() {
        assert_eq!(peer_dc("atla"), Some("pdxa"));
    }

    #[test]
    fn peer_dc_pdxa_pairs_with_atla() {
        assert_eq!(peer_dc("pdxa"), Some("atla"));
    }

    #[test]
    fn peer_dc_unknown_returns_none() {
        assert_eq!(peer_dc("smfa"), None);
        assert_eq!(peer_dc(""), None);
        assert_eq!(peer_dc("staging"), None);
    }

    #[test]
    fn cluster_labels_are_stable() {
        assert_eq!(AdIndexCluster::Local.as_label(), "local");
        assert_eq!(AdIndexCluster::Remote.as_label(), "remote");
    }

    #[test]
    fn prod_wily_path_is_plain_serverset() {
        assert_eq!(
            ad_index_wily_path("atla", "prod"),
            "/s/ads/xai-ad-index:grpc"
        );
        assert_eq!(
            ad_index_wily_path("pdxa", "prod"),
            "/s/ads/xai-ad-index:grpc"
        );
    }

    #[test]
    fn staging_wily_path_targets_staging_serverset_with_encoded_hash() {
        assert_eq!(
            ad_index_wily_path("atla", "staging"),
            "/srv%23/staging/atla/ads/xai-ad-index:grpc"
        );
        assert_eq!(
            ad_index_wily_path("pdxa", "staging"),
            "/srv%23/staging/pdxa/ads/xai-ad-index:grpc"
        );
    }

    #[test]
    fn sni_matches_env() {
        assert_eq!(
            ad_index_sni("atla", "prod"),
            "ads.ad-index.prod.atla.s2s.twttr.net"
        );
        assert_eq!(
            ad_index_sni("pdxa", "staging"),
            "ads.ad-index.staging.pdxa.s2s.twttr.net"
        );
    }
}
