use std::sync::Arc;

use tracing::warn;
use xai_manhattan::{LkeySelector, NativeManhattanClient, Tenant};

use crate::facts::EntityType;

pub use xai_abuse_enforcement_types::allowlist::{AllowlistEntry, AllowlistRecord};

pub(crate) const MH_PKEY: &str = "allowlist";

const MH_SCAN_LIMIT: u32 = 100_000;

#[derive(Clone)]
pub struct ManhattanAllowlist {
    client: Arc<NativeManhattanClient>,
    tenant: Tenant,
}

impl ManhattanAllowlist {
    pub fn new(client: Arc<NativeManhattanClient>, tenant: Tenant) -> Self {
        Self { client, tenant }
    }

    fn entity_lkey(entity_type: EntityType, entity_id: i64) -> String {
        match entity_type {
            EntityType::User => entity_id.to_string(),
            other => format!("{}:{}", other.as_str(), entity_id),
        }
    }

    pub async fn get(&self, user_id: i64) -> Option<AllowlistRecord> {
        let (entry, ttl_secs) = self.get_entity(EntityType::User, user_id).await?;
        Some(AllowlistRecord {
            user_id,
            added_by: entry.added_by,
            reason: entry.reason,
            added_at: entry.added_at,
            ttl_secs,
        })
    }

    pub async fn add(
        &self,
        user_id: i64,
        ttl_secs: u64,
        entry: &AllowlistEntry,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.add_entity(EntityType::User, user_id, ttl_secs, entry)
            .await
    }

    pub async fn remove(&self, user_id: i64) -> Result<bool, Box<dyn std::error::Error>> {
        self.remove_entity(EntityType::User, user_id).await
    }

    pub async fn get_entity(
        &self,
        entity_type: EntityType,
        entity_id: i64,
    ) -> Option<(AllowlistEntry, i64)> {
        let lkey = Self::entity_lkey(entity_type, entity_id);
        let item = match self
            .client
            .get(self.tenant.clone(), [MH_PKEY], [lkey.as_str()])
            .await
        {
            Ok(v) => v?,
            Err(e) => {
                warn!("Manhattan allowlist GET failed: {e}");
                crate::metrics::MANHATTAN_ERRORS_TOTAL.inc();
                return None;
            }
        };
        let entry: AllowlistEntry = serde_json::from_slice(item.value().as_bytes()).ok()?;
        let ttl_secs = crate::manhattan::remaining_ttl_secs(
            item.expires_at()
                .map(|d| d.timestamp_nanos_opt().unwrap_or(0) as u64),
        );
        Some((entry, ttl_secs))
    }

    pub async fn add_entity(
        &self,
        entity_type: EntityType,
        entity_id: i64,
        ttl_secs: u64,
        entry: &AllowlistEntry,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let lkey = Self::entity_lkey(entity_type, entity_id);
        let value = serde_json::to_vec(entry)?;
        let deadline = crate::manhattan::ttl_nanos_from_secs(ttl_secs);
        self.client
            .put_with_ttl(
                self.tenant.clone(),
                [MH_PKEY],
                [lkey.as_str()],
                value,
                deadline,
            )
            .await?;
        Ok(())
    }

    pub async fn remove_entity(
        &self,
        entity_type: EntityType,
        entity_id: i64,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let lkey = Self::entity_lkey(entity_type, entity_id);
        let existed = self
            .client
            .get(self.tenant.clone(), [MH_PKEY], [lkey.as_str()])
            .await?
            .is_some();
        self.client
            .delete(self.tenant.clone(), [MH_PKEY], [lkey.as_str()])
            .await?;
        Ok(existed)
    }

    pub async fn list_all(&self) -> Vec<AllowlistRecord> {
        let items = match self
            .client
            .scan(
                self.tenant.clone(),
                [MH_PKEY],
                LkeySelector::Range {
                    from: None,
                    to: None,
                },
                MH_SCAN_LIMIT,
            )
            .await
        {
            Ok(items) => items,
            Err(e) => {
                warn!("Manhattan allowlist scan failed: {e}");
                crate::metrics::MANHATTAN_ERRORS_TOTAL.inc();
                return Vec::new();
            }
        };

        let mut records = Vec::with_capacity(items.len());
        for item in items {
            let Some(id_bytes) = item.lkey().into_byte_vecs().into_iter().next() else {
                continue;
            };
            let Ok(user_id) = std::str::from_utf8(&id_bytes)
                .unwrap_or_default()
                .parse::<i64>()
            else {
                continue;
            };
            let ttl_secs = crate::manhattan::remaining_ttl_secs(
                item.expires_at()
                    .map(|d| d.timestamp_nanos_opt().unwrap_or(0) as u64),
            );
            if let Ok(entry) = serde_json::from_slice::<AllowlistEntry>(item.value().as_bytes()) {
                records.push(AllowlistRecord {
                    user_id,
                    added_by: entry.added_by,
                    reason: entry.reason,
                    added_at: entry.added_at,
                    ttl_secs,
                });
            }
        }
        records.sort_by_key(|r| r.user_id);
        records
    }
}
