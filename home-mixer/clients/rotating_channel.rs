use arc_swap::ArcSwap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tonic::transport::Channel;
use tracing::{info, warn};
use xai_x_rpc::grpc_client::ChannelBuilder;

const NUM_SLOTS: usize = 2;

pub struct RotatingChannel {
    endpoint: String,
    slots: [ArcSwap<Channel>; NUM_SLOTS],
    next: AtomicUsize,
}

impl RotatingChannel {
    pub async fn new(
        endpoint: impl Into<String>,
        request_timeout: Duration,
        refresh_interval: Duration,
    ) -> Result<Arc<Self>, tonic::Status> {
        let endpoint = endpoint.into();
        let c1 = Self::build(&endpoint, request_timeout).await?;
        let c2 = Self::build(&endpoint, request_timeout).await?;
        let this = Arc::new(Self {
            endpoint,
            slots: [ArcSwap::from_pointee(c1), ArcSwap::from_pointee(c2)],
            next: AtomicUsize::new(0),
        });

        let refresh = Arc::clone(&this);
        tokio::spawn(async move {
            let start = tokio::time::Instant::now() + refresh_interval;
            let mut interval = tokio::time::interval_at(start, refresh_interval);
            let mut slot = 0usize;
            loop {
                interval.tick().await;
                refresh.refresh(slot % NUM_SLOTS, request_timeout).await;
                slot += 1;
            }
        });

        Ok(this)
    }

    pub fn next(&self) -> Arc<Channel> {
        let idx = self.next.fetch_add(1, Ordering::Relaxed) % NUM_SLOTS;
        self.slots[idx].load_full()
    }

    async fn refresh(&self, slot: usize, request_timeout: Duration) {
        match Self::build(&self.endpoint, request_timeout).await {
            Ok(channel) => {
                self.slots[slot].store(Arc::new(channel));
                info!(endpoint = %self.endpoint, slot, "channel refreshed");
            }
            Err(e) => {
                warn!(endpoint = %self.endpoint, error = %e, "failed to refresh channel");
            }
        }
    }

    async fn build(endpoint: &str, request_timeout: Duration) -> Result<Channel, tonic::Status> {
        ChannelBuilder::new("rotating-channel")
            .destination(endpoint)
            .request_timeout(request_timeout)
            .build()
            .await
            .map_err(|e| {
                tonic::Status::unavailable(format!("failed to connect to {endpoint}: {e:#}"))
            })
    }
}
