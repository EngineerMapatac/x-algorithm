// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 X.AI Corp.
use std::env;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use bytes::Bytes;
use futures::stream::StreamExt;
use lazy_static::lazy_static;
use log::info;
use object_store::aws::AmazonS3Builder;
use object_store::path::Path as ObjectPath;
use object_store::{
    ClientOptions, ObjectMeta, ObjectStore, ObjectStoreExt, PutPayload, RetryConfig,
};
use prometheus::{IntCounterVec, register_int_counter_vec};
use tokio::time::sleep;

lazy_static! {
    static ref O2_ERRORS: IntCounterVec = register_int_counter_vec!(
        "o2_client_errors_total",
        "Total O2 storage client errors by method and fatality",
        &["method", "fatal"]
    )
    .unwrap();
}

const RETRY_TIMEOUT: Duration = Duration::from_secs(1);
const MAX_RETRIES: usize = 9;

#[derive(Clone)]
pub struct O2Client {
    store: Arc<dyn ObjectStore>,
    bucket: String,
    prefix: String,
}

impl O2Client {
    fn should_retry_error(err: &object_store::Error) -> bool {
        matches!(err, object_store::Error::Generic { .. })
    }

    pub fn from_env(bucket: &str, prefix: &str) -> Result<Self> {
        let access_key_id = env::var("MM_O2_ACCESS_KEY_ID")
            .context("O2_ACCESS_KEY_ID environment variable is required")?;
        let secret_access_key = env::var("MM_O2_SECRET_ACCESS_KEY")
            .context("O2_SECRET_ACCESS_KEY environment variable is required")?;
        let endpoint_url =
            env::var("MM_O2_ENDPOINT").unwrap_or("http://o2.example.invalid".to_string());

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
            .with_timeout(Duration::from_secs(4 * 60))
            .with_allow_http(true);
        let retry_config = RetryConfig {
            max_retries: 0,
            ..Default::default()
        };

        let store = AmazonS3Builder::new()
            .with_bucket_name(bucket)
            .with_access_key_id(access_key_id)
            .with_secret_access_key(secret_access_key)
            .with_endpoint(endpoint_url)
            .with_client_options(client_options)
            .with_retry(retry_config)
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
        let mut attempt = 0;
        loop {
            let result = self
                .store
                .put(&path, PutPayload::from_bytes(data.clone()))
                .await;
            match result {
                Ok(_) => break,
                Err(err) => {
                    if !Self::should_retry_error(&err) || attempt >= MAX_RETRIES {
                        O2_ERRORS.with_label_values(&["put", "true"]).inc();
                        return Err(err).with_context(|| format!("Failed to put object: {}", path));
                    }
                    O2_ERRORS.with_label_values(&["put", "false"]).inc();
                    log::warn!(
                        "ObjectStore error {}; attempt={}; retrying put...",
                        err,
                        attempt + 1
                    );
                    sleep(RETRY_TIMEOUT * 2u32.pow(attempt as u32)).await;
                    attempt += 1;
                }
            }
        }
        Ok(())
    }

    pub async fn list(&self, key: &str) -> Result<Vec<ObjectMeta>> {
        let prefix = self.full_path(key);
        let mut attempt = 0;
        let mut objects: Vec<ObjectMeta> = loop {
            let results: Vec<_> = self.store.list(Some(&prefix)).collect().await;

            match results
                .into_iter()
                .collect::<std::result::Result<Vec<_>, _>>()
            {
                Ok(objects) => break objects,
                Err(err) => {
                    if !Self::should_retry_error(&err) || attempt >= MAX_RETRIES {
                        O2_ERRORS.with_label_values(&["list", "true"]).inc();
                        return Err(err).context("Failed to list objects");
                    }
                    O2_ERRORS.with_label_values(&["list", "false"]).inc();
                    log::warn!(
                        "ObjectStore error {}; attempt={}; retrying list...",
                        err,
                        attempt + 1
                    );
                    sleep(RETRY_TIMEOUT * 2u32.pow(attempt as u32)).await;
                    attempt += 1;
                }
            }
        };

        objects.sort_by_key(|b| std::cmp::Reverse(b.last_modified));

        Ok(objects)
    }

    pub async fn get_raw(&self, path: &ObjectPath) -> Result<Bytes> {
        let mut attempt = 0;
        loop {
            let result = match self.store.get(path).await {
                Ok(result) => result,
                Err(err) => {
                    if !Self::should_retry_error(&err) || attempt >= MAX_RETRIES {
                        O2_ERRORS.with_label_values(&["get_raw", "true"]).inc();
                        return Err(err).with_context(|| format!("Failed to get object: {}", path));
                    }
                    O2_ERRORS.with_label_values(&["get_raw", "false"]).inc();
                    log::warn!(
                        "ObjectStore error {}; attempt={}; retrying get_raw...",
                        err,
                        attempt + 1
                    );
                    sleep(RETRY_TIMEOUT * 2u32.pow(attempt as u32)).await;
                    attempt += 1;
                    continue;
                }
            };
            match result.bytes().await {
                Ok(bytes) => return Ok(bytes),
                Err(err) => {
                    if !Self::should_retry_error(&err) || attempt >= MAX_RETRIES {
                        O2_ERRORS.with_label_values(&["get_raw", "true"]).inc();
                        return Err(err).with_context(|| format!("Failed to get object: {}", path));
                    }
                    O2_ERRORS.with_label_values(&["get_raw", "false"]).inc();
                    log::warn!(
                        "ObjectStore error {}; attempt={}; retrying get_raw...",
                        err,
                        attempt + 1
                    );
                    sleep(RETRY_TIMEOUT * 2u32.pow(attempt as u32)).await;
                    attempt += 1;
                    continue;
                }
            }
        }
    }

    pub async fn try_list(&self, key: &str) -> Result<Vec<ObjectMeta>> {
        let prefix = self.full_path(key);
        let results: Vec<_> = self.store.list(Some(&prefix)).collect().await;
        let mut objects: Vec<ObjectMeta> = results
            .into_iter()
            .collect::<std::result::Result<Vec<_>, _>>()
            .context("Failed to list objects")?;
        objects.sort_by_key(|b| std::cmp::Reverse(b.last_modified));
        Ok(objects)
    }

    pub async fn try_get_raw(&self, path: &ObjectPath) -> Result<Bytes> {
        let result = self
            .store
            .get(path)
            .await
            .with_context(|| format!("Failed to get object: {}", path))?;
        result
            .bytes()
            .await
            .with_context(|| format!("Failed to read bytes: {}", path))
    }

    pub async fn delete(&self, path: &ObjectPath) -> Result<()> {
        self.store
            .delete(path)
            .await
            .inspect_err(|_| {
                O2_ERRORS.with_label_values(&["delete", "true"]).inc();
            })
            .with_context(|| format!("Failed to delete object: {}", path))?;
        Ok(())
    }
}
