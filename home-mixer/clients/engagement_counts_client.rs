use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use prost::Message;
use tonic::async_trait;
use xai_proto::engagement_counter::EngagementCounts;
use xai_redis_client::{NodeOpStat, XdsRedisClient, XdsRedisConfig};

const CACHE_NAME: &str = "view-count-cache";
const CACHE_ENVIRONMENT: &str = "prod";
const CACHE_NAMESPACE: &str = "cache";
const RESPONSE_TIMEOUT: Duration = Duration::from_millis(200);
const MGET_NODE_TIMEOUT: Duration = Duration::from_millis(100);

#[async_trait]
pub trait EngagementCountsClient: Send + Sync {
    async fn get_engagement_counts(
        &self,
        tweet_ids: &[u64],
    ) -> Result<HashMap<u64, EngagementCounts>, String>;
}

pub struct ProdEngagementCountsClient {
    client: Arc<XdsRedisClient>,
}

impl ProdEngagementCountsClient {
    pub async fn new(datacenter: &str) -> anyhow::Result<Self> {
        let eds_resource_name = eds_resource_name(datacenter);
        let client = XdsRedisClient::new_with_response_timeout(
            XdsRedisConfig {
                eds_resource_name: eds_resource_name.clone(),
            },
            RESPONSE_TIMEOUT,
        )
        .await
        .map_err(|e| anyhow::anyhow!("ProdEngagementCountsClient build failed: {e}"))?;

        anyhow::ensure!(
            client.is_ready(),
            "ProdEngagementCountsClient EDS has no ready endpoints after warmup: {eds_resource_name}",
        );

        Ok(Self {
            client: Arc::new(client),
        })
    }

    async fn mget_engagement_counts(
        &self,
        tweet_ids: &[u64],
    ) -> Result<HashMap<u64, EngagementCounts>, String> {
        if tweet_ids.is_empty() {
            return Ok(HashMap::new());
        }

        let stage_start = Instant::now();
        let keys: Vec<Vec<u8>> = tweet_ids.iter().map(|&id| cache_key(id)).collect();
        let (responses, stats) = self.client.mget_raw(&keys, MGET_NODE_TIMEOUT).await;

        let mut counts = HashMap::with_capacity(tweet_ids.len());
        for (tweet_id, response) in tweet_ids.iter().zip(responses.iter()) {
            if let Some(bytes) = response
                && let Some(parsed) = parse_engagement_counts(bytes)
            {
                counts.insert(*tweet_id, parsed);
            }
        }

        let missing = (tweet_ids.len() - counts.len()) as u64;
        record_metrics(tweet_ids.len() as u64, missing, stage_start.elapsed());

        if is_total_failure(&stats) {
            return Err(format!(
                "EngagementCounter mget failed: {} shard op(s), none succeeded",
                stats.len()
            ));
        }

        Ok(counts)
    }
}

fn eds_resource_name(datacenter: &str) -> String {
    format!(
        "xdstp://discovery-{datacenter}/envoy.config.endpoint.v3.ClusterLoadAssignment/redis-global.{CACHE_NAME}.{CACHE_ENVIRONMENT}.{CACHE_NAMESPACE}:redis"
    )
}

fn cache_key(tweet_id: u64) -> Vec<u8> {
    tweet_id.to_string().into_bytes()
}

fn parse_engagement_counts(bytes: &[u8]) -> Option<EngagementCounts> {
    EngagementCounts::decode(bytes).ok()
}

fn is_total_failure(stats: &[NodeOpStat]) -> bool {
    stats.iter().all(|stat| !stat.ok)
}

fn record_metrics(queried: u64, missing: u64, elapsed: Duration) {
    let Some(receiver) = xai_stats_receiver::global_stats_receiver() else {
        return;
    };
    receiver.incr(
        "tweet_impressions.engagement_counter_query_count",
        &[],
        queried,
    );
    receiver.incr(
        "tweet_impressions.engagement_counter_missing_count",
        &[],
        missing,
    );
    receiver.observe_expo(
        "tweet_impressions.engagement_counter_latency_seconds",
        &[],
        elapsed.as_secs_f64(),
    );
}

#[async_trait]
impl EngagementCountsClient for ProdEngagementCountsClient {
    async fn get_engagement_counts(
        &self,
        tweet_ids: &[u64],
    ) -> Result<HashMap<u64, EngagementCounts>, String> {
        self.mget_engagement_counts(tweet_ids).await
    }
}

#[derive(Default)]
pub struct MockEngagementCountsClient {
    pub counts: HashMap<u64, EngagementCounts>,
}

#[async_trait]
impl EngagementCountsClient for MockEngagementCountsClient {
    async fn get_engagement_counts(
        &self,
        tweet_ids: &[u64],
    ) -> Result<HashMap<u64, EngagementCounts>, String> {
        Ok(tweet_ids
            .iter()
            .filter_map(|&id| self.counts.get(&id).map(|counts| (id, *counts)))
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_view_count_from_engagement_counts() {
        let counts = EngagementCounts {
            view_count: 135_680,
            ..Default::default()
        };
        let parsed = parse_engagement_counts(&counts.encode_to_vec()).unwrap();
        assert_eq!(parsed.view_count, 135_680);
    }

    #[test]
    fn cache_key_matches_writer_encoding() {
        assert_eq!(cache_key(1_959_000_000_000_000_001), b"1959000000000000001");
    }

    #[test]
    fn total_failure_only_when_no_shard_succeeds() {
        let ok = NodeOpStat {
            latency: Duration::ZERO,
            ok: true,
        };
        let err = NodeOpStat {
            latency: Duration::ZERO,
            ok: false,
        };
        assert!(is_total_failure(&[]));
        assert!(is_total_failure(&[err]));
        assert!(!is_total_failure(&[err, ok]));
    }
}
