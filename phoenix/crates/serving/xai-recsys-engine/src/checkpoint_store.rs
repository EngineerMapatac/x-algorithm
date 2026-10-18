// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 X.AI Corp.
use async_trait::async_trait;
use bytes::Bytes;
use log::{error, info, warn};
use object_store::aws::AmazonS3Builder;
use object_store::chunked::ChunkedStore;
use object_store::gcp::{GcpCredential, GoogleCloudStorageBuilder};
use object_store::path::Path as ObjectPath;
use object_store::{
    ClientOptions, ObjectStore, ObjectStoreExt, PutPayload, RetryConfig, WriteMultipart,
};

use futures::TryStreamExt;
use std::collections::HashMap;
use std::env;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;
use tokio::sync::Semaphore;

use xai_o2::{O2ClientBuilder, base_client::BaseO2Client};

const MAX_CONCURRENT: usize = 16;

const MAX_RETRIES: usize = 6;

const MULTIPART_THRESHOLD: usize = 100 * 1024 * 1024;

const MULTIPART_PART_SIZE: usize = 32 * 1024 * 1024;

const MAX_CONCURRENT_WRITE_REQUESTS: usize = 1;

fn env_usize(name: &str, default: usize) -> usize {
    match env::var(name) {
        Ok(raw) => match raw.trim().parse::<usize>() {
            Ok(v) => v,
            Err(_) => {
                warn!(
                    "[checkpoint-store] invalid {}={:?}, using default {}",
                    name, raw, default
                );
                default
            }
        },
        Err(_) => default,
    }
}

pub struct CheckpointStore {
    client: Arc<dyn ObjectStore>,
    sem: Arc<Semaphore>,
}

impl CheckpointStore {
    fn new(client: Arc<dyn ObjectStore>) -> Self {
        Self {
            client,
            sem: Arc::new(Semaphore::new(MAX_CONCURRENT)),
        }
    }

    pub async fn get(&self, path: &ObjectPath, label: &str) -> Result<Bytes, String> {
        self.with_retry(label, || async {
            let data = self
                .client
                .get(path)
                .await
                .map_err(|e| format!("GET {} failed: {}", label, e))?;
            data.bytes()
                .await
                .map_err(|e| format!("read bytes {} failed: {}", label, e))
        })
        .await
    }

    pub async fn put(&self, path: &ObjectPath, data: Bytes, label: &str) -> Result<(), String> {
        let use_multipart = data.len() >= MULTIPART_THRESHOLD;
        self.with_retry(label, || {
            let data = data.clone();
            let path = path.clone();
            async move {
                if use_multipart {
                    let upload = self
                        .client
                        .put_multipart(&path)
                        .await
                        .map_err(|e| format!("PUT multipart {} failed: {}", label, e))?;
                    let mut writer =
                        WriteMultipart::new_with_chunk_size(upload, MULTIPART_PART_SIZE);
                    for chunk in data.chunks(MULTIPART_PART_SIZE) {
                        writer
                            .wait_for_capacity(MAX_CONCURRENT)
                            .await
                            .map_err(|e| format!("PUT multipart {} capacity: {}", label, e))?;
                        writer.write(chunk);
                    }
                    writer
                        .finish()
                        .await
                        .map(|_| ())
                        .map_err(|e| format!("PUT multipart {} finish: {}", label, e))
                } else {
                    self.client
                        .put(&path, PutPayload::from_bytes(data))
                        .await
                        .map(|_| ())
                        .map_err(|e| format!("PUT {} failed: {}", label, e))
                }
            }
        })
        .await
    }

    pub async fn list(
        &self,
        prefix: &ObjectPath,
        label: &str,
    ) -> Result<Vec<object_store::ObjectMeta>, String> {
        self.with_retry(label, || async {
            self.client
                .list(Some(prefix))
                .try_collect()
                .await
                .map_err(|e| format!("LIST {} failed: {}", label, e))
        })
        .await
    }

    async fn with_retry<F, Fut, T>(&self, label: &str, op: F) -> Result<T, String>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<T, String>>,
    {
        let mut last_error: Option<String> = None;
        for attempt in 1..=MAX_RETRIES {
            let result = {
                let _permit = self
                    .sem
                    .acquire()
                    .await
                    .map_err(|e| format!("semaphore: {}", e))?;
                op().await
            };
            match result {
                Ok(v) => {
                    if attempt > 1 {
                        info!(
                            "[checkpoint-store] {} succeeded after {} attempts",
                            label, attempt
                        );
                    }
                    return Ok(v);
                }
                Err(e) => {
                    last_error = Some(e.clone());
                    if attempt < MAX_RETRIES {
                        let backoff = Duration::from_millis(1000 * (1 << (attempt - 1)));
                        error!(
                            "[checkpoint-store] {} failed (attempt {}/{}): {}. Retrying in {:?}",
                            label, attempt, MAX_RETRIES, e, backoff
                        );
                        tokio::time::sleep(backoff).await;
                    }
                }
            }
        }
        Err(format!(
            "[checkpoint-store] {} failed after {} attempts. Last error: {}",
            label,
            MAX_RETRIES,
            last_error.unwrap_or_else(|| "unknown".to_string()),
        ))
    }
}

pub struct XaiO2Store {
    client: Arc<dyn BaseO2Client>,
}

impl XaiO2Store {
    pub async fn put(&self, path: &ObjectPath, data: Bytes, label: &str) -> Result<(), String> {
        self.client
            .put(path.as_ref(), data)
            .await
            .map_err(|e| format!("PUT {} failed: {}", label, e))
    }
}

#[derive(Clone)]
pub enum Store {
    ObjectStore(Arc<CheckpointStore>),
    XaiO2(Arc<XaiO2Store>),
}

impl Store {
    pub async fn put(&self, path: &ObjectPath, data: Bytes, label: &str) -> Result<(), String> {
        match self {
            Store::ObjectStore(s) => s.put(path, data, label).await,
            Store::XaiO2(s) => s.put(path, data, label).await,
        }
    }
}

#[derive(Debug)]
struct WorkloadIdentityCredentialProvider {
    inner: google_cloud_auth::credentials::AccessTokenCredentials,
}

impl WorkloadIdentityCredentialProvider {
    fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let contents = std::fs::read_to_string(path)?;
        let config: serde_json::Value = serde_json::from_str(&contents)?;

        let cred_type = config.get("type").and_then(|v| v.as_str()).unwrap_or("");
        if cred_type != "external_account" {
            return Err(format!(
                "WorkloadIdentityCredentialProvider only handles external_account, got {:?}",
                cred_type
            )
            .into());
        }

        let creds = google_cloud_auth::credentials::external_account::Builder::new(config)
            .with_scopes(["https://www.googleapis.com/auth/cloud-platform"])
            .build_access_token_credentials()
            .map_err(|e| format!("failed to build external_account credentials: {}", e))?;

        Ok(Self { inner: creds })
    }

    fn from_file_on_runtime(
        path: &str,
        runtime: &tokio::runtime::Runtime,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let path = path.to_string();
        let _guard = runtime.enter();
        Self::from_file(&path)
    }
}

#[async_trait]
impl object_store::client::CredentialProvider for WorkloadIdentityCredentialProvider {
    type Credential = GcpCredential;

    async fn get_credential(&self) -> object_store::Result<Arc<GcpCredential>> {
        let token = self
            .inner
            .access_token()
            .await
            .map_err(|e| object_store::Error::Generic {
                store: "GCS",
                source: format!("workload identity token fetch failed: {}", e).into(),
            })?;

        Ok(Arc::new(GcpCredential {
            bearer: token.token,
        }))
    }
}

fn is_external_account_credential(path: &str) -> bool {
    let Ok(contents) = std::fs::read_to_string(path) else {
        return false;
    };
    let Ok(v) = serde_json::from_str::<serde_json::Value>(&contents) else {
        return false;
    };
    v.get("type").and_then(|t| t.as_str()) == Some("external_account")
}

static STORES: OnceLock<Mutex<HashMap<String, Arc<CheckpointStore>>>> = OnceLock::new();

static XAI_O2_STORES: OnceLock<Mutex<HashMap<String, Arc<XaiO2Store>>>> = OnceLock::new();

pub fn get_checkpoint_store(
    url: &str,
    runtime: Option<&tokio::runtime::Runtime>,
) -> Result<(Arc<CheckpointStore>, ObjectPath), Box<dyn std::error::Error + Send + Sync>> {
    let is_gcs = url.starts_with("gs://");

    let parsed = url::Url::parse(url)?;
    let bucket = parsed
        .host_str()
        .ok_or_else(|| format!("No bucket in URL: {}", url))?
        .to_string();
    let path = parsed.path().trim_start_matches('/');

    let cache_key = if is_gcs {
        format!("gs://{}", bucket)
    } else {
        format!("s3://{}", bucket)
    };

    let stores = STORES.get_or_init(|| Mutex::new(HashMap::new()));
    let mut map = stores.lock().map_err(|e| format!("lock poisoned: {}", e))?;

    if let Some(store) = map.get(&cache_key) {
        return Ok((store.clone(), ObjectPath::from(path)));
    }

    let client: Arc<dyn ObjectStore> = if is_gcs {
        let mut builder = GoogleCloudStorageBuilder::new().with_bucket_name(&bucket);

        let cred_path = env::var("GOOGLE_APPLICATION_CREDENTIALS").ok();
        if let Some(ref cp) = cred_path {
            if is_external_account_credential(cp) {
                info!(
                    "[checkpoint-store] Using WorkloadIdentityCredentialProvider for {}",
                    cp,
                );
                let provider = if let Some(rt) = runtime {
                    WorkloadIdentityCredentialProvider::from_file_on_runtime(cp, rt)?
                } else {
                    WorkloadIdentityCredentialProvider::from_file(cp)?
                };
                builder = builder.with_credentials(Arc::new(provider));
            } else {
                builder = builder.with_application_credentials(cp);
            }
        }

        let retry_config = RetryConfig {
            max_retries: 0,
            ..Default::default()
        };
        builder = builder.with_retry(retry_config);

        let client_options = ClientOptions::default()
            .with_connect_timeout(Duration::from_secs(30))
            .with_timeout(Duration::from_secs(300));
        builder = builder.with_client_options(client_options);

        Arc::new(builder.build()?)
    } else {
        let client_options = ClientOptions::default()
            .with_connect_timeout(Duration::from_secs(30))
            .with_timeout(Duration::from_secs(300))
            .with_allow_http(true);

        let retry_config = RetryConfig {
            max_retries: 0,
            ..Default::default()
        };

        let endpoint = env::var("O2_ENDPOINT_URL").map_err(|_| {
            "O2 endpoint not set: set XAI_O2_ENDPOINT_URL or O2_ENDPOINT_URL".to_string()
        })?;
        let access_key = env::var("O2_ACCESS_KEY_ID").unwrap_or_else(|_| "anonymous".to_string());
        let secret_key =
            env::var("O2_SECRET_ACCESS_KEY").unwrap_or_else(|_| "anonymous".to_string());

        let builder = AmazonS3Builder::new()
            .with_region("global")
            .with_virtual_hosted_style_request(false)
            .with_endpoint(endpoint)
            .with_bucket_name(&bucket)
            .with_client_options(client_options)
            .with_retry(retry_config);
        let skip_signature = access_key == "anonymous" && secret_key == "anonymous";
        let builder = builder
            .with_access_key_id(access_key)
            .with_secret_access_key(secret_key)
            .with_skip_signature(skip_signature);

        let client = builder.build()?;

        let chunk_size = env::var("O2_CHUNK_SIZE_BYTES")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(0);
        if chunk_size > 0 {
            Arc::new(ChunkedStore::new(Arc::new(client), chunk_size))
        } else {
            Arc::new(client)
        }
    };

    let store = Arc::new(CheckpointStore::new(client));
    map.insert(cache_key, store.clone());

    Ok((store, ObjectPath::from(path)))
}

fn is_o2_scheme(url: &str) -> bool {
    url.starts_with("s3://") || url.starts_with("s3-o2://")
}

fn use_xai_o2_for(url: &str) -> bool {
    if !is_o2_scheme(url) {
        return false;
    }
    match env::var("XAI_RECSYS_S3_BACKEND") {
        Ok(raw) => {
            let v = raw.trim();
            if v.eq_ignore_ascii_case("xai-o2") {
                true
            } else {
                if !v.is_empty() {
                    warn!(
                        "[checkpoint-store] unknown XAI_RECSYS_S3_BACKEND={:?}, \
                         using object_store (set to \"xai-o2\" for the xai-o2 backend)",
                        raw
                    );
                }
                false
            }
        }
        Err(_) => false,
    }
}

pub fn get_store(
    url: &str,
    runtime: Option<&tokio::runtime::Runtime>,
) -> Result<(Store, ObjectPath), Box<dyn std::error::Error + Send + Sync>> {
    if use_xai_o2_for(url) {
        let (store, path) = get_xai_o2_store(url)?;
        return Ok((Store::XaiO2(store), path));
    }
    let (store, path) = get_checkpoint_store(url, runtime)?;
    Ok((Store::ObjectStore(store), path))
}

fn get_xai_o2_store(
    url: &str,
) -> Result<(Arc<XaiO2Store>, ObjectPath), Box<dyn std::error::Error + Send + Sync>> {
    let parsed = url::Url::parse(url)?;
    let bucket = parsed
        .host_str()
        .ok_or_else(|| format!("No bucket in URL: {}", url))?
        .to_string();
    let path = parsed.path().trim_start_matches('/');

    let cache_key = format!("s3://{}", bucket);
    let stores = XAI_O2_STORES.get_or_init(|| Mutex::new(HashMap::new()));
    let mut map = stores.lock().map_err(|e| format!("lock poisoned: {}", e))?;
    if let Some(store) = map.get(&cache_key) {
        return Ok((store.clone(), ObjectPath::from(path)));
    }

    let store = Arc::new(XaiO2Store {
        client: build_xai_o2_client(&bucket)?,
    });
    map.insert(cache_key, store.clone());
    Ok((store, ObjectPath::from(path)))
}

fn build_xai_o2_client(
    bucket: &str,
) -> Result<Arc<dyn BaseO2Client>, Box<dyn std::error::Error + Send + Sync>> {
    let endpoint = env::var("O2_ENDPOINT_URL").map_err(|_| {
        "O2 endpoint not set: set XAI_O2_ENDPOINT_URL or O2_ENDPOINT_URL".to_string()
    })?;
    let access_key = env::var("O2_ACCESS_KEY_ID").unwrap_or_else(|_| "anonymous".to_string());
    let secret_key = env::var("O2_SECRET_ACCESS_KEY").unwrap_or_else(|_| "anonymous".to_string());
    let skip_signature = access_key == "anonymous" && secret_key == "anonymous";

    let max_write_reqs = env_usize(
        "XAI_RECSYS_O2_MAX_CONCURRENT_WRITE_REQUESTS",
        MAX_CONCURRENT_WRITE_REQUESTS,
    )
    .max(1);
    let write_chunk_concurrency =
        env_usize("XAI_RECSYS_O2_WRITE_CHUNK_CONCURRENCY", MAX_CONCURRENT).max(1);
    let write_chunk_size_mib = env_usize(
        "XAI_RECSYS_O2_WRITE_CHUNK_SIZE_MIB",
        MULTIPART_PART_SIZE / (1024 * 1024),
    )
    .max(1);

    info!(
        "[checkpoint-store] Creating xai-o2 client bucket={} \
         max_concurrent_write_requests={} write_chunk_concurrency={} \
         write_chunk_size_mib={}",
        bucket, max_write_reqs, write_chunk_concurrency, write_chunk_size_mib
    );

    let mut builder = O2ClientBuilder::default();
    builder
        .endpoint(endpoint)
        .bucket(bucket)
        .prefix("")
        .write_chunk_size_mib(write_chunk_size_mib)
        .write_chunk_concurrency(write_chunk_concurrency)
        .max_concurrent_write_requests(max_write_reqs)
        .retry_max_times(MAX_RETRIES)
        .retry_min_delay(Duration::from_secs(1))
        .retry_max_delay(Duration::from_secs(32))
        .timeout(Duration::from_secs(300))
        .io_timeout(Duration::from_secs(300))
        .trace_always(false);
    if skip_signature {
        builder.allow_anonymous(true);
    } else {
        builder
            .access_key_id(access_key)
            .secret_access_key(secret_key);
    }
    builder
        .build()
        .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { e.into() })
}
