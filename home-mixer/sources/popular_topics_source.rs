use crate::models::candidate::{PostCandidate, RetrievalSource};
use crate::models::query::ScoredPostsQuery;
use std::sync::Arc;
use tonic::async_trait;
use xai_candidate_pipeline::component_library::clients::StratoClient;
use xai_candidate_pipeline::source::Source;
use xai_home_mixer_proto as pb;
use xai_recsys_proto::grok_topics;

const ALL_TOPICS: &[i64] = &[
    grok_topics::TOPIC_NEWS,
    grok_topics::TOPIC_SPORTS,
    grok_topics::TOPIC_MUSIC,
    grok_topics::TOPIC_CELEBRITY,
    grok_topics::TOPIC_MOVIES_TV,
    grok_topics::TOPIC_TECHNOLOGY,
    grok_topics::TOPIC_BUSINESS_FINANCE,
    grok_topics::TOPIC_CRYPTOCURRENCY,
    grok_topics::TOPIC_GAMING,
    grok_topics::TOPIC_TRAVEL,
    grok_topics::TOPIC_FOOD,
    grok_topics::TOPIC_HEALTH_FITNESS,
    grok_topics::TOPIC_MEMES,
    grok_topics::TOPIC_BEAUTY,
    grok_topics::TOPIC_FASHION,
    grok_topics::TOPIC_NATURE_OUTDOORS,
    grok_topics::TOPIC_PETS,
    grok_topics::TOPIC_CARS,
    grok_topics::TOPIC_HOME_GARDEN,
    grok_topics::TOPIC_SCIENCE,
    grok_topics::TOPIC_RELATIONSHIPS,
    grok_topics::TOPIC_ART,
];

const SMART_TOPICS: &[i64] = &[
    grok_topics::TOPIC_TECHNOLOGY,
    grok_topics::TOPIC_NEWS,
    grok_topics::TOPIC_BUSINESS_FINANCE,
    grok_topics::TOPIC_SCIENCE,
];

fn topic_ids_for_subset(subset: &str) -> &'static [i64] {
    match subset {
        "smart" => SMART_TOPICS,
        _ => ALL_TOPICS,
    }
}

pub struct PopularTopicsSource {
    pub strato_client: Arc<dyn StratoClient + Send + Sync>,
}

#[async_trait]
impl Source<ScoredPostsQuery, PostCandidate> for PopularTopicsSource {
    fn enable(&self, query: &ScoredPostsQuery) -> bool {
        false
    }

    async fn source(&self, query: &ScoredPostsQuery) -> Result<Vec<PostCandidate>, String> {
        let max_per_topic = 5usize;
        let subset: String = "all".to_string();
        let topics = topic_ids_for_subset(&subset);

        let lang = if query.language_code.is_empty() {
            None
        } else {
            Some(query.language_code.clone())
        };

        let futures: Vec<_> = topics
            .iter()
            .map(|&topic_id| {
                let client = Arc::clone(&self.strato_client);
                let lang = lang.clone();
                async move {
                    client
                        .get_popular_topic_tweets(Some(topic_id), lang, None)
                        .await
                        .unwrap_or_default()
                }
            })
            .collect();

        let results = futures::future::join_all(futures).await;

        let candidates: Vec<PostCandidate> = results
            .into_iter()
            .flat_map(|tweet_ids| {
                tweet_ids
                    .into_iter()
                    .take(max_per_topic)
                    .map(|id| PostCandidate {
                        tweet_id: id as u64,
                        served_type: Some(pb::ServedType::ForYouPopularTopics),
                        retrieval_sources: vec![RetrievalSource::from_served_type(
                            pb::ServedType::ForYouPopularTopics,
                        )],
                        ..Default::default()
                    })
            })
            .collect();

        Ok(candidates)
    }
}
