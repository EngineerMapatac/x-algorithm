use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use xai_manhattan::s2s::S2sConfig;
use xai_manhattan::{NativeManhattanClient, Tenant};

use crate::config::Config;

pub fn build_tenant(cfg: &Config) -> Tenant {
    Tenant {
        cluster: cfg.mh_cluster.clone(),
        app_id: cfg.mh_app_id.clone(),
        dataset: cfg.mh_dataset.clone(),
    }
}

pub async fn build_client(cfg: &Config) -> Result<Arc<NativeManhattanClient>> {
    let ca_cert_path = cfg.strato_ca_cert_path.clone().ok_or_else(|| {
        anyhow::anyhow!("Manhattan backend requires --strato-ca-cert-path (S2S CA)")
    })?;
    let client_cert_path = cfg.strato_client_cert_path.clone().ok_or_else(|| {
        anyhow::anyhow!("Manhattan backend requires --strato-client-cert-path (S2S client cert)")
    })?;
    let client_key_path = cfg.strato_client_key_path.clone().ok_or_else(|| {
        anyhow::anyhow!("Manhattan backend requires --strato-client-key-path (S2S client key)")
    })?;

    let s2s = S2sConfig {
        client_cert_path,
        client_key_path,
        ca_cert_path,
    };
    let tenant = build_tenant(cfg);
    let client = NativeManhattanClient::from_tenant_s2s(&tenant, &cfg.mh_datacenter, s2s)
        .await
        .map_err(|e| anyhow::anyhow!("Manhattan connect failed: {e}"))?;
    Ok(Arc::new(client))
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

pub fn ttl_nanos_from_secs(ttl_secs: u64) -> i64 {
    ttl_nanos_from_secs_at(now_secs(), ttl_secs)
}

pub fn remaining_ttl_secs(expires_at_nanos: Option<u64>) -> i64 {
    remaining_ttl_secs_at(expires_at_nanos, now_secs())
}

fn ttl_nanos_from_secs_at(now_secs: u64, ttl_secs: u64) -> i64 {
    (now_secs.saturating_add(ttl_secs) as i64).saturating_mul(1_000_000_000)
}

fn remaining_ttl_secs_at(expires_at_nanos: Option<u64>, now_secs: u64) -> i64 {
    match expires_at_nanos {
        Some(nanos) => {
            let expires_secs = (nanos / 1_000_000_000) as i64;
            (expires_secs - now_secs as i64).max(0)
        }
        None => -1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ttl_nanos_is_now_plus_ttl_in_nanos() {
        assert_eq!(ttl_nanos_from_secs_at(1_000, 60), 1_060 * 1_000_000_000);
        assert_eq!(ttl_nanos_from_secs_at(0, 86_400), 86_400 * 1_000_000_000);
    }

    #[test]
    fn remaining_ttl_counts_down_to_expiry() {
        assert_eq!(
            remaining_ttl_secs_at(Some(1_060 * 1_000_000_000), 1_000),
            60
        );
    }

    #[test]
    fn remaining_ttl_floors_past_due_at_zero() {
        assert_eq!(remaining_ttl_secs_at(Some(500 * 1_000_000_000), 1_000), 0);
    }

    #[test]
    fn remaining_ttl_no_expiry_is_negative_one() {
        assert_eq!(remaining_ttl_secs_at(None, 1_000), -1);
    }

    #[test]
    fn ttl_roundtrip_preserves_window() {
        let deadline = ttl_nanos_from_secs_at(1_000, 3_600);
        assert_eq!(remaining_ttl_secs_at(Some(deadline as u64), 1_000), 3_600);
    }
}
