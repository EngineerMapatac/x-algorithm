// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 X.AI Corp.
use bytes::Bytes;
use futures::future::join_all;
use futures::{StreamExt, TryStreamExt};
use google_cloud_storage::client::{Storage, StorageControl};

use crate::checkpoint_store::{get_checkpoint_store, get_store};
use log::{debug, error, info};
use numpy::PyReadwriteArray1;
use object_store::aws::AmazonS3Builder;
use object_store::chunked::ChunkedStore;
use object_store::path::Path as ObjectPath;
use object_store::{
    ClientOptions, ObjectStore, ObjectStoreExt, PutPayload, RetryConfig, WriteMultipart,
};
use pyo3::prelude::*;
use pyo3::types::{PyList, PyTuple};
use std::collections::HashSet;
use std::env;
use std::fs;
use std::mem;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, OnceLock};
use std::time::Duration;
use tokio::runtime::Runtime;
use tokio::task;

static GCS_RUNTIME: OnceLock<Runtime> = OnceLock::new();
static O2_RUNTIME: OnceLock<Runtime> = OnceLock::new();

fn get_runtime() -> &'static Runtime {
    GCS_RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .worker_threads(128)
            .thread_name("gcs-upload")
            .build()
            .expect("Failed to create GCS upload runtime")
    })
}

fn get_o2_runtime() -> &'static Runtime {
    O2_RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .worker_threads(128)
            .thread_name("o2-storage")
            .build()
            .expect("Failed to create O2 storage runtime")
    })
}

const MAX_CONCURRENT_UPLOADS: usize = 16;

const MULTIPART_THRESHOLD: usize = 100 * 1024 * 1024;

const MULTIPART_PART_SIZE: usize = 16 * 1024 * 1024;

fn o2_store_from_url(
    url: &str,
) -> Result<(Arc<dyn ObjectStore>, ObjectPath), Box<dyn std::error::Error + Send + Sync>> {
    let parsed = url::Url::parse(url)?;
    let bucket = parsed
        .host_str()
        .ok_or_else(|| format!("No bucket in URL: {}", url))?;
    let path = parsed.path().trim_start_matches('/');

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
    let secret_key = env::var("O2_SECRET_ACCESS_KEY").unwrap_or_else(|_| "anonymous".to_string());

    let builder = AmazonS3Builder::new()
        .with_region("global")
        .with_virtual_hosted_style_request(false)
        .with_endpoint(endpoint)
        .with_bucket_name(bucket)
        .with_client_options(client_options)
        .with_retry(retry_config);
    let skip_signature = access_key == "anonymous" && secret_key == "anonymous";
    let builder = builder
        .with_access_key_id(access_key)
        .with_secret_access_key(secret_key)
        .with_skip_signature(skip_signature);

    let store = builder.build()?;

    let chunk_size = env::var("O2_CHUNK_SIZE_BYTES")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(0);
    let store: Arc<dyn ObjectStore> = if chunk_size > 0 {
        Arc::new(ChunkedStore::new(Arc::new(store), chunk_size))
    } else {
        Arc::new(store)
    };

    Ok((store, ObjectPath::from(path)))
}

fn o2_shared_store(
    url: &str,
) -> Result<(Arc<dyn ObjectStore>, String), Box<dyn std::error::Error + Send + Sync>> {
    let (store, base_path) = o2_store_from_url(url)?;
    Ok((store, base_path.as_ref().to_string()))
}

async fn upload_with_retry(
    store: &Arc<dyn ObjectStore>,
    object_path: &ObjectPath,
    data: Bytes,
    label: &str,
    max_retries: usize,
) -> Result<(), String> {
    let use_multipart = data.len() >= MULTIPART_THRESHOLD;
    let mut last_error: Option<String> = None;

    for attempt in 1..=max_retries {
        let result = if use_multipart {
            async {
                let upload = store.put_multipart(object_path).await?;
                let mut writer = WriteMultipart::new_with_chunk_size(upload, MULTIPART_PART_SIZE);
                for chunk in data.chunks(MULTIPART_PART_SIZE) {
                    writer.wait_for_capacity(MAX_CONCURRENT_UPLOADS).await?;
                    writer.write(chunk);
                }
                writer.finish().await.map(|_| ())
            }
            .await
        } else {
            store
                .put(object_path, PutPayload::from_bytes(data.clone()))
                .await
                .map(|_| ())
        };

        match result {
            Ok(()) => {
                if attempt > 1 {
                    info!("Successfully uploaded {} after {} attempts", label, attempt);
                }
                debug!("Uploaded {}", label);
                return Ok(());
            }
            Err(e) => {
                last_error = Some(format!("{}", e));
                if attempt < max_retries {
                    let backoff = Duration::from_millis(1000 * (1 << (attempt - 1)));
                    error!(
                        "Failed to upload {} (attempt {}/{}): {}. Retrying in {:?}",
                        label, attempt, max_retries, e, backoff
                    );
                    tokio::time::sleep(backoff).await;
                }
            }
        }
    }

    Err(format!(
        "Failed to upload {} after {} attempts. Last error: {}",
        label,
        max_retries,
        last_error.unwrap_or_else(|| "unknown".to_string())
    ))
}

fn is_o2_url(url: &str) -> bool {
    url.starts_with("s3-o2://") || url.starts_with("s3://")
}

fn sanitize_path_for_o2(path: &str) -> String {
    path.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric()
                || c == '-'
                || c == '_'
                || c == '='
                || c == ' '
                || c == '.'
                || c == '@'
                || c == '+'
                || c == '('
                || c == ')'
                || c == '!'
                || c == '/'
            {
                c
            } else {
                '_'
            }
        })
        .collect()
}

#[pyfunction]
pub fn upload_files_to_storage(
    local_base_dir: String,
    relative_paths: Vec<String>,
    storage_url: String,
) -> PyResult<()> {
    let num_files = relative_paths.len();
    let is_o2 = is_o2_url(&storage_url);

    let upload_runtime = if is_o2 {
        get_o2_runtime()
    } else {
        get_runtime()
    };

    info!(
        "[checkpoint-store] Starting upload of {} files to {} (blocking)",
        num_files, storage_url,
    );
    let upload_start = std::time::Instant::now();
    let store_result = get_store(&storage_url, Some(upload_runtime));
    match store_result {
        Ok((store, base_path)) => {
            let base = base_path.as_ref().to_string();
            let errors = Arc::new(AtomicUsize::new(0));
            let finished = Arc::new(AtomicUsize::new(0));
            let mut handles = Vec::with_capacity(num_files);
            for relative_path in relative_paths {
                let base_dir = local_base_dir.clone();
                let store = store.clone();
                let base = base.clone();
                let errors = errors.clone();
                let finished = finished.clone();
                let total = num_files;

                handles.push(upload_runtime.spawn(async move {
                    let local_path =
                        format!("{}/{}", base_dir.trim_end_matches('/'), relative_path);
                    let contents = match task::spawn_blocking({
                        let path = local_path.clone();
                        move || fs::read(&path)
                    })
                    .await
                    {
                        Ok(Ok(c)) => c,
                        Ok(Err(e)) => {
                            error!("[checkpoint-store] read {} failed: {}", local_path, e);
                            errors.fetch_add(1, Ordering::Relaxed);
                            finished.fetch_add(1, Ordering::Relaxed);
                            return;
                        }
                        Err(e) => {
                            error!("[checkpoint-store] spawn read {} failed: {}", local_path, e);
                            errors.fetch_add(1, Ordering::Relaxed);
                            finished.fetch_add(1, Ordering::Relaxed);
                            return;
                        }
                    };
                    let buf = Bytes::from(contents);
                    let key = if is_o2 {
                        sanitize_path_for_o2(&relative_path)
                    } else {
                        relative_path.clone()
                    };
                    let path = ObjectPath::from(format!("{}/{}", base, key));

                    if let Err(e) = store.put(&path, buf, &relative_path).await {
                        error!(
                            "[checkpoint-store] upload failed for {}: {}",
                            relative_path, e
                        );
                        errors.fetch_add(1, Ordering::Relaxed);
                    }
                    let done = finished.fetch_add(1, Ordering::Relaxed) + 1;
                    if done.is_multiple_of(20) || done == total {
                        info!("[checkpoint-store] {}/{} files uploaded", done, total);
                    }
                }));
            }
            upload_runtime.block_on(async {
                for handle in handles {
                    let _ = handle.await;
                }
            });
            let elapsed = upload_start.elapsed();
            let err_count = errors.load(Ordering::Relaxed);
            if err_count == 0 {
                info!(
                    "[checkpoint-store] All {} files uploaded to {} in {:.1}s",
                    num_files,
                    storage_url,
                    elapsed.as_secs_f64(),
                );
            } else {
                let msg = format!(
                    "[checkpoint-store] {}/{} files FAILED uploading to {} in {:.1}s",
                    err_count,
                    num_files,
                    storage_url,
                    elapsed.as_secs_f64(),
                );
                error!("{}", msg);
                return Err(pyo3::exceptions::PyRuntimeError::new_err(msg));
            }
        }
        Err(e) => {
            let msg = format!(
                "[checkpoint-store] Failed to create store for {}: {}",
                storage_url, e
            );
            error!("{}", msg);
            return Err(pyo3::exceptions::PyRuntimeError::new_err(msg));
        }
    }
    Ok(())
}

#[pyfunction]
#[pyo3(signature = (storage_url, local_dir, max_concurrent=None, shallow=false))]
pub fn download_prefix_to_local(
    storage_url: String,
    local_dir: String,
    max_concurrent: Option<usize>,
    shallow: bool,
) -> PyResult<usize> {
    let max_concurrent = max_concurrent.unwrap_or(32);

    let runtime = if is_o2_url(&storage_url) {
        get_o2_runtime()
    } else {
        get_runtime()
    };

    runtime
        .block_on(download_prefix_to_local_async(
            &storage_url,
            &local_dir,
            max_concurrent,
            shallow,
        ))
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("{}", e)))
}

async fn download_prefix_to_local_async(
    storage_url: &str,
    local_dir: &str,
    _max_concurrent: usize,
    shallow: bool,
) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
    let start = std::time::Instant::now();

    let (store, base_path) = get_checkpoint_store(storage_url, None)?;
    let base_prefix = {
        let s = base_path.as_ref().to_string();
        if !s.is_empty() && !s.ends_with('/') {
            format!("{}/", s)
        } else {
            s
        }
    };

    let list_prefix = ObjectPath::from(base_prefix.trim_end_matches('/').to_string());
    let objects = store
        .list(&list_prefix, storage_url)
        .await
        .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { e.into() })?;

    let objects: Vec<_> = if shallow {
        objects
            .into_iter()
            .filter(|meta| {
                let key = meta.location.to_string();
                if let Some(rel) = key.strip_prefix(&base_prefix) {
                    !rel.contains('/')
                } else {
                    true
                }
            })
            .collect()
    } else {
        objects
    };

    let num_files = objects.len();
    let total_bytes: u64 = objects.iter().map(|m| m.size).sum();
    info!(
        "[checkpoint-store] Found {} objects ({:.1} MB) under {} (shallow={})",
        num_files,
        total_bytes as f64 / (1024.0 * 1024.0),
        storage_url,
        shallow,
    );

    let finished = Arc::new(AtomicUsize::new(0));
    let mut handles = Vec::with_capacity(num_files);

    for meta in &objects {
        let rel_path = {
            let key = meta.location.to_string();
            if key.starts_with(&base_prefix) {
                key[base_prefix.len()..].to_string()
            } else {
                key
            }
        };
        let local_path = format!("{}/{}", local_dir, rel_path);

        if let Some(parent) = std::path::Path::new(&local_path).parent() {
            fs::create_dir_all(parent)?;
        }

        let store = store.clone();
        let obj_path = meta.location.clone();
        let finished = finished.clone();
        let total = num_files;
        let label = rel_path.clone();

        handles.push(task::spawn(async move {
            let bytes = store.get(&obj_path, &label).await?;
            tokio::fs::write(&local_path, &bytes)
                .await
                .map_err(|e| format!("write {} failed: {}", local_path, e))?;
            let done = finished.fetch_add(1, Ordering::Relaxed) + 1;
            if done.is_multiple_of(50) || done == total {
                info!("[checkpoint-store] {}/{} files downloaded", done, total);
            }
            Ok::<(), String>(())
        }));
    }

    for handle in handles {
        handle
            .await
            .map_err(|e| format!("join error: {}", e))?
            .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { e.into() })?;
    }

    info!(
        "[checkpoint-store] Done: {} files ({:.1} MB) to {} in {:.1}s",
        num_files,
        total_bytes as f64 / (1024.0 * 1024.0),
        local_dir,
        start.elapsed().as_secs_f64(),
    );
    Ok(num_files)
}

#[pyfunction]
#[pyo3(signature = (storage_url, relative_paths, local_dir, max_concurrent=None))]
pub fn download_files_from_storage(
    storage_url: String,
    relative_paths: Vec<String>,
    local_dir: String,
    max_concurrent: Option<usize>,
) -> PyResult<usize> {
    let max_concurrent = max_concurrent.unwrap_or(32);
    let num_files = relative_paths.len();

    let runtime = if is_o2_url(&storage_url) {
        get_o2_runtime()
    } else {
        get_runtime()
    };

    runtime
        .block_on(download_files_from_storage_async(
            &storage_url,
            &relative_paths,
            &local_dir,
            max_concurrent,
        ))
        .map_err(|e| {
            pyo3::exceptions::PyRuntimeError::new_err(format!(
                "download_files_from_storage failed ({} files from {}): {}",
                num_files, storage_url, e
            ))
        })
}

async fn download_files_from_storage_async(
    storage_url: &str,
    relative_paths: &[String],
    local_dir: &str,
    _max_concurrent: usize,
) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
    let start = std::time::Instant::now();
    let num_files = relative_paths.len();

    let (store, base_path) = get_checkpoint_store(storage_url, None)?;
    let base_prefix = {
        let s = base_path.as_ref().to_string();
        if !s.is_empty() && !s.ends_with('/') {
            format!("{}/", s)
        } else {
            s
        }
    };

    info!(
        "[checkpoint-store] Downloading {} files from {} to {}",
        num_files, storage_url, local_dir,
    );

    let finished = Arc::new(AtomicUsize::new(0));
    let mut handles = Vec::with_capacity(num_files);

    for rel_path in relative_paths {
        let obj_key = format!("{}{}", base_prefix, rel_path);
        let obj_path = ObjectPath::from(obj_key);
        let local_path = format!("{}/{}", local_dir, rel_path);

        if let Some(parent) = std::path::Path::new(&local_path).parent() {
            fs::create_dir_all(parent)?;
        }

        let store = store.clone();
        let finished = finished.clone();
        let total = num_files;
        let rel = rel_path.clone();

        handles.push(task::spawn(async move {
            let bytes = store.get(&obj_path, &rel).await?;
            tokio::fs::write(&local_path, &bytes)
                .await
                .map_err(|e| format!("write {} failed: {}", local_path, e))?;
            let done = finished.fetch_add(1, Ordering::Relaxed) + 1;
            if done.is_multiple_of(10) || done == total {
                info!("[checkpoint-store] {}/{} files downloaded", done, total);
            }
            Ok::<(), String>(())
        }));
    }

    for handle in handles {
        handle
            .await
            .map_err(|e| format!("join error: {}", e))?
            .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { e.into() })?;
    }

    info!(
        "[checkpoint-store] Done: {} files to {} in {:.1}s",
        num_files,
        local_dir,
        start.elapsed().as_secs_f64(),
    );
    Ok(num_files)
}

#[pyfunction]
pub fn list_storage_objects(storage_url: String) -> PyResult<Vec<String>> {
    let runtime = if is_o2_url(&storage_url) {
        get_o2_runtime()
    } else {
        get_runtime()
    };

    runtime
        .block_on(list_storage_objects_async(&storage_url))
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("{}", e)))
}

async fn list_storage_objects_async(
    storage_url: &str,
) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
    let (store, base_path) = get_checkpoint_store(storage_url, None)?;
    let base_prefix = {
        let s = base_path.as_ref().to_string();
        if !s.is_empty() && !s.ends_with('/') {
            format!("{}/", s)
        } else {
            s
        }
    };

    let list_prefix = ObjectPath::from(base_prefix.trim_end_matches('/').to_string());
    let objects = store
        .list(&list_prefix, storage_url)
        .await
        .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { e.into() })?;

    let rels: Vec<String> = objects
        .into_iter()
        .filter_map(|meta| {
            let key = meta.location.to_string();
            key.strip_prefix(&base_prefix).map(|s| s.to_string())
        })
        .collect();

    Ok(rels)
}

#[pyfunction]
#[pyo3(signature = (base_url, name_size_pairs, fixed_prefix=None))]
pub fn find_latest_storage_checkpoint(
    base_url: String,
    name_size_pairs: Vec<(String, u64)>,
    fixed_prefix: Option<String>,
) -> PyResult<Option<(String, usize)>> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("{}", e)))?;
    runtime.block_on(async {
        find_latest_storage_checkpoint_async(&base_url, &name_size_pairs, fixed_prefix.as_deref())
            .await
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("{}", e)))
    })
}

async fn find_latest_storage_checkpoint_async(
    base_url: &str,
    name_size_pairs: &[(String, u64)],
    fixed_prefix: Option<&str>,
) -> Result<Option<(String, usize)>, Box<dyn std::error::Error + Send + Sync>> {
    let (all_objects, base_prefix) = if is_o2_url(base_url) {
        let (store, base_path) = o2_store_from_url(base_url)?;
        let base_prefix = {
            let s = base_path.to_string();
            if !s.is_empty() && !s.ends_with('/') {
                format!("{}/", s)
            } else {
                s
            }
        };

        let search_prefix = if let Some(fp) = fixed_prefix {
            let fp_normalized = fp.trim_end_matches('/');
            format!("{}{}/", base_prefix, fp_normalized)
        } else {
            base_prefix.clone()
        };

        let list_prefix = object_store::path::Path::from(search_prefix.trim_end_matches('/'));
        let objects: Vec<_> = store.list(Some(&list_prefix)).try_collect().await?;
        let all_objects: Vec<(String, u64)> = objects
            .into_iter()
            .map(|meta| (meta.location.to_string(), meta.size))
            .collect();

        (all_objects, base_prefix)
    } else {
        let gcs_path = base_url.strip_prefix("gs://").unwrap_or(base_url);
        let (bucket_name, base_prefix) = match gcs_path.split_once('/') {
            Some((bucket, p)) => (bucket.to_string(), p.to_string()),
            None => (gcs_path.to_string(), String::new()),
        };
        let gcs_bucket = format!("projects/_/buckets/{}", bucket_name);

        let base_prefix = if !base_prefix.is_empty() && !base_prefix.ends_with('/') {
            format!("{}/", base_prefix)
        } else {
            base_prefix
        };

        let search_prefix = if let Some(fp) = fixed_prefix {
            let fp_normalized = fp.trim_end_matches('/');
            format!("{}{}/", base_prefix, fp_normalized)
        } else {
            base_prefix.clone()
        };

        let control = StorageControl::builder().build().await?;
        let mut all_objects: Vec<(String, u64)> = Vec::new();
        let mut page_token: Option<String> = None;

        loop {
            let mut list_request = control
                .list_objects()
                .set_parent(gcs_bucket.clone())
                .set_prefix(&search_prefix);

            if let Some(token) = page_token.take() {
                list_request = list_request.set_page_token(token);
            }

            let response = list_request.send().await?;
            all_objects.extend(
                response
                    .objects
                    .into_iter()
                    .map(|obj| (obj.name, obj.size as u64)),
            );

            if response.next_page_token.is_empty() {
                break;
            } else {
                page_token = Some(response.next_page_token);
            }
        }

        (all_objects, base_prefix)
    };

    let base_prefix_len = base_prefix.len();
    let candidate_paths: HashSet<String> = if let Some(fp) = fixed_prefix {
        let fp_normalized = fp.trim_end_matches('/').to_string();
        info!("Using fixed prefix: {}", fp_normalized);
        std::iter::once(fp_normalized).collect()
    } else {
        let mut paths: HashSet<String> = HashSet::new();
        for (name, _) in &all_objects {
            if name.len() > base_prefix_len {
                let suffix = &name[base_prefix_len..];
                if let Some(first_slash) = suffix.find('/') {
                    let after_first = &suffix[first_slash + 1..];
                    if let Some(second_slash) = after_first.find('/') {
                        let two_level_path = &suffix[..first_slash + 1 + second_slash];
                        paths.insert(two_level_path.to_string());
                    }
                }
            }
        }
        paths
    };

    info!(
        "Found {} candidate paths under {}",
        candidate_paths.len(),
        base_url
    );

    let mut matching_paths: Vec<(String, usize)> = Vec::new();

    for candidate in &candidate_paths {
        let candidate_prefix = format!("{}{}/", base_prefix, candidate);
        let mut all_match = true;
        let mut max_shards: usize = 0;

        for (name, expected_size) in name_size_pairs {
            let sanitized_name = if is_o2_url(base_url) {
                sanitize_path_for_o2(name)
            } else {
                name.clone()
            };
            let full_prefix = format!("{}{}/", candidate_prefix, sanitized_name);

            let matching_objects: Vec<_> = all_objects
                .iter()
                .filter(|(obj_name, _)| obj_name.starts_with(&full_prefix))
                .collect();

            let total_size: u64 = matching_objects.iter().map(|(_, size)| size).sum();

            info!(
                "Checking array '{}' (sanitized: '{}'): found {} objects, total size {} bytes, expected {} bytes",
                name,
                sanitized_name,
                matching_objects.len(),
                total_size,
                expected_size
            );

            if total_size != *expected_size {
                info!(
                    "Size mismatch for '{}': expected {} bytes, got {} bytes (diff: {} bytes)",
                    name,
                    expected_size,
                    total_size,
                    (total_size as i64) - (*expected_size as i64)
                );
                all_match = false;
                break;
            }

            max_shards = max_shards.max(matching_objects.len());
        }

        if all_match {
            matching_paths.push((candidate.clone(), max_shards));
        }
    }

    info!(
        "Found {} matching paths out of {} candidates",
        matching_paths.len(),
        candidate_paths.len()
    );

    let result = matching_paths.into_iter().max_by(|a, b| a.0.cmp(&b.0));

    if let Some((ref path, max_shards)) = result {
        info!(
            "Selected greatest matching path: {} with max_shards: {}",
            path, max_shards
        );
    }

    Ok(result)
}

#[pyfunction]
#[pyo3(signature = (storage_url, name_array_pairs, max_concurrent_downloads=None))]
pub fn load_arrays_from_storage<'py>(
    _py: Python<'py>,
    storage_url: String,
    name_array_pairs: Bound<'py, PyList>,
    max_concurrent_downloads: Option<usize>,
) -> PyResult<()> {
    if name_array_pairs.is_empty() {
        return Ok(());
    }

    let max_concurrent = max_concurrent_downloads.unwrap_or(32);

    let mut arrays: Vec<PyReadwriteArray1<'py, u8>> = Vec::with_capacity(name_array_pairs.len());
    let mut names: Vec<String> = Vec::with_capacity(name_array_pairs.len());

    for item in name_array_pairs.iter() {
        let tuple = item
            .downcast::<PyTuple>()
            .map_err(|_| pyo3::exceptions::PyTypeError::new_err("expected tuple (name, array)"))?;
        if tuple.len() != 2 {
            return Err(pyo3::exceptions::PyTypeError::new_err(
                "expected tuple of length 2 (name, array)",
            ));
        }
        let name: String = tuple.get_item(0)?.extract()?;
        let arr: PyReadwriteArray1<'py, u8> = tuple.get_item(1)?.extract()?;
        names.push(name);
        arrays.push(arr);
    }

    let mut slices: Vec<&'static mut [u8]> = Vec::with_capacity(arrays.len());
    for arr in arrays.iter_mut() {
        let slice: &'static mut [u8] = unsafe { mem::transmute(arr.as_slice_mut()?) };
        slices.push(slice);
    }

    let num_arrays = names.len();
    info!(
        "Loading {} arrays from {} (max_concurrent={})",
        num_arrays, storage_url, max_concurrent
    );

    let load_future = async {
        let semaphore = Arc::new(tokio::sync::Semaphore::new(max_concurrent));
        let futures: Vec<_> = names
            .into_iter()
            .zip(slices)
            .map(|(name, slice)| {
                let url = storage_url.clone();
                let sem = semaphore.clone();
                task::spawn(async move {
                    let _permit = sem.acquire().await.unwrap();
                    load_single_array(&url, &name, slice).await
                })
            })
            .collect();

        let results = join_all(futures).await;
        for result in results {
            result??;
        }
        Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
    };

    let runtime = get_o2_runtime();
    runtime
        .block_on(load_future)
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("{}", e)))?;

    info!("Loaded {} arrays from {}", num_arrays, storage_url);

    Ok(())
}

#[pyfunction]
pub fn remove_old_from_storage(storage_url: String, n: usize, name_size_pairs: Vec<(String, u64)>) {
    info!(
        "Spawning task to remove old checkpoints from {}, keeping {} most recent complete",
        storage_url, n
    );

    let runtime = if is_o2_url(&storage_url) {
        get_o2_runtime()
    } else {
        get_runtime()
    };

    runtime.spawn(async move {
        if let Err(e) = remove_old_from_storage_async(&storage_url, n, &name_size_pairs).await {
            error!(
                "Failed to remove old checkpoints from {}: {:?}",
                storage_url, e
            );
        }
    });
}

async fn remove_old_from_storage_async(
    storage_url: &str,
    n: usize,
    name_size_pairs: &[(String, u64)],
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if name_size_pairs.is_empty() {
        return Err("name_size_pairs is empty: cannot determine checkpoint completeness without tensor specifications".into());
    }

    let (all_objects_with_size, base_prefix, storage_context) = if is_o2_url(storage_url) {
        let (store, base_path) = o2_store_from_url(storage_url)?;
        let base_prefix = {
            let s = base_path.to_string();
            if !s.is_empty() && !s.ends_with('/') {
                format!("{}/", s)
            } else {
                s
            }
        };

        let list_prefix = if base_prefix.is_empty() {
            None
        } else {
            Some(object_store::path::Path::from(
                base_prefix.trim_end_matches('/'),
            ))
        };

        let objects: Vec<_> = store.list(list_prefix.as_ref()).try_collect().await?;
        let all_objects: Vec<(String, u64)> = objects
            .into_iter()
            .map(|meta| (meta.location.to_string(), meta.size))
            .collect();

        (all_objects, base_prefix, StorageContext::O2(store))
    } else {
        let gcs_path = storage_url.strip_prefix("gs://").unwrap_or(storage_url);
        let (bucket_name, base_prefix) = match gcs_path.split_once('/') {
            Some((bucket, p)) => (bucket.to_string(), p.to_string()),
            None => (gcs_path.to_string(), String::new()),
        };
        let gcs_bucket = format!("projects/_/buckets/{}", bucket_name);

        let base_prefix = if !base_prefix.is_empty() && !base_prefix.ends_with('/') {
            format!("{}/", base_prefix)
        } else {
            base_prefix
        };

        let control = StorageControl::builder().build().await?;
        let mut all_objects: Vec<(String, u64)> = Vec::new();
        let mut page_token: Option<String> = None;

        loop {
            let mut list_request = control
                .list_objects()
                .set_parent(gcs_bucket.clone())
                .set_prefix(&base_prefix);

            if let Some(token) = page_token.take() {
                list_request = list_request.set_page_token(token);
            }

            let response = list_request.send().await?;
            all_objects.extend(
                response
                    .objects
                    .into_iter()
                    .map(|obj| (obj.name, obj.size as u64)),
            );

            if response.next_page_token.is_empty() {
                break;
            } else {
                page_token = Some(response.next_page_token);
            }
        }

        (all_objects, base_prefix, StorageContext::Gcs(gcs_bucket))
    };

    let base_prefix_len = base_prefix.len();
    let mut checkpoint_dirs: HashSet<String> = HashSet::new();

    for (name, _) in &all_objects_with_size {
        if name.len() > base_prefix_len {
            let suffix = &name[base_prefix_len..];
            if suffix.starts_with("elapsed_samples_") {
                if let Some(first_slash) = suffix.find('/') {
                    let after_first = &suffix[first_slash + 1..];
                    if let Some(second_slash) = after_first.find('/') {
                        let two_level_path = &suffix[..first_slash + 1 + second_slash];
                        checkpoint_dirs.insert(two_level_path.to_string());
                    }
                }
            }
        }
    }

    let mut complete_dirs: Vec<String> = Vec::new();
    let mut incomplete_dirs: Vec<String> = Vec::new();

    for checkpoint_dir in &checkpoint_dirs {
        let checkpoint_prefix = format!("{}{}/", base_prefix, checkpoint_dir);
        let mut is_complete = true;

        for (name, expected_size) in name_size_pairs {
            let sanitized_name = if is_o2_url(storage_url) {
                sanitize_path_for_o2(name)
            } else {
                name.clone()
            };
            let full_prefix = format!("{}{}/", checkpoint_prefix, sanitized_name);

            let total_size: u64 = all_objects_with_size
                .iter()
                .filter(|(obj_name, _)| obj_name.starts_with(&full_prefix))
                .map(|(_, size)| size)
                .sum();

            if total_size != *expected_size {
                is_complete = false;
                break;
            }
        }

        if is_complete {
            complete_dirs.push(checkpoint_dir.clone());
        } else {
            incomplete_dirs.push(checkpoint_dir.clone());
        }
    }

    complete_dirs.sort();
    incomplete_dirs.sort();

    info!(
        "Found {} complete and {} incomplete checkpoint directories",
        complete_dirs.len(),
        incomplete_dirs.len()
    );

    fn extract_run_id(checkpoint_dir: &str) -> Option<&str> {
        checkpoint_dir.split('/').nth(1)
    }

    let max_run_id: Option<String> = checkpoint_dirs
        .iter()
        .filter_map(|d| extract_run_id(d))
        .max()
        .map(|s| s.to_string());

    info!("Maximal run_id found: {:?}", max_run_id);

    let complete_to_keep: HashSet<String> = if complete_dirs.len() <= n {
        complete_dirs.iter().cloned().collect()
    } else {
        complete_dirs[complete_dirs.len() - n..]
            .iter()
            .cloned()
            .collect()
    };

    let oldest_kept_complete: Option<&String> = if complete_dirs.len() <= n {
        complete_dirs.first()
    } else {
        complete_dirs.get(complete_dirs.len() - n)
    };

    let current_run_incomplete: Vec<&String> = if let Some(ref max_id) = max_run_id {
        incomplete_dirs
            .iter()
            .filter(|dir| extract_run_id(dir) == Some(max_id.as_str()))
            .collect()
    } else {
        incomplete_dirs.iter().collect()
    };

    let incomplete_from_old_runs = incomplete_dirs.len() - current_run_incomplete.len();
    if incomplete_from_old_runs > 0 {
        info!(
            "Discarding {} incomplete checkpoints from old runs (run_id < {:?})",
            incomplete_from_old_runs, max_run_id
        );
    }

    let incomplete_to_keep: HashSet<String> = if let Some(oldest_complete) = oldest_kept_complete {
        let newer_incomplete: Vec<&String> = current_run_incomplete
            .into_iter()
            .filter(|dir| dir.as_str() > oldest_complete.as_str())
            .collect();
        if newer_incomplete.len() <= n {
            newer_incomplete.into_iter().cloned().collect()
        } else {
            newer_incomplete[newer_incomplete.len() - n..]
                .iter()
                .copied()
                .cloned()
                .collect()
        }
    } else {
        if current_run_incomplete.len() <= n {
            current_run_incomplete.into_iter().cloned().collect()
        } else {
            current_run_incomplete[current_run_incomplete.len() - n..]
                .iter()
                .copied()
                .cloned()
                .collect()
        }
    };

    let dirs_to_keep: HashSet<String> = complete_to_keep
        .union(&incomplete_to_keep)
        .cloned()
        .collect();

    let dirs_to_remove: Vec<&String> = checkpoint_dirs
        .iter()
        .filter(|d| !dirs_to_keep.contains(*d))
        .collect();

    if dirs_to_remove.is_empty() {
        info!(
            "Keeping {} complete and {} incomplete checkpoints, nothing to remove",
            complete_to_keep.len(),
            incomplete_to_keep.len()
        );
        return Ok(());
    }

    info!(
        "Keeping {} complete and {} incomplete checkpoints, removing {} directories",
        complete_to_keep.len(),
        incomplete_to_keep.len(),
        dirs_to_remove.len()
    );

    let prefixes_to_remove: HashSet<String> = dirs_to_remove
        .iter()
        .map(|d| format!("{}{}/", base_prefix, d))
        .collect();

    let objects_to_remove: Vec<String> = all_objects_with_size
        .into_iter()
        .filter(|(name, _)| {
            prefixes_to_remove
                .iter()
                .any(|prefix| name.starts_with(prefix))
        })
        .map(|(name, _)| name)
        .collect();

    info!(
        "Removing {} objects from {} directories",
        objects_to_remove.len(),
        dirs_to_remove.len()
    );

    match storage_context {
        StorageContext::O2(store) => {
            let runtime = get_o2_runtime();
            for object_name in objects_to_remove {
                let store_clone = store.clone();
                runtime.spawn(async move {
                    let path = object_store::path::Path::from(object_name.as_str());
                    if let Err(e) = store_clone.delete(&path).await {
                        error!("Failed to delete {}: {:?}", object_name, e);
                    } else {
                        debug!("Deleted {}", object_name);
                    }
                });
            }
        }
        StorageContext::Gcs(gcs_bucket) => {
            let runtime = get_runtime();
            for object_name in objects_to_remove {
                let bucket = gcs_bucket.clone();
                runtime.spawn(async move {
                    if let Err(e) = delete_single_object(&bucket, &object_name).await {
                        error!("Failed to delete {}: {:?}", object_name, e);
                    }
                });
            }
        }
    }

    Ok(())
}

enum StorageContext {
    O2(Arc<dyn ObjectStore>),
    Gcs(String),
}

async fn delete_single_object(
    gcs_bucket: &str,
    object_name: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let control = StorageControl::builder().build().await?;
    control
        .delete_object()
        .set_bucket(gcs_bucket)
        .set_object(object_name)
        .send()
        .await?;
    info!("Deleted {}/{}", gcs_bucket, object_name);
    Ok(())
}

async fn load_single_array(
    storage_url: &str,
    name: &str,
    buf: &mut [u8],
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let bytes_read = if is_o2_url(storage_url) {
        let sanitized_name = sanitize_path_for_o2(name);
        let full_url = format!("{}/{}", storage_url.trim_end_matches('/'), sanitized_name);
        let (store, object_path) = o2_store_from_url(&full_url)?;

        let max_retries = 6;
        let mut last_error: Option<String> = None;

        for attempt in 1..=max_retries {
            let result = match store.get(&object_path).await {
                Ok(r) => r,
                Err(e) => {
                    last_error = Some(format!("GET request failed: {}", e));
                    if attempt < max_retries {
                        let backoff = Duration::from_millis(1000 * (1 << (attempt - 1)));
                        error!(
                            "Failed to GET {} (attempt {}/{}): {}. Retrying in {:?}",
                            name, attempt, max_retries, e, backoff
                        );
                        tokio::time::sleep(backoff).await;
                    }
                    continue;
                }
            };

            let etag = result.meta.e_tag.clone();

            let mut stream = result.into_stream();

            let mut offset = 0;
            let mut crc32c_value: u32 = 0;
            let mut stream_error: Option<String> = None;

            while let Some(chunk_result) = stream.next().await {
                match chunk_result {
                    Ok(chunk) => {
                        if offset + chunk.len() > buf.len() {
                            return Err(format!(
                                "Size mismatch for {}: buffer size is {}, but received at least {} bytes",
                                name,
                                buf.len(),
                                offset + chunk.len()
                            )
                            .into());
                        }
                        crc32c_value = crc32c::crc32c_append(crc32c_value, &chunk);
                        buf[offset..offset + chunk.len()].copy_from_slice(&chunk);
                        offset += chunk.len();
                    }
                    Err(e) => {
                        stream_error = Some(format!("Stream error at offset {}: {}", offset, e));
                        break;
                    }
                }
            }

            if let Some(err) = stream_error {
                last_error = Some(err.clone());
                if attempt < max_retries {
                    let backoff = Duration::from_millis(1000 * (1 << (attempt - 1)));
                    error!(
                        "{} (attempt {}/{}). Retrying entire download in {:?}",
                        err, attempt, max_retries, backoff
                    );
                    tokio::time::sleep(backoff).await;
                }
                continue;
            }

            let crc_ok = if let Some(etag_value) = &etag {
                let etag_clean = etag_value.trim_matches('"');
                if let Some((checksum_hex, _size_str)) = etag_clean.split_once('-') {
                    if checksum_hex.len() == 8 {
                        let computed_crc32c = format!("{:08x}", crc32c_value);
                        if computed_crc32c != checksum_hex {
                            last_error = Some(format!(
                                "CRC32C mismatch for {}: expected {} (from ETag), computed {}",
                                name, checksum_hex, computed_crc32c
                            ));
                            false
                        } else {
                            debug!("CRC32C verified for {} ({})", name, computed_crc32c);
                            true
                        }
                    } else {
                        debug!(
                            "Skipping checksum verification for {} (unrecognized ETag format: {})",
                            name, etag_clean
                        );
                        true
                    }
                } else if etag_clean.len() == 32 {
                    let computed_md5 = format!("{:x}", md5::compute(&buf[..offset]));
                    if computed_md5 != etag_clean {
                        last_error = Some(format!(
                            "MD5 mismatch for {}: expected {} (from ETag), computed {}",
                            name, etag_clean, computed_md5
                        ));
                        false
                    } else {
                        debug!("MD5 verified for {} ({})", name, computed_md5);
                        true
                    }
                } else {
                    debug!(
                        "Skipping checksum verification for {} (unrecognized ETag format: {})",
                        name, etag_clean
                    );
                    true
                }
            } else {
                debug!(
                    "No ETag available for {}, skipping checksum verification",
                    name
                );
                true
            };

            if crc_ok {
                if name != "checksums.0.json" && offset != buf.len() {
                    return Err(format!(
                        "Size mismatch for {}: expected {}, got {}",
                        name,
                        buf.len(),
                        offset
                    )
                    .into());
                }

                if attempt > 1 {
                    info!("Successfully loaded {} after {} attempts", name, attempt);
                }

                debug!("Loaded {} ({} bytes) from {}", name, offset, storage_url);
                return Ok(());
            }

            if attempt < max_retries {
                let backoff = Duration::from_millis(1000 * (1 << (attempt - 1)));
                error!(
                    "Checksum verification failed for {} (attempt {}/{}): {}. Retrying in {:?}",
                    name,
                    attempt,
                    max_retries,
                    last_error.as_ref().unwrap(),
                    backoff
                );
                tokio::time::sleep(backoff).await;
            }
        }

        return Err(format!(
            "Failed to load {} after {} attempts. Last error: {}",
            name,
            max_retries,
            last_error.unwrap_or_else(|| "unknown".to_string())
        )
        .into());
    } else {
        let gcs_path = storage_url.strip_prefix("gs://").unwrap_or(storage_url);
        let (bucket_name, gcs_prefix) = match gcs_path.split_once('/') {
            Some((bucket, prefix)) => (bucket, prefix),
            None => (gcs_path, ""),
        };
        let gcs_bucket = format!("projects/_/buckets/{}", bucket_name);
        let object_name = if gcs_prefix.is_empty() {
            name.to_string()
        } else {
            format!("{}/{}", gcs_prefix.trim_end_matches('/'), name)
        };

        let storage = Storage::builder().build().await?;
        let mut response = storage
            .read_object(&gcs_bucket, &object_name)
            .send()
            .await?;

        let mut offset = 0;
        while let Some(chunk_result) = response.next().await {
            let chunk = chunk_result?;
            if offset + chunk.len() > buf.len() {
                return Err(format!(
                    "Size mismatch for {}: buffer size is {}, but received at least {} bytes",
                    name,
                    buf.len(),
                    offset + chunk.len()
                )
                .into());
            }
            buf[offset..offset + chunk.len()].copy_from_slice(&chunk);
            offset += chunk.len();
        }
        offset
    };

    if name != "checksums.0.json" && bytes_read != buf.len() {
        return Err(format!(
            "Size mismatch for {}: expected {}, got {}",
            name,
            buf.len(),
            bytes_read
        )
        .into());
    }

    debug!(
        "Loaded {} ({} bytes) from {}",
        name, bytes_read, storage_url
    );
    Ok(())
}

#[pyfunction]
pub fn reshard_col_to_row_and_upload(
    col_shard_info: Vec<(String, u64, u64)>,
    output_url: String,
    tensor_name: String,
    num_rows: usize,
    row_size_bytes: usize,
    num_output_shards: usize,
    num_threads: usize,
) -> PyResult<()> {
    use rayon::prelude::*;
    use std::fs::File;
    use std::io::{Read, Seek, SeekFrom};

    info!(
        "Resharding {} column shards to {} row shards for tensor '{}' ({} rows, {} bytes/row)",
        col_shard_info.len(),
        num_output_shards,
        tensor_name,
        num_rows,
        row_size_bytes
    );

    if col_shard_info.is_empty() {
        return Err(pyo3::exceptions::PyValueError::new_err(
            "col_shard_info is empty",
        ));
    }

    if !num_rows.is_multiple_of(num_output_shards) {
        return Err(pyo3::exceptions::PyValueError::new_err(format!(
            "num_rows ({}) must be divisible by num_output_shards ({})",
            num_rows, num_output_shards
        )));
    }

    let num_col_shards = col_shard_info.len();
    if !row_size_bytes.is_multiple_of(num_col_shards) {
        return Err(pyo3::exceptions::PyValueError::new_err(format!(
            "row_size_bytes ({}) must be divisible by num_col_shards ({})",
            row_size_bytes, num_col_shards
        )));
    }

    let col_chunk_bytes = row_size_bytes / num_col_shards;
    let rows_per_output_shard = num_rows / num_output_shards;
    let output_shard_size = rows_per_output_shard * row_size_bytes;

    info!(
        "Column chunk size: {} bytes, rows per output shard: {}, output shard size: {} bytes",
        col_chunk_bytes, rows_per_output_shard, output_shard_size
    );

    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build()
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("{}", e)))?;

    info!(
        "Reading {} column shards from OCDBT files...",
        num_col_shards
    );
    let col_shards: Vec<Vec<u8>> = pool.install(|| {
        col_shard_info
            .par_iter()
            .map(|(path, offset, size)| {
                let mut file =
                    File::open(path).unwrap_or_else(|_| panic!("Failed to open {}", path));
                file.seek(SeekFrom::Start(*offset))
                    .unwrap_or_else(|_| panic!("Failed to seek to offset {} in {}", offset, path));
                let mut data = vec![0u8; *size as usize];
                file.read_exact(&mut data)
                    .unwrap_or_else(|_| panic!("Failed to read {} bytes from {}", size, path));
                data
            })
            .collect()
    });

    let expected_col_shard_size = num_rows * col_chunk_bytes;
    for (i, shard) in col_shards.iter().enumerate() {
        if shard.len() != expected_col_shard_size {
            return Err(pyo3::exceptions::PyValueError::new_err(format!(
                "Column shard {} has size {} but expected {}",
                i,
                shard.len(),
                expected_col_shard_size
            )));
        }
    }

    info!("Building {} output row shards...", num_output_shards);

    let row_shards: Vec<Vec<u8>> = pool.install(|| {
        (0..num_output_shards)
            .into_par_iter()
            .map(|shard_idx| {
                let mut output = vec![0u8; output_shard_size];
                let start_row = shard_idx * rows_per_output_shard;

                for local_row in 0..rows_per_output_shard {
                    let global_row = start_row + local_row;
                    let output_row_start = local_row * row_size_bytes;

                    for (col_idx, col_shard) in col_shards.iter().enumerate() {
                        let col_start = col_idx * col_chunk_bytes;
                        let src_offset = global_row * col_chunk_bytes;
                        output[output_row_start + col_start
                            ..output_row_start + col_start + col_chunk_bytes]
                            .copy_from_slice(&col_shard[src_offset..src_offset + col_chunk_bytes]);
                    }
                }

                output
            })
            .collect()
    });

    info!(
        "Uploading {} row shards to {}...",
        num_output_shards, output_url
    );

    let runtime = get_o2_runtime();
    let upload_results: Vec<Result<(), String>> = runtime.block_on(async {
        let futures: Vec<_> = row_shards
            .into_iter()
            .enumerate()
            .map(|(shard_idx, shard_data)| {
                let url = output_url.clone();
                let name = tensor_name.clone();
                task::spawn(async move {
                    let object_key = format!("{}/c/{}/0", name, shard_idx);
                    let sanitized_key = sanitize_path_for_o2(&object_key);
                    let full_url = format!("{}/{}", url.trim_end_matches('/'), sanitized_key);
                    let (store, object_path) = o2_store_from_url(&full_url)
                        .map_err(|e| format!("Failed to create store for {}: {}", full_url, e))?;

                    let buf = Bytes::from(shard_data);
                    let max_retries = 6;
                    let mut last_error: Option<String> = None;

                    for attempt in 1..=max_retries {
                        match store
                            .put(&object_path, PutPayload::from_bytes(buf.clone()))
                            .await
                        {
                            Ok(_) => {
                                if attempt > 1 {
                                    info!(
                                        "Successfully uploaded shard {} after {} attempts",
                                        shard_idx, attempt
                                    );
                                }
                                debug!("Uploaded row shard {} to {}", shard_idx, full_url);
                                return Ok(());
                            }
                            Err(e) => {
                                last_error = Some(format!("{}", e));
                                if attempt < max_retries {
                                    let backoff = Duration::from_millis(1000 * (1 << (attempt - 1)));
                                    error!(
                                        "Failed to upload shard {} (attempt {}/{}): {}. Retrying in {:?}",
                                        shard_idx, attempt, max_retries, e, backoff
                                    );
                                    tokio::time::sleep(backoff).await;
                                }
                            }
                        }
                    }

                    Err(format!(
                        "Failed to upload shard {} after {} attempts. Last error: {}",
                        shard_idx,
                        max_retries,
                        last_error.unwrap_or_else(|| "unknown".to_string())
                    ))
                })
            })
            .collect();

        join_all(futures)
            .await
            .into_iter()
            .map(|r| r.map_err(|e| format!("Task join error: {}", e)).and_then(|r| r))
            .collect()
    });

    let failures: Vec<_> = upload_results
        .iter()
        .enumerate()
        .filter_map(|(i, r)| r.as_ref().err().map(|e| (i, e.clone())))
        .collect();

    if !failures.is_empty() {
        let error_msg = failures
            .iter()
            .map(|(i, e)| format!("Shard {}: {}", i, e))
            .collect::<Vec<_>>()
            .join("; ");
        return Err(pyo3::exceptions::PyRuntimeError::new_err(format!(
            "Failed to upload {} shards: {}",
            failures.len(),
            error_msg
        )));
    }

    info!(
        "Successfully resharded and uploaded {} row shards to {}",
        num_output_shards, output_url
    );

    Ok(())
}

#[pyfunction]
pub fn copy_chunks_to_storage(
    chunks: Vec<(String, u64, u64, String)>,
    output_url: String,
    num_threads: usize,
) -> PyResult<()> {
    use rayon::prelude::*;
    use std::fs::File;
    use std::io::{Read, Seek, SeekFrom};

    let num_chunks = chunks.len();
    info!(
        "Copying {} chunks to {} using {} threads",
        num_chunks, output_url, num_threads
    );

    if chunks.is_empty() {
        return Ok(());
    }

    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build()
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("{}", e)))?;

    info!("Reading {} chunks from OCDBT files...", num_chunks);
    let chunk_data: Vec<(Vec<u8>, String)> = pool.install(|| {
        chunks
            .par_iter()
            .map(|(path, offset, size, dest_key)| {
                let mut file =
                    File::open(path).unwrap_or_else(|_| panic!("Failed to open {}", path));
                file.seek(SeekFrom::Start(*offset))
                    .unwrap_or_else(|_| panic!("Failed to seek to offset {} in {}", offset, path));
                let mut data = vec![0u8; *size as usize];
                file.read_exact(&mut data)
                    .unwrap_or_else(|_| panic!("Failed to read {} bytes from {}", size, path));
                (data, dest_key.clone())
            })
            .collect()
    });

    info!("Uploading {} chunks to {}...", num_chunks, output_url);

    let runtime = get_o2_runtime();
    let upload_results: Vec<Result<(), String>> = runtime.block_on(async {
        let futures: Vec<_> = chunk_data
            .into_iter()
            .map(|(data, dest_key)| {
                let url = output_url.clone();
                task::spawn(async move {
                    let sanitized_key = sanitize_path_for_o2(&dest_key);
                    let full_url = format!("{}/{}", url.trim_end_matches('/'), sanitized_key);
                    let (store, object_path) = o2_store_from_url(&full_url)
                        .map_err(|e| format!("Failed to create store for {}: {}", full_url, e))?;

                    let buf = Bytes::from(data);
                    let max_retries = 6;
                    let mut last_error: Option<String> = None;

                    for attempt in 1..=max_retries {
                        match store
                            .put(&object_path, PutPayload::from_bytes(buf.clone()))
                            .await
                        {
                            Ok(_) => {
                                debug!("Uploaded chunk to {}", full_url);
                                return Ok(());
                            }
                            Err(e) => {
                                last_error = Some(format!("{}", e));
                                if attempt < max_retries {
                                    let backoff =
                                        Duration::from_millis(1000 * (1 << (attempt - 1)));
                                    error!(
                                        "Failed to upload {} (attempt {}/{}): {}. Retrying in {:?}",
                                        dest_key, attempt, max_retries, e, backoff
                                    );
                                    tokio::time::sleep(backoff).await;
                                }
                            }
                        }
                    }

                    Err(format!(
                        "Failed to upload {} after {} attempts. Last error: {}",
                        dest_key,
                        max_retries,
                        last_error.unwrap_or_else(|| "unknown".to_string())
                    ))
                })
            })
            .collect();

        join_all(futures)
            .await
            .into_iter()
            .map(|r| {
                r.map_err(|e| format!("Task join error: {}", e))
                    .and_then(|r| r)
            })
            .collect()
    });

    let failures: Vec<_> = upload_results
        .iter()
        .enumerate()
        .filter_map(|(i, r)| r.as_ref().err().map(|e| (i, e.clone())))
        .collect();

    if !failures.is_empty() {
        let error_msg = failures
            .iter()
            .take(10)
            .map(|(i, e)| format!("Chunk {}: {}", i, e))
            .collect::<Vec<_>>()
            .join("; ");
        return Err(pyo3::exceptions::PyRuntimeError::new_err(format!(
            "{} chunk uploads failed: {}{}",
            failures.len(),
            error_msg,
            if failures.len() > 10 { "..." } else { "" }
        )));
    }

    info!(
        "Successfully copied {} chunks to {}",
        num_chunks, output_url
    );

    Ok(())
}

#[pyfunction]
pub fn upload_inline_data_to_storage(
    data_entries: Vec<(String, Vec<u8>)>,
    output_url: String,
) -> PyResult<()> {
    let num_entries = data_entries.len();
    info!(
        "Uploading {} inline data entries to {}",
        num_entries, output_url
    );

    if data_entries.is_empty() {
        return Ok(());
    }

    let (store, base_path) = o2_shared_store(&output_url)
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("{}", e)))?;

    let runtime = get_o2_runtime();
    let upload_results: Vec<Result<(), String>> = runtime.block_on(async {
        let sem = Arc::new(tokio::sync::Semaphore::new(MAX_CONCURRENT_UPLOADS));
        let mut handles = Vec::with_capacity(num_entries);
        for (dest_key, data) in data_entries {
            let store = store.clone();
            let sem = sem.clone();
            let base = base_path.clone();
            handles.push(task::spawn(async move {
                let _permit = sem
                    .acquire()
                    .await
                    .map_err(|e| format!("Semaphore error: {}", e))?;
                let sanitized = sanitize_path_for_o2(&dest_key);
                let path = ObjectPath::from(format!("{}/{}", base, sanitized));
                let buf = Bytes::from(data);
                upload_with_retry(&store, &path, buf, &dest_key, 6).await
            }));
        }
        join_all(handles)
            .await
            .into_iter()
            .map(|r| {
                r.map_err(|e| format!("Task join error: {}", e))
                    .and_then(|r| r)
            })
            .collect()
    });

    let failures: Vec<_> = upload_results
        .iter()
        .enumerate()
        .filter_map(|(i, r)| r.as_ref().err().map(|e| (i, e.clone())))
        .collect();

    if !failures.is_empty() {
        let error_msg = failures
            .iter()
            .take(10)
            .map(|(i, e)| format!("Entry {}: {}", i, e))
            .collect::<Vec<_>>()
            .join("; ");
        return Err(pyo3::exceptions::PyRuntimeError::new_err(format!(
            "{} inline data uploads failed: {}{}",
            failures.len(),
            error_msg,
            if failures.len() > 10 { "..." } else { "" }
        )));
    }

    info!(
        "Successfully uploaded {} inline data entries to {}",
        num_entries, output_url
    );

    Ok(())
}

#[pyfunction]
pub fn load_reshard_row_to_col(
    base_path: String,
    shard_keys: Vec<(usize, String)>,
    output_dir: String,
    tensor_name: String,
    num_rows: usize,
    row_size_bytes: usize,
    num_col_shards: usize,
    num_threads: usize,
) -> PyResult<()> {
    use rayon::prelude::*;
    use std::fs;
    use std::io::Write;

    let num_row_shards = shard_keys.len();
    let is_s3 = base_path.starts_with("s3://");

    info!(
        "load_reshard_row_to_col: {} row shards -> {} col shards for '{}' ({} rows, {} bytes/row, source={})",
        num_row_shards,
        num_col_shards,
        tensor_name,
        num_rows,
        row_size_bytes,
        if is_s3 { "s3" } else { "local" }
    );

    if shard_keys.is_empty() {
        return Err(pyo3::exceptions::PyValueError::new_err(
            "shard_keys is empty",
        ));
    }

    let rows_per_shard = num_rows / num_row_shards;
    if !num_rows.is_multiple_of(num_row_shards) {
        return Err(pyo3::exceptions::PyValueError::new_err(format!(
            "num_rows ({}) must be divisible by num_row_shards ({})",
            num_rows, num_row_shards
        )));
    }

    let col_chunk_bytes = row_size_bytes / num_col_shards;
    if !row_size_bytes.is_multiple_of(num_col_shards) {
        return Err(pyo3::exceptions::PyValueError::new_err(format!(
            "row_size_bytes ({}) must be divisible by num_col_shards ({})",
            row_size_bytes, num_col_shards
        )));
    }

    let row_shard_size = rows_per_shard * row_size_bytes;
    let col_shard_size = num_rows * col_chunk_bytes;

    let mut shard_keys = shard_keys;
    shard_keys.sort_by_key(|(idx, _)| *idx);

    let t0 = std::time::Instant::now();

    info!(
        "Allocating {} col shard buffers ({:.2} GB total)...",
        num_col_shards,
        num_col_shards as f64 * col_shard_size as f64 * 1e-9
    );
    let mut col_shards: Vec<Vec<u8>> = (0..num_col_shards)
        .map(|_| vec![0u8; col_shard_size])
        .collect();

    let alloc_secs = t0.elapsed().as_secs_f64();
    info!("Allocated col buffers in {:.2}s", alloc_secs);

    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build()
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("{}", e)))?;

    let t1 = std::time::Instant::now();
    let total_bytes = num_row_shards * row_shard_size;

    info!(
        "Streaming {} row shards (read one, scatter, drop). num_threads={}",
        num_row_shards, num_threads
    );

    for (shard_pos, (_row_idx, rel_key)) in shard_keys.iter().enumerate() {
        let row_shard: Vec<u8> = if is_s3 {
            let runtime = get_o2_runtime();
            let full_url = format!("{}/{}", base_path.trim_end_matches('/'), rel_key);
            runtime
                .block_on(async {
                    let (store, object_path) = o2_store_from_url(&full_url)
                        .map_err(|e| format!("Failed to create store for {}: {}", full_url, e))?;

                    let max_retries = 6;
                    let mut last_error: Option<String> = None;
                    for attempt in 1..=max_retries {
                        match store.get(&object_path).await {
                            Ok(result) => {
                                return Ok(result
                                    .bytes()
                                    .await
                                    .map_err(|e| format!("{}", e))?
                                    .to_vec());
                            }
                            Err(e) => {
                                last_error = Some(format!("{}", e));
                                if attempt < max_retries {
                                    let backoff =
                                        Duration::from_millis(1000 * (1 << (attempt - 1)));
                                    error!(
                                        "Failed to download shard {} (attempt {}/{}): {}. Retrying in {:?}",
                                        shard_pos, attempt, max_retries, e, backoff
                                    );
                                    tokio::time::sleep(backoff).await;
                                }
                            }
                        }
                    }
                    Err(format!(
                        "Failed to download shard {} after {} attempts: {}",
                        shard_pos,
                        max_retries,
                        last_error.unwrap_or_else(|| "unknown".to_string())
                    ))
                })
                .map_err(|e: String| pyo3::exceptions::PyRuntimeError::new_err(e))?
        } else {
            let full_path = format!("{}/{}", base_path.trim_end_matches('/'), rel_key);
            fs::read(&full_path).map_err(|e| {
                pyo3::exceptions::PyRuntimeError::new_err(format!(
                    "Failed to read {}: {}",
                    full_path, e
                ))
            })?
        };

        if row_shard.len() != row_shard_size {
            return Err(pyo3::exceptions::PyValueError::new_err(format!(
                "Row shard {} has size {} but expected {}",
                shard_pos,
                row_shard.len(),
                row_shard_size
            )));
        }

        let dst_row_start = shard_pos * rows_per_shard;

        pool.install(|| {
            col_shards
                .par_chunks_mut(1)
                .enumerate()
                .for_each(|(col_idx, chunk)| {
                    let col_buf = &mut chunk[0];
                    let col_start = col_idx * col_chunk_bytes;

                    for local_row in 0..rows_per_shard {
                        let src_offset = local_row * row_size_bytes + col_start;
                        let dst_offset = (dst_row_start + local_row) * col_chunk_bytes;
                        col_buf[dst_offset..dst_offset + col_chunk_bytes]
                            .copy_from_slice(&row_shard[src_offset..src_offset + col_chunk_bytes]);
                    }
                });
        });

        if (shard_pos + 1) % 32 == 0 || (shard_pos + 1) == num_row_shards {
            let elapsed = t1.elapsed().as_secs_f64();
            let done_bytes = (shard_pos + 1) * row_shard_size;
            info!(
                "  Processed {}/{} row shards ({:.2} GB, {:.1}s, {:.2} GB/s)",
                shard_pos + 1,
                num_row_shards,
                done_bytes as f64 * 1e-9,
                elapsed,
                done_bytes as f64 * 1e-9 / elapsed
            );
        }
    }

    let scatter_secs = t1.elapsed().as_secs_f64();
    info!(
        "Scattered {} row shards ({:.2} GB) in {:.2}s ({:.2} GB/s)",
        num_row_shards,
        total_bytes as f64 * 1e-9,
        scatter_secs,
        total_bytes as f64 * 1e-9 / scatter_secs
    );

    let out_base = format!("{}/{}/c/0", output_dir, tensor_name);
    fs::create_dir_all(&out_base).map_err(|e| {
        pyo3::exceptions::PyRuntimeError::new_err(format!("Failed to create {}: {}", out_base, e))
    })?;

    info!(
        "Writing {} col shard files to {}...",
        num_col_shards, out_base
    );
    let t2 = std::time::Instant::now();

    pool.install(|| {
        col_shards
            .par_iter()
            .enumerate()
            .for_each(|(col_idx, data)| {
                let path = format!("{}/{}", out_base, col_idx);
                let mut file = fs::File::create(&path)
                    .unwrap_or_else(|e| panic!("Failed to create {}: {}", path, e));
                file.write_all(data)
                    .unwrap_or_else(|e| panic!("Failed to write {}: {}", path, e));
            });
    });

    let write_secs = t2.elapsed().as_secs_f64();
    let total_col_bytes: usize = col_shards.iter().map(|d| d.len()).sum();
    info!(
        "Wrote {} col shards ({:.2} GB) in {:.2}s ({:.2} GB/s)",
        num_col_shards,
        total_col_bytes as f64 * 1e-9,
        write_secs,
        total_col_bytes as f64 * 1e-9 / write_secs
    );

    let total_secs = t0.elapsed().as_secs_f64();
    info!(
        "load_reshard_row_to_col complete: {:.2}s total (alloc {:.2}s + scatter {:.2}s + write {:.2}s)",
        total_secs, alloc_secs, scatter_secs, write_secs
    );

    Ok(())
}
