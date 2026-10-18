// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 X.AI Corp.
use std::env;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use bytes::Bytes;
use futures::stream::StreamExt;
use lazy_static::lazy_static;
use object_store::aws::AmazonS3Builder;
use object_store::path::Path as ObjectPath;
use object_store::{ClientOptions, ObjectMeta, ObjectStore, ObjectStoreExt};
use prometheus::{IntCounterVec, register_int_counter_vec};
use tokio::time::sleep;

use crate::o2::O2Client;

lazy_static! {
    static ref FETCHER_ERRORS: IntCounterVec = register_int_counter_vec!(
        "mm_embedding_fetcher_errors_total",
        "MM embedding fetcher errors by method and fatality",
        &["method", "fatal"]
    )
    .unwrap();
    static ref FETCHER_SOURCE: IntCounterVec = register_int_counter_vec!(
        "mm_embedding_fetcher_source_total",
        "Successful MM embedding fetches by source (o2 or data_loader) and method",
        &["source", "method"]
    )
    .unwrap();
}

const RETRY_TIMEOUT: Duration = Duration::from_secs(1);
const MAX_RETRIES: usize = 9;

#[derive(Clone)]
pub struct MmEmbeddingFetcher {
    primary: O2Client,
    fallback_store: Option<Arc<dyn ObjectStore>>,
    fallback_prefix: String,
}

impl MmEmbeddingFetcher {
    pub fn from_env(bucket: &str, prefix: &str) -> Result<Self> {
        let primary = O2Client::from_env(bucket, prefix)?;

        if env::var("MM_CACHE_FALLBACK_DISABLED")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false)
        {
            log::info!(
                "MM embedding fetcher: data loader fallback DISABLED via MM_CACHE_FALLBACK_DISABLED"
            );
            return Ok(Self {
                primary,
                fallback_store: None,
                fallback_prefix: String::new(),
            });
        }

        let cache_endpoint = env::var("MM_CACHE_ENDPOINT")
            .unwrap_or_else(|_| "http://mm-data-loader.example.invalid:9090".to_string());

        let fallback_store = Self::build_unsigned_store(bucket, &cache_endpoint);
        match &fallback_store {
            Some(_) => log::info!(
                "MM embedding fetcher: data loader fallback enabled at {}",
                cache_endpoint
            ),
            None => log::info!(
                "MM embedding fetcher: no data loader fallback ({})",
                cache_endpoint
            ),
        }

        Ok(Self {
            primary,
            fallback_store,
            fallback_prefix: prefix.trim_matches('/').to_string(),
        })
    }

    pub fn new(primary: O2Client) -> Self {
        Self {
            primary,
            fallback_store: None,
            fallback_prefix: String::new(),
        }
    }

    pub fn primary(&self) -> &O2Client {
        &self.primary
    }

    pub fn bucket(&self) -> &str {
        self.primary.bucket()
    }

    pub fn prefix(&self) -> &str {
        self.primary.prefix()
    }

    pub async fn list(&self, key: &str) -> Result<Vec<ObjectMeta>> {
        let mut attempt = 0;

        loop {
            match self.primary.try_list(key).await {
                Ok(objects) => {
                    FETCHER_SOURCE.with_label_values(&["o2", "list"]).inc();
                    return Ok(objects);
                }
                Err(primary_err) => {
                    FETCHER_ERRORS.with_label_values(&["list", "false"]).inc();
                    log::warn!("O2 list error: {}; attempt={}", primary_err, attempt + 1);

                    if let Some(fb_store) = &self.fallback_store {
                        let fb_prefix = self.fallback_full_path(key);
                        let fb_results: Vec<_> = fb_store.list(Some(&fb_prefix)).collect().await;
                        if let Ok(mut objects) = fb_results
                            .into_iter()
                            .collect::<std::result::Result<Vec<_>, _>>()
                        {
                            FETCHER_SOURCE
                                .with_label_values(&["data_loader", "list"])
                                .inc();
                            log::info!(
                                "O2 list failed, served from data loader (attempt={})",
                                attempt + 1
                            );
                            objects.sort_by_key(|b| std::cmp::Reverse(b.last_modified));
                            return Ok(objects);
                        }
                        log::warn!("Data loader list also failed (attempt={})", attempt + 1);
                    }

                    if attempt >= MAX_RETRIES {
                        FETCHER_ERRORS.with_label_values(&["list", "true"]).inc();
                        return Err(primary_err).context("Failed to list objects");
                    }

                    sleep(RETRY_TIMEOUT * 2u32.pow(attempt as u32)).await;
                    attempt += 1;
                }
            }
        }
    }

    pub async fn get_raw(&self, path: &ObjectPath) -> Result<Bytes> {
        let mut attempt = 0;

        loop {
            match self.primary.try_get_raw(path).await {
                Ok(bytes) => {
                    FETCHER_SOURCE.with_label_values(&["o2", "get_raw"]).inc();
                    return Ok(bytes);
                }
                Err(primary_err) => {
                    FETCHER_ERRORS
                        .with_label_values(&["get_raw", "false"])
                        .inc();
                    log::warn!(
                        "O2 get error for {}: {}; attempt={}",
                        path,
                        primary_err,
                        attempt + 1
                    );

                    if let Some(fb_store) = &self.fallback_store {
                        let fb_result = async {
                            let result = fb_store.get(path).await?;
                            result.bytes().await
                        }
                        .await;

                        if let Ok(bytes) = fb_result {
                            FETCHER_SOURCE
                                .with_label_values(&["data_loader", "get_raw"])
                                .inc();
                            return Ok(bytes);
                        }
                        log::warn!(
                            "Data loader get also failed for {} (attempt={})",
                            path,
                            attempt + 1
                        );
                    }

                    if attempt >= MAX_RETRIES {
                        FETCHER_ERRORS.with_label_values(&["get_raw", "true"]).inc();
                        return Err(primary_err)
                            .with_context(|| format!("Failed to get object: {}", path));
                    }

                    sleep(RETRY_TIMEOUT * 2u32.pow(attempt as u32)).await;
                    attempt += 1;
                }
            }
        }
    }

    fn fallback_full_path(&self, key: &str) -> ObjectPath {
        let key = key.trim_start_matches('/');
        if self.fallback_prefix.is_empty() {
            ObjectPath::from(key)
        } else {
            ObjectPath::from(format!("{}/{}", self.fallback_prefix, key))
        }
    }

    fn build_unsigned_store(bucket: &str, endpoint: &str) -> Option<Arc<dyn ObjectStore>> {
        let client_options = ClientOptions::default()
            .with_connect_timeout(Duration::from_secs(10))
            .with_timeout(Duration::from_secs(4 * 60))
            .with_allow_http(true);

        AmazonS3Builder::new()
            .with_bucket_name(bucket)
            .with_access_key_id("anonymous")
            .with_secret_access_key("anonymous")
            .with_endpoint(endpoint)
            .with_client_options(client_options)
            .build()
            .ok()
            .map(|s| Arc::new(s) as Arc<dyn ObjectStore>)
    }
}
