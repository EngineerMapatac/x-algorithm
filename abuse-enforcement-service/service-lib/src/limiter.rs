use std::sync::Arc;

use tracing::{info, warn};
use xai_rate_limiter_client::rate_limiting_client::{ProdRateLimitingClient, RateLimitingClient};
use xai_rate_limiter_client::{Response, S2sConfig};

use crate::metrics;

pub const GLOBAL_BUCKET_USER_ID: i64 = 1;
pub const GLOBAL_BUCKET_POST_ID: i64 = 10;

#[derive(Clone, Debug)]
pub struct LimiterConfig {
    pub datacenter: String,
    pub feature: String,
    pub fail_open: bool,
    pub client_id: String,
    pub ca_cert_path: Option<String>,
    pub client_cert_path: Option<String>,
    pub client_key_path: Option<String>,
}

#[derive(Clone)]
pub struct LimiterClient {
    client: Arc<dyn RateLimitingClient + Send + Sync>,
    feature: String,
    bucket_user_id: i64,
    fail_open: bool,
}

impl LimiterClient {
    pub async fn connect(cfg: &LimiterConfig) -> anyhow::Result<Self> {
        let client: ProdRateLimitingClient = match (
            &cfg.ca_cert_path,
            &cfg.client_cert_path,
            &cfg.client_key_path,
        ) {
            (Some(ca), Some(cert), Some(key)) => {
                let s2s = S2sConfig::new(ca.clone(), cert.clone(), key.clone());
                ProdRateLimitingClient::new_with_s2s_and_client_id(
                    &cfg.datacenter,
                    s2s,
                    Some(cfg.client_id.clone()),
                )
                .await
                .map_err(|e| anyhow::anyhow!("limiter S2S connect: {e}"))?
            }
            _ => ProdRateLimitingClient::new(&cfg.datacenter)
                .await
                .map_err(|e| anyhow::anyhow!("limiter connect: {e}"))?,
        };

        info!(
            datacenter = cfg.datacenter,
            feature = cfg.feature,
            mtls = cfg.ca_cert_path.is_some(),
            "Limiter client connected (ThriftMux/WilyNS)"
        );

        Ok(Self {
            client: Arc::new(client),
            feature: cfg.feature.clone(),
            bucket_user_id: GLOBAL_BUCKET_USER_ID,
            fail_open: cfg.fail_open,
        })
    }

    pub fn with_bucket(&self, bucket_user_id: i64) -> Self {
        Self {
            client: self.client.clone(),
            feature: self.feature.clone(),
            bucket_user_id,
            fail_open: self.fail_open,
        }
    }

    async fn increment(&self) -> Option<Response> {
        match self
            .client
            .increment_feature(&self.feature, Some(self.bucket_user_id))
            .await
        {
            Ok(resp) => {
                let outcome = if resp.denial.is_some() {
                    "deny"
                } else {
                    "allow"
                };
                metrics::LIMITER_DECISIONS_TOTAL
                    .with_label_values(&[outcome])
                    .inc();
                Some(resp)
            }
            Err(e) => {
                metrics::LIMITER_DECISIONS_TOTAL
                    .with_label_values(&["error"])
                    .inc();
                warn!("Limiter error: {e}");
                None
            }
        }
    }

    pub async fn increment_count(&self) -> Option<i64> {
        let resp = self.increment().await?;
        resp.details
            .as_ref()
            .and_then(|d| d.total_requests)
            .map(|t| t as i64)
    }

    pub async fn try_record(&self) -> bool {
        match self.increment().await {
            Some(resp) => resp.denial.is_none(),
            None => self.fail_open,
        }
    }

    pub async fn record_report(&self) -> Option<serde_json::Value> {
        let resp = self.increment().await?;
        let usage = resp.usage.as_ref();
        Some(serde_json::json!({
            "allowed": resp.denial.is_none(),
            "limit": usage.and_then(|u| u.limit),
            "remaining": usage.and_then(|u| u.remaining),
            "reset_at": usage.and_then(|u| u.reset_at),
            "denial": resp.denial.as_ref().map(|d| serde_json::json!({
                "status_code": d.status_code,
                "message": d.message.clone(),
            })),
        }))
    }

    pub async fn stats(&self) -> Option<(u32, u32, u32)> {
        let usage = self
            .client
            .get_feature_usage(&self.feature, Some(self.bucket_user_id))
            .await
            .ok()?;
        let cap = usage.limit.unwrap_or(0).max(0) as u32;
        let remaining = usage.remaining.unwrap_or(0).max(0) as u32;
        let used = cap.saturating_sub(remaining);
        Some((used, cap, remaining))
    }
}
