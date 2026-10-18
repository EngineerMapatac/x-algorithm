// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 X.AI Corp.
use std::fmt::Debug;

use anyhow::Result;
use async_trait::async_trait;
use bytes::Bytes;

#[async_trait]
pub trait BaseO2Client: Debug + Send + Sync {
    async fn put(&self, key: &str, data: Bytes) -> Result<()>;
}
