// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 X.AI Corp.
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Clone, Debug, Default)]
pub struct LoadGauge {
    load: Arc<AtomicUsize>,
}

impl LoadGauge {
    pub fn try_acquire(
        &self,
        _ticket_id: Option<u64>,
        load: usize,
        _priority: Option<i64>,
    ) -> Option<LoadGuard> {
        self.load.fetch_add(load, Ordering::Relaxed);
        Some(LoadGuard {
            load: self.load.clone(),
            amount: load,
        })
    }

    pub fn get_load(&self) -> usize {
        self.load.load(Ordering::Relaxed)
    }
}

#[derive(Debug)]
pub struct LoadGuard {
    load: Arc<AtomicUsize>,
    amount: usize,
}

impl Drop for LoadGuard {
    fn drop(&mut self) {
        self.load.fetch_sub(self.amount, Ordering::Relaxed);
    }
}
