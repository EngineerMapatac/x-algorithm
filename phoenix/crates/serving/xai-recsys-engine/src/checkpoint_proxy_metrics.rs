// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 X.AI Corp.
use lazy_static::lazy_static;
use prometheus::{
    Counter, Gauge, Histogram, IntCounter, exponential_buckets, register_counter, register_gauge,
    register_histogram, register_int_counter,
};

lazy_static! {
    pub static ref POLL_TOTAL: IntCounter =
        register_int_counter!("checkpoint_proxy_poll_total", "Total poll attempts").unwrap();
    pub static ref DOWNLOAD_TOTAL: IntCounter = register_int_counter!(
        "checkpoint_proxy_download_total",
        "Successful checkpoint downloads"
    )
    .unwrap();
    pub static ref DOWNLOAD_ERRORS_TOTAL: IntCounter = register_int_counter!(
        "checkpoint_proxy_download_errors_total",
        "Failed download attempts"
    )
    .unwrap();
    pub static ref DOWNLOAD_DURATION_SECONDS: Histogram = register_histogram!(
        "checkpoint_proxy_download_duration_seconds",
        "Time to download a full checkpoint",
        exponential_buckets(1.0, 2.0, 16).unwrap()
    )
    .unwrap();
    pub static ref DOWNLOAD_BYTES_TOTAL: Counter = register_counter!(
        "checkpoint_proxy_download_bytes_total",
        "Total bytes downloaded from trainers"
    )
    .unwrap();
    pub static ref DOWNLOAD_RATE_BYTES_PER_SEC: Histogram = register_histogram!(
        "checkpoint_proxy_download_rate_bytes_per_sec",
        "Download throughput in bytes/s per checkpoint",
        exponential_buckets(1e6, 4.0, 12).unwrap()
    )
    .unwrap();
    pub static ref CHANNEL_DOWNLOAD_DURATION_SECONDS: Histogram = register_histogram!(
        "checkpoint_proxy_channel_download_duration_seconds",
        "Time to download all files from a single trainer channel",
        exponential_buckets(1.0, 2.0, 16).unwrap()
    )
    .unwrap();
    pub static ref CHANNEL_DOWNLOAD_RATE_BYTES_PER_SEC: Histogram = register_histogram!(
        "checkpoint_proxy_channel_download_rate_bytes_per_sec",
        "Per-channel download throughput in bytes/s",
        exponential_buckets(1e6, 4.0, 12).unwrap()
    )
    .unwrap();
    pub static ref LATEST_CHECKPOINT_TIMESTAMP: Gauge = register_gauge!(
        "checkpoint_proxy_latest_checkpoint_timestamp",
        "Unix timestamp when the latest checkpoint was finalized"
    )
    .unwrap();
    pub static ref CHECKPOINTS_AVAILABLE: Gauge = register_gauge!(
        "checkpoint_proxy_checkpoints_available",
        "Number of complete checkpoints in /dev/shm"
    )
    .unwrap();
    pub static ref TRAINER_URL_COUNT: Gauge = register_gauge!(
        "checkpoint_proxy_trainer_url_count",
        "Number of trainer URLs assigned to this server"
    )
    .unwrap();
}
