use std::time::Duration;

use tonic::async_trait;
use xai_manhattan::{s2s::S2sConfig, NativeManhattanClient, Tenant};
use xai_stats_receiver::global_stats_receiver;
use xai_twistly_thrift::twistly::{
    RecentTweetEngagements, TwistlyEngagementType, UserRecentEngagedTweets,
    UserRecentVideoViewTweets, VideoViewEngagementType,
};

const MH_TIMEOUT: Duration = Duration::from_millis(300);
const DECODE_FAILURE_METRIC: &str = "EngagementSignalsClient.decode_failures";

fn engaged_tweets_tenant() -> Tenant {
    Tenant {
        cluster: "omega".to_string(),
        app_id: "twistly_prod".to_string(),
        dataset: "twistly_user_recent_engaged_tweets_prod".to_string(),
    }
}

fn tweet_engagements_tenant() -> Tenant {
    Tenant {
        cluster: "omega".to_string(),
        app_id: "twistly_prod".to_string(),
        dataset: "twistly_recent_tweet_engagements".to_string(),
    }
}

fn video_view_tweets_tenant() -> Tenant {
    Tenant {
        cluster: "nash".to_string(),
        app_id: "uss_prod".to_string(),
        dataset: "recent_video_views".to_string(),
    }
}

fn long_to_native_bytes(value: i64) -> Vec<u8> {
    let mut bytes = value.to_be_bytes();
    bytes[0] ^= 0x80;
    bytes.to_vec()
}

#[async_trait]
pub trait EngagementSignalsClient: Send + Sync {
    async fn fetch_engaged_tweets(
        &self,
        user_id: u64,
    ) -> Result<Option<UserRecentEngagedTweets>, String>;

    async fn fetch_tweet_engagements(
        &self,
        user_id: u64,
        engagement_type: TwistlyEngagementType,
    ) -> Result<Option<RecentTweetEngagements>, String>;

    async fn fetch_video_view_tweets(
        &self,
        user_id: u64,
        engagement_type: VideoViewEngagementType,
    ) -> Result<Option<UserRecentVideoViewTweets>, String>;
}

pub struct ProdEngagementSignalsClient {
    omega: NativeManhattanClient,
    nash: NativeManhattanClient,
}

impl ProdEngagementSignalsClient {
    pub async fn new(datacenter: &str, s2s: S2sConfig) -> Result<Self, String> {
        let omega = NativeManhattanClient::builder_from_tenant_s2s(
            &engaged_tweets_tenant(),
            datacenter,
            s2s.clone(),
        )
        .timeout(MH_TIMEOUT)
        .retries(1)
        .multiplexed()
        .no_batch()
        .build()
        .await
        .map_err(|e| e.to_string())?;

        let nash = NativeManhattanClient::builder_from_tenant_s2s(
            &video_view_tweets_tenant(),
            datacenter,
            s2s,
        )
        .timeout(MH_TIMEOUT)
        .retries(1)
        .multiplexed()
        .no_batch()
        .build()
        .await
        .map_err(|e| e.to_string())?;

        Ok(Self { omega, nash })
    }

    async fn get_and_decode<T>(
        client: &NativeManhattanClient,
        tenant: Tenant,
        pkey: Vec<Vec<u8>>,
        lkey: Vec<Vec<u8>>,
    ) -> Result<Option<T>, String>
    where
        T: xai_twistly_thrift::TSerializable + Send + 'static,
    {
        let dataset = tenant.dataset.clone();
        let item = client
            .get(tenant, pkey, lkey)
            .await
            .map_err(|e| e.to_string())?;

        match item {
            Some(item) => {
                let bytes = item.value().as_bytes().to_vec();
                tokio::task::spawn_blocking(move || {
                    match xai_twistly_thrift::deserialize_binary::<T>(&bytes) {
                        Ok(value) => Some(value),
                        Err(e) => {
                            tracing::warn!("Failed to decode {dataset} value: {e}");
                            if let Some(receiver) = global_stats_receiver() {
                                receiver.incr(
                                    DECODE_FAILURE_METRIC,
                                    &[("dataset", dataset.as_str())],
                                    1,
                                );
                            }
                            None
                        }
                    }
                })
                .await
                .map_err(|e| e.to_string())
            }
            None => Ok(None),
        }
    }
}

#[async_trait]
impl EngagementSignalsClient for ProdEngagementSignalsClient {
    async fn fetch_engaged_tweets(
        &self,
        user_id: u64,
    ) -> Result<Option<UserRecentEngagedTweets>, String> {
        Self::get_and_decode(
            &self.omega,
            engaged_tweets_tenant(),
            vec![long_to_native_bytes(user_id as i64)],
            vec![long_to_native_bytes(0)],
        )
        .await
    }

    async fn fetch_tweet_engagements(
        &self,
        user_id: u64,
        engagement_type: TwistlyEngagementType,
    ) -> Result<Option<RecentTweetEngagements>, String> {
        Self::get_and_decode(
            &self.omega,
            tweet_engagements_tenant(),
            vec![(user_id as i64).to_be_bytes().to_vec()],
            vec![engagement_type.0.to_be_bytes().to_vec()],
        )
        .await
    }

    async fn fetch_video_view_tweets(
        &self,
        user_id: u64,
        engagement_type: VideoViewEngagementType,
    ) -> Result<Option<UserRecentVideoViewTweets>, String> {
        Self::get_and_decode(
            &self.nash,
            video_view_tweets_tenant(),
            vec![(user_id as i64).to_be_bytes().to_vec()],
            vec![engagement_type.0.to_be_bytes().to_vec()],
        )
        .await
    }
}

pub struct MockEngagementSignalsClient;

#[async_trait]
impl EngagementSignalsClient for MockEngagementSignalsClient {
    async fn fetch_engaged_tweets(
        &self,
        _user_id: u64,
    ) -> Result<Option<UserRecentEngagedTweets>, String> {
        Ok(None)
    }

    async fn fetch_tweet_engagements(
        &self,
        _user_id: u64,
        _engagement_type: TwistlyEngagementType,
    ) -> Result<Option<RecentTweetEngagements>, String> {
        Ok(None)
    }

    async fn fetch_video_view_tweets(
        &self,
        _user_id: u64,
        _engagement_type: VideoViewEngagementType,
    ) -> Result<Option<UserRecentVideoViewTweets>, String> {
        Ok(None)
    }
}
