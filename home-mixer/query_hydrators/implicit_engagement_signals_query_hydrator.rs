use std::sync::Arc;
use std::time::Duration;

use tonic::async_trait;
use xai_candidate_pipeline::query_hydrator::QueryHydrator;
use xai_stats_receiver::global_stats_receiver;
use xai_twistly_thrift::twistly::{
    RecentTweetEngagements, TwistlyEngagementType, UserRecentVideoViewTweets,
    VideoViewEngagementType,
};

use crate::clients::engagement_signals_client::EngagementSignalsClient;
use crate::models::engagement_signals::{
    sort_dedup_truncate, EngagementSignal, EngagementSignalType, EngagementSignalsByType,
};
use crate::models::query::ScoredPostsQuery;
use crate::params::{EnableImplicitEngagementSignals, EngagementSignalsMaxPerType};

const HYDRATE_TIMEOUT: Duration = Duration::from_millis(300);
const SIGNALS_METRIC: &str = "ImplicitEngagementSignals.signals";
const MIN_VIDEO_DURATION_SECS: i64 = 10;

pub struct ImplicitEngagementSignalsQueryHydrator {
    client: Arc<dyn EngagementSignalsClient>,
}

impl ImplicitEngagementSignalsQueryHydrator {
    pub fn new(client: Arc<dyn EngagementSignalsClient>) -> Self {
        Self { client }
    }
}

#[async_trait]
impl QueryHydrator<ScoredPostsQuery> for ImplicitEngagementSignalsQueryHydrator {
    fn enable(&self, query: &ScoredPostsQuery) -> bool {
        query.params.get(EnableImplicitEngagementSignals)
    }

    async fn hydrate(&self, query: &ScoredPostsQuery) -> Result<ScoredPostsQuery, String> {
        let user_id = query.user_id;
        let (photo_expand, vqv, immersive_vqv) = tokio::time::timeout(HYDRATE_TIMEOUT, async {
            tokio::join!(
                self.client
                    .fetch_tweet_engagements(user_id, TwistlyEngagementType::PHOTO_EXPAND),
                self.client
                    .fetch_video_view_tweets(user_id, VideoViewEngagementType::VIDEO_QUALITY_VIEW),
                self.client.fetch_video_view_tweets(
                    user_id,
                    VideoViewEngagementType::IMMERSIVE_VIDEO_QUALITY_VIEW
                ),
            )
        })
        .await
        .map_err(|_| "ImplicitEngagementSignals MH gets timed out".to_string())?;

        let max_per_type = query.params.get(EngagementSignalsMaxPerType);
        let mut signals = EngagementSignalsByType::new();
        insert_tweet_engagements(
            &mut signals,
            EngagementSignalType::PhotoExpand,
            photo_expand?,
            max_per_type,
        );
        insert_video_views(
            &mut signals,
            EngagementSignalType::VideoQualityView,
            vqv?,
            max_per_type,
        );
        insert_video_views(
            &mut signals,
            EngagementSignalType::ImmersiveVideoQualityView,
            immersive_vqv?,
            max_per_type,
        );

        if let Some(receiver) = global_stats_receiver() {
            for (signal_type, signals) in &signals {
                receiver.incr(
                    SIGNALS_METRIC,
                    &[("type", signal_type.as_str())],
                    signals.len() as u64,
                );
            }
        }

        Ok(ScoredPostsQuery {
            implicit_engagement_signals: Some(signals),
            ..Default::default()
        })
    }

    fn update(&self, query: &mut ScoredPostsQuery, hydrated: ScoredPostsQuery) {
        query.implicit_engagement_signals = hydrated.implicit_engagement_signals;
    }
}

fn insert_tweet_engagements(
    signals: &mut EngagementSignalsByType,
    signal_type: EngagementSignalType,
    row: Option<RecentTweetEngagements>,
    max_per_type: usize,
) {
    let Some(row) = row else { return };
    let converted = row
        .engagements
        .into_iter()
        .map(|engagement| EngagementSignal {
            tweet_id: engagement.tweet_id,
            author_id: engagement.author_id,
            engaged_at_ms: engagement.engaged_at,
        })
        .collect();
    signals.insert(signal_type, sort_dedup_truncate(converted, max_per_type));
}

fn insert_video_views(
    signals: &mut EngagementSignalsByType,
    signal_type: EngagementSignalType,
    row: Option<UserRecentVideoViewTweets>,
    max_per_type: usize,
) {
    let Some(row) = row else { return };
    let converted = row
        .recent_engaged_tweets
        .into_iter()
        .filter(|view| {
            !view.is_promoted_tweet && view.video_duration_seconds >= MIN_VIDEO_DURATION_SECS
        })
        .map(|view| EngagementSignal {
            tweet_id: view.tweet_id,
            author_id: view.author_id,
            engaged_at_ms: view.engaged_at,
        })
        .collect();
    signals.insert(signal_type, sort_dedup_truncate(converted, max_per_type));
}
