// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 X.AI Corp.
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;

use crate::base_client::BaseO2Client;

pub const BACKEND_UNAVAILABLE: &str =
    "the xai-o2 backend is not included in this build; use the default object_store backend";

#[derive(Debug, Default, Clone)]
pub struct O2ClientBuilder {}

impl O2ClientBuilder {
    pub fn endpoint(&mut self, _value: impl Into<String>) -> &mut Self {
        self
    }

    pub fn bucket(&mut self, _value: impl Into<String>) -> &mut Self {
        self
    }

    pub fn prefix(&mut self, _value: impl Into<String>) -> &mut Self {
        self
    }

    pub fn access_key_id(&mut self, _value: impl Into<String>) -> &mut Self {
        self
    }

    pub fn secret_access_key(&mut self, _value: impl Into<String>) -> &mut Self {
        self
    }

    pub fn allow_anonymous(&mut self, _value: bool) -> &mut Self {
        self
    }

    pub fn timeout(&mut self, _value: Duration) -> &mut Self {
        self
    }

    pub fn io_timeout(&mut self, _value: Duration) -> &mut Self {
        self
    }

    pub fn retry_max_times(&mut self, _value: usize) -> &mut Self {
        self
    }

    pub fn retry_min_delay(&mut self, _value: Duration) -> &mut Self {
        self
    }

    pub fn retry_max_delay(&mut self, _value: Duration) -> &mut Self {
        self
    }

    pub fn write_chunk_size_mib(&mut self, _value: usize) -> &mut Self {
        self
    }

    pub fn write_chunk_concurrency(&mut self, _value: usize) -> &mut Self {
        self
    }

    pub fn max_concurrent_write_requests(&mut self, _value: usize) -> &mut Self {
        self
    }

    pub fn trace_always(&mut self, _value: bool) -> &mut Self {
        self
    }

    pub fn build(&self) -> Result<Arc<dyn BaseO2Client>> {
        Err(anyhow::anyhow!(BACKEND_UNAVAILABLE))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_fails_loudly_after_full_configuration() {
        let mut builder = O2ClientBuilder::default();
        builder
            .endpoint("http://localhost:9000")
            .bucket("some-bucket")
            .prefix("")
            .write_chunk_size_mib(8)
            .write_chunk_concurrency(16)
            .max_concurrent_write_requests(64)
            .retry_max_times(6)
            .retry_min_delay(Duration::from_secs(1))
            .retry_max_delay(Duration::from_secs(32))
            .timeout(Duration::from_secs(300))
            .io_timeout(Duration::from_secs(300))
            .trace_always(false);
        builder.allow_anonymous(true);
        builder.access_key_id("key").secret_access_key("secret");

        let err = builder.build().expect_err("build must fail in this build");
        assert_eq!(err.to_string(), BACKEND_UNAVAILABLE);
    }

    #[test]
    fn build_error_converts_to_boxed_std_error() {
        let err: Box<dyn std::error::Error + Send + Sync> =
            O2ClientBuilder::default().build().unwrap_err().into();
        assert!(err.to_string().contains("object_store"));
    }
}
