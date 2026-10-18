use std::sync::Arc;
use std::time::Duration;

use tonic::async_trait;
use xai_candidate_pipeline::query_hydrator::QueryHydrator;
use xai_stats_receiver::global_stats_receiver;
use xai_twistly_thrift::twistly::{EngagementMetadata, UserRecentEngagedTweets};

use crate::clients::engagement_signals_client::EngagementSignalsClient;
use crate::models::engagement_signals::{
    sort_dedup_truncate, EngagementSignal, EngagementSignalType, EngagementSignalsByType,
};
use crate::models::query::ScoredPostsQuery;
use crate::params::{EnableExplicitEngagementSignals, EngagementSignalsMaxPerType};

const HYDRATE_TIMEOUT: Duration = Duration::from_millis(300);
const SIGNALS_METRIC: &str = "ExplicitEngagementSignals.signals";

pub struct ExplicitEngagementSignalsQueryHydrator {
    client: Arc<dyn EngagementSignalsClient>,
}

impl ExplicitEngagementSignalsQueryHydrator {
    pub fn new(client: Arc<dyn EngagementSignalsClient>) -> Self {
        Self { client }
    }
}

#[async_trait]
impl QueryHydrator<ScoredPostsQuery> for ExplicitEngagementSignalsQueryHydrator {
    fn enable(&self, query: &ScoredPostsQuery) -> bool {
        query.params.get(EnableExplicitEngagementSignals)
    }

    async fn hydrate(&self, query: &ScoredPostsQuery) -> Result<ScoredPostsQuery, String> {
        let row = tokio::time::timeout(
            HYDRATE_TIMEOUT,
            self.client.fetch_engaged_tweets(query.user_id),
        )
        .await
        .map_err(|_| "ExplicitEngagementSignals MH get timed out".to_string())??;

        let max_per_type = query.params.get(EngagementSignalsMaxPerType);
        let row = row.unwrap_or_else(|| UserRecentEngagedTweets::new(vec![], None));
        let signals = split_by_type(row, query.user_id as i64, max_per_type);

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
            explicit_engagement_signals: Some(signals),
            ..Default::default()
        })
    }

    fn update(&self, query: &mut ScoredPostsQuery, hydrated: ScoredPostsQuery) {
        query.explicit_engagement_signals = hydrated.explicit_engagement_signals;
    }
}

fn split_by_type(
    row: UserRecentEngagedTweets,
    user_id: i64,
    max_per_type: usize,
) -> EngagementSignalsByType {
    let mut by_type = EngagementSignalsByType::new();

    for tweet in row.recent_engaged_tweets {
        let (signal_type, author_id) = match &tweet.engagement_metadata {
            EngagementMetadata::FavoriteMetadata(m) => {
                (EngagementSignalType::Favorite, Some(m.tweet_user_id))
            }
            EngagementMetadata::RetweetMetadata(m) => {
                (EngagementSignalType::Retweet, Some(m.source_tweet_user_id))
            }
            EngagementMetadata::ReplyTweetMetadata(m) => {
                (EngagementSignalType::Reply, Some(m.source_tweet_user_id))
            }
            EngagementMetadata::BookmarkMetadata(m) => {
                (EngagementSignalType::Bookmark, Some(m.tweet_user_id))
            }
            EngagementMetadata::TweetShareMetadata(m) => {
                (EngagementSignalType::Share, Some(m.author_id))
            }
            EngagementMetadata::OriginalTweetMetadata(_) => {
                (EngagementSignalType::OriginalTweet, Some(user_id))
            }
            _ => continue,
        };

        by_type
            .entry(signal_type)
            .or_default()
            .push(EngagementSignal {
                tweet_id: tweet.tweet_id,
                author_id,
                engaged_at_ms: tweet.engaged_at,
            });
    }

    by_type
        .into_iter()
        .map(|(signal_type, signals)| (signal_type, sort_dedup_truncate(signals, max_per_type)))
        .collect()
}
