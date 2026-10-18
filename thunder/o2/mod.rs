use std::env;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use bytes::Bytes;
use futures::TryStreamExt;
use log::info;
use object_store::aws::AmazonS3Builder;
use object_store::path::Path as ObjectPath;
use object_store::{ClientOptions, ObjectStore, ObjectStoreExt, PutPayload};

#[derive(Clone)]
pub struct O2Client {
    store: Arc<dyn ObjectStore>,
    bucket: String,
    prefix: String,
}

impl O2Client {
    pub fn from_env(bucket: &str, prefix: &str) -> Result<Self> {
        let access_key_id = env::var("O2_ACCESS_KEY_ID")
            .context("O2_ACCESS_KEY_ID environment variable is required")?;
        let secret_access_key = env::var("O2_SECRET_ACCESS_KEY")
            .context("O2_SECRET_ACCESS_KEY environment variable is required")?;
        let endpoint_url = env::var("O2_ENDPOINT_URL")
            .context("O2_ENDPOINT_URL environment variable is required")?;

        Self::new(
            bucket,
            prefix,
            &access_key_id,
            &secret_access_key,
            &endpoint_url,
        )
    }

    pub fn new(
        bucket: &str,
        prefix: &str,
        access_key_id: &str,
        secret_access_key: &str,
        endpoint_url: &str,
    ) -> Result<Self> {
        let client_options = ClientOptions::default()
            .with_connect_timeout(Duration::from_secs(30))
            .with_timeout(Duration::from_secs(5 * 60))
            .with_allow_http(true);

        let store = AmazonS3Builder::new()
            .with_bucket_name(bucket)
            .with_access_key_id(access_key_id)
            .with_secret_access_key(secret_access_key)
            .with_endpoint(endpoint_url)
            .with_client_options(client_options)
            .build()
            .context("Failed to build O2 S3 client")?;

        info!(
            "O2 client initialized: bucket={}, prefix={}, endpoint={}",
            bucket, prefix, endpoint_url
        );

        Ok(Self {
            store: Arc::new(store),
            bucket: bucket.to_string(),
            prefix: prefix.trim_matches('/').to_string(),
        })
    }

    fn full_path(&self, key: &str) -> ObjectPath {
        let key = key.trim_start_matches('/');
        if self.prefix.is_empty() {
            ObjectPath::from(key)
        } else {
            ObjectPath::from(format!("{}/{}", self.prefix, key))
        }
    }

    pub fn bucket(&self) -> &str {
        &self.bucket
    }

    pub fn prefix(&self) -> &str {
        &self.prefix
    }

    pub async fn put(&self, key: &str, data: Bytes) -> Result<()> {
        let path = self.full_path(key);
        self.store
            .put(&path, PutPayload::from_bytes(data))
            .await
            .with_context(|| format!("Failed to put object: {}", path))?;
        Ok(())
    }

    pub async fn get(&self, key: &str) -> Result<Bytes> {
        let path = self.full_path(key);
        let result = self
            .store
            .get(&path)
            .await
            .with_context(|| format!("Failed to get object: {}", path))?;
        let bytes = result
            .bytes()
            .await
            .with_context(|| format!("Failed to read object bytes: {}", path))?;
        Ok(bytes)
    }

    pub async fn exists(&self, key: &str) -> Result<bool> {
        let path = self.full_path(key);
        match self.store.head(&path).await {
            Ok(_) => Ok(true),
            Err(object_store::Error::NotFound { .. }) => Ok(false),
            Err(e) => Err(anyhow!("Failed to check object existence: {}", e)),
        }
    }

    pub async fn delete(&self, key: &str) -> Result<()> {
        let path = self.full_path(key);
        self.store
            .delete(&path)
            .await
            .with_context(|| format!("Failed to delete object: {}", path))?;
        Ok(())
    }

    pub async fn list(&self, key_prefix: &str) -> Result<Vec<String>> {
        let path = self.full_path(key_prefix);
        let prefix_str = format!("{}/", self.prefix);

        let objects: Vec<_> = self
            .store
            .list(Some(&path))
            .try_collect()
            .await
            .with_context(|| format!("Failed to list objects with prefix: {}", path))?;

        let keys: Vec<String> = objects
            .into_iter()
            .map(|meta| {
                let full_key = meta.location.to_string();
                if self.prefix.is_empty() {
                    full_key
                } else {
                    full_key
                        .strip_prefix(&prefix_str)
                        .unwrap_or(&full_key)
                        .to_string()
                }
            })
            .collect();

        Ok(keys)
    }

    pub async fn head(&self, key: &str) -> Result<ObjectMeta> {
        let path = self.full_path(key);
        let meta = self
            .store
            .head(&path)
            .await
            .with_context(|| format!("Failed to get object metadata: {}", path))?;
        Ok(ObjectMeta {
            size: meta.size,
            last_modified_ms: meta.last_modified.timestamp_millis(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct ObjectMeta {
    pub size: u64,
    pub last_modified_ms: i64,
}
