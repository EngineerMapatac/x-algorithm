use std::collections::HashMap;
use std::time::Duration;

use anyhow::{Context, Result};
use log::warn;
use tonic::transport::{Channel, Endpoint};
use tonic::Request;

use super::metrics::{
    SidMetrics, LATENCY_ERROR, LATENCY_SUCCESS, STATUS_ERROR, STATUS_HIT, STATUS_MISS,
    STATUS_TOO_FRESH,
};
use super::post_age_seconds;
use super::proto::sid_lookup_service_client::SidLookupServiceClient;
use super::proto::{LookupSidsRequest, PostSids};

#[derive(Clone)]
pub struct SidClient {
    inner: SidLookupServiceClient<Channel>,
    min_post_age_seconds: f64,
    request_timeout: Duration,
    metrics: SidMetrics,
}

impl SidClient {
    pub fn new(
        endpoint: &str,
        min_post_age_seconds: f64,
        request_timeout: Duration,
        metrics: SidMetrics,
    ) -> Result<Self> {
        let url = if let Some(rest) = endpoint.strip_prefix("https://") {
            anyhow::bail!(
                "SidClient does not support TLS endpoints; use plaintext http://, got https://{rest}"
            );
        } else if endpoint.starts_with("http://") {
            endpoint.to_string()
        } else {
            format!("http://{endpoint}")
        };

        let channel = Endpoint::from_shared(url.clone())
            .with_context(|| format!("invalid SID endpoint: {url}"))?
            .timeout(request_timeout)
            .connect_timeout(Duration::from_secs(10))
            .tcp_keepalive(Some(Duration::from_secs(30)))
            .http2_keep_alive_interval(Duration::from_secs(30))
            .keep_alive_timeout(Duration::from_secs(10))
            .keep_alive_while_idle(true)
            .connect_lazy();

        let inner = SidLookupServiceClient::new(channel)
            .max_decoding_message_size(64 * 1024 * 1024)
            .max_encoding_message_size(64 * 1024 * 1024);

        Ok(Self {
            inner,
            min_post_age_seconds,
            request_timeout,
            metrics,
        })
    }

    pub async fn lookup_sids(
        &self,
        post_ids: &[i64],
        now_ms: Option<i64>,
    ) -> HashMap<i64, Vec<i32>> {
        if post_ids.is_empty() {
            return HashMap::new();
        }

        let now_ms = now_ms.unwrap_or_else(|| chrono::Utc::now().timestamp_millis());

        let mut eligible: Vec<i64> = Vec::with_capacity(post_ids.len());
        let mut too_fresh: u64 = 0;
        for &pid in post_ids {
            if pid <= 0 {
                continue;
            }
            let age_s = post_age_seconds(pid, now_ms);
            if age_s < self.min_post_age_seconds {
                too_fresh += 1;
                continue;
            }
            eligible.push(pid);
        }
        if too_fresh > 0 {
            self.metrics
                .lookup_count
                .with_label_values(&[STATUS_TOO_FRESH])
                .inc_by(too_fresh as f64);
        }
        if eligible.is_empty() {
            return HashMap::new();
        }

        let mut request = Request::new(LookupSidsRequest {
            post_ids: eligible.clone(),
        });
        request.set_timeout(self.request_timeout);

        let start = std::time::Instant::now();
        let response = match self.inner.clone().lookup_sids(request).await {
            Ok(resp) => resp.into_inner(),
            Err(status) => {
                let elapsed = start.elapsed().as_secs_f64();
                self.metrics
                    .lookup_latency_seconds
                    .with_label_values(&[LATENCY_ERROR])
                    .observe(elapsed);
                self.metrics
                    .lookup_count
                    .with_label_values(&[STATUS_ERROR])
                    .inc_by(eligible.len() as f64);
                warn!(
                    "SID lookup failed ({} ids, {:.0}ms): {}",
                    eligible.len(),
                    elapsed * 1000.0,
                    status
                );
                return HashMap::new();
            }
        };
        let elapsed = start.elapsed().as_secs_f64();
        self.metrics
            .lookup_latency_seconds
            .with_label_values(&[LATENCY_SUCCESS])
            .observe(elapsed);

        let mut out = HashMap::with_capacity(response.results.len());
        let mut hits: u64 = 0;
        let mut misses: u64 = 0;
        for (pid, PostSids { codes }) in eligible.iter().zip(response.results) {
            if codes.is_empty() {
                misses += 1;
            } else {
                out.insert(*pid, codes);
                hits += 1;
            }
        }
        if hits > 0 {
            self.metrics
                .lookup_count
                .with_label_values(&[STATUS_HIT])
                .inc_by(hits as f64);
        }
        if misses > 0 {
            self.metrics
                .lookup_count
                .with_label_values(&[STATUS_MISS])
                .inc_by(misses as f64);
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sid::TWITTER_EPOCH_MS;

    fn snowflake_at_ms(ms: i64) -> i64 {
        (ms - TWITTER_EPOCH_MS) << 22
    }

    #[test]
    fn rejects_https_endpoint() {
        let result = SidClient::new(
            "https://sid:50051",
            180.0,
            Duration::from_secs(10),
            SidMetrics::for_tests(),
        );
        let Err(err) = result else {
            panic!("https endpoint should be rejected");
        };
        let msg = err.to_string();
        assert!(msg.contains("TLS"), "got: {msg}");
    }

    #[tokio::test]
    async fn accepts_bare_and_http_endpoints() {
        for ep in ["sid.svc:50051", "http://sid.svc:50051"] {
            SidClient::new(ep, 0.0, Duration::from_secs(1), SidMetrics::for_tests())
                .unwrap_or_else(|e| panic!("endpoint {ep:?} should work, got {e}"));
        }
    }

    #[tokio::test]
    async fn empty_batch_short_circuits() {
        let c = SidClient::new(
            "sid:50051",
            180.0,
            Duration::from_secs(10),
            SidMetrics::for_tests(),
        )
        .unwrap();
        let r = c.lookup_sids(&[], None).await;
        assert!(r.is_empty());
    }

    #[tokio::test]
    async fn freshness_gate_filters_young_posts_without_rpc() {
        let metrics = SidMetrics::for_tests();
        let c = SidClient::new(
            "127.0.0.1:1",
            300.0,
            Duration::from_millis(50),
            metrics.clone(),
        )
        .unwrap();
        let now_ms = TWITTER_EPOCH_MS + 1_700_000_000_000;
        let young = snowflake_at_ms(now_ms);
        let r = c.lookup_sids(&[young], Some(now_ms)).await;
        assert!(r.is_empty());
        assert_eq!(
            metrics
                .lookup_count
                .with_label_values(&[STATUS_TOO_FRESH])
                .get() as u64,
            1
        );
        assert_eq!(
            metrics
                .lookup_count
                .with_label_values(&[STATUS_ERROR])
                .get() as u64,
            0
        );
    }

    #[tokio::test]
    async fn invalid_post_ids_dropped_before_rpc() {
        let metrics = SidMetrics::for_tests();
        let c = SidClient::new(
            "127.0.0.1:1",
            0.0,
            Duration::from_millis(50),
            metrics.clone(),
        )
        .unwrap();
        let r = c.lookup_sids(&[0, -1], None).await;
        assert!(r.is_empty());
        for status in [STATUS_TOO_FRESH, STATUS_ERROR, STATUS_HIT, STATUS_MISS] {
            assert_eq!(
                metrics.lookup_count.with_label_values(&[status]).get() as u64,
                0,
                "status={status}"
            );
        }
    }
}
