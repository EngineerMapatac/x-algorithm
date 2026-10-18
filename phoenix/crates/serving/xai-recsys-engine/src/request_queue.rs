// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 X.AI Corp.
use crate::request_metrics::WAITING_QUEUE_SIZE;
use futures::future::BoxFuture;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, Notify, mpsc};

const BATCH_WAIT_POLL: Duration = Duration::from_millis(1);

pub trait QueuedItem {
    fn enqueue_time(&self) -> Instant;

    fn deadline(&self) -> Option<Instant>;

    fn on_stale_drop(self, max_staleness: Duration);
}

fn is_expired<T: QueuedItem>(item: &T, max_staleness: Duration, now: Instant) -> bool {
    match item.deadline() {
        Some(deadline) => now > deadline,
        None => {
            max_staleness != Duration::MAX
                && now.saturating_duration_since(item.enqueue_time()) > max_staleness
        }
    }
}

pub struct RequestQueue<T> {
    request_buffer: Arc<Mutex<Vec<T>>>,
    space_available_notify: Arc<Notify>,
    batch_size: usize,
    max_staleness: Duration,
    batch_wait: Option<Duration>,
}

impl<T: QueuedItem + Send + 'static> RequestQueue<T> {
    pub fn new(
        batch_size: usize,
        channel_size: usize,
        max_staleness: Duration,
    ) -> (Self, mpsc::Sender<T>, BoxFuture<'static, ()>) {
        Self::with_batch_wait(batch_size, channel_size, max_staleness, None)
    }

    pub fn with_batch_wait(
        batch_size: usize,
        channel_size: usize,
        max_staleness: Duration,
        batch_wait: Option<Duration>,
    ) -> (Self, mpsc::Sender<T>, BoxFuture<'static, ()>) {
        let (tx, mut rx) = mpsc::channel::<T>(channel_size);
        let request_buffer = Arc::new(Mutex::new(Vec::with_capacity(batch_size)));
        let space_available_notify = Arc::new(Notify::new());

        let buffer_for_task = Arc::clone(&request_buffer);
        let space_available_for_task = Arc::clone(&space_available_notify);
        let max_staleness_for_task = max_staleness;

        let background_task = async move {
            loop {
                let first_item = match rx.recv().await {
                    Some(item) => item,
                    None => break,
                };

                if is_expired(&first_item, max_staleness_for_task, Instant::now()) {
                    first_item.on_stale_drop(max_staleness_for_task);
                    continue;
                }

                let mut buffer = buffer_for_task.lock().await;

                while buffer.len() == buffer.capacity() {
                    drop(buffer);
                    space_available_for_task.notified().await;
                    buffer = buffer_for_task.lock().await;
                }

                buffer.push(first_item);

                while buffer.len() < buffer.capacity() {
                    match rx.try_recv() {
                        Ok(item) => {
                            if is_expired(&item, max_staleness_for_task, Instant::now()) {
                                item.on_stale_drop(max_staleness_for_task);
                                continue;
                            }
                            buffer.push(item);
                        }
                        Err(_) => break,
                    }
                }

                WAITING_QUEUE_SIZE.set(buffer.len() as i64);
            }
        };

        let queue = Self {
            request_buffer,
            space_available_notify,
            batch_size,
            max_staleness,
            batch_wait,
        };

        (queue, tx, Box::pin(background_task))
    }

    pub async fn dequeue(&mut self) -> Vec<T> {
        let mut items = Vec::with_capacity(self.batch_size);
        let mut buffer = self.request_buffer.lock().await;
        if !buffer.is_empty() {
            std::mem::swap(&mut items, &mut buffer);
            WAITING_QUEUE_SIZE.set(buffer.len() as i64);
            self.space_available_notify.notify_one();
        }
        drop(buffer);
        if let Some(wait) = self.batch_wait {
            self.wait_for_batch(&mut items, wait).await;
        }

        if items.is_empty() {
            return items;
        }
        let now = Instant::now();
        let mut fresh = Vec::with_capacity(items.len());
        for item in items {
            if is_expired(&item, self.max_staleness, now) {
                item.on_stale_drop(self.max_staleness);
            } else {
                fresh.push(item);
            }
        }
        fresh
    }

    async fn wait_for_batch(&self, items: &mut Vec<T>, wait: Duration) {
        let deadline = Instant::now() + wait;
        let target = if items.is_empty() { 1 } else { self.batch_size };
        while items.len() < target {
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                break;
            }
            tokio::time::sleep(remaining.min(BATCH_WAIT_POLL)).await;
            let mut buffer = self.request_buffer.lock().await;
            if !buffer.is_empty() {
                let k = (self.batch_size - items.len()).min(buffer.len());
                items.extend(buffer.drain(..k));
                WAITING_QUEUE_SIZE.set(buffer.len() as i64);
                self.space_available_notify.notify_one();
            }
        }
    }

    pub fn reset_dequeue_timer(&mut self) {}

    pub fn is_empty(&self) -> bool {
        true
    }

    pub fn len(&self) -> usize {
        0
    }
}
