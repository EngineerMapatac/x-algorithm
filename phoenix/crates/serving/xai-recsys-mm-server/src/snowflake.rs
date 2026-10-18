// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 X.AI Corp.
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const TWEPOCH_MS: u64 = 1288834974657;
const TIMESTAMP_SHIFT: u64 = 22;

pub fn snowflake_timestamp_ms(id: u64) -> u64 {
    (id >> TIMESTAMP_SHIFT) + TWEPOCH_MS
}

pub fn snowflake_age(id: u64) -> Option<Duration> {
    let created_ms = snowflake_timestamp_ms(id);
    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    now_ms.checked_sub(created_ms).map(Duration::from_millis)
}
