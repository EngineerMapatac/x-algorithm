use super::*;
use crate::clients::socialgraph_client::EdgeQuery;
use crate::hydration::batch::{HydrationBatch, HydrationError, RawHydrationBatch};
use crate::hydration::community_source::CommunityPost;
use crate::hydration::sources::{
    author_batch, core_batch, country_batch, exchange, label_batch, lifecycle_batch, tweet_batch,
    viewer_batch, Answered, Bytes, DecodedAuthor, DecodedViewer, EdgeKey, Observed, Sources,
};
use crate::models::{ArticleLifecycle, PureCore, TweetFeatures};
use crate::post_card::PostCard;
use prost::Message;
use std::collections::HashMap;
use std::hash::Hash;
use xai_core_entities::entities::ConversationControl;
use xai_visibility_filtering_proto as vf_pb;

pub(crate) struct Replay {
    recording: Recording,
    misses: Mutex<Vec<String>>,
}

fn result<V>(answer: Hydrated<V>) -> Result<Option<V>, HydrationError> {
    match answer {
        Hydrated::Found(value) | Hydrated::Partial(value) => Ok(Some(value)),
        Hydrated::NotFound => Ok(None),
        Hydrated::Failed(error) => Err(error),
    }
}

impl Replay {
    fn answer<X: Exchange>(&self, key: &X::Key) -> Hydrated<X::Wire> {
        let name = X::NAME;
        let recorded = self.recording.answers.get(name);
        let wire = |json: &serde_json::Value| {
            X::Wire::deserialize(json).unwrap_or_else(|e| panic!("{name}/{key}: {e}"))
        };
        match recorded.and_then(|answers| answers.get(&key.to_string())) {
            Some(Answer::Found(json)) => Hydrated::Found(wire(json)),
            Some(Answer::Partial(json)) if X::KEEPS_PARTIAL => Hydrated::Partial(wire(json)),
            Some(Answer::Partial(_)) => {
                panic!("{name}/{key}: a partial answer, which capture never keeps")
            }
            Some(Answer::NotFound) => Hydrated::NotFound,
            Some(Answer::Failed) => Hydrated::Failed(HydrationError::Error),
            None => {
                self.misses.lock().unwrap().push(format!("{name}/{key}"));
                Hydrated::NotFound
            }
        }
    }

    fn batch<X: Exchange>(
        &self,
        keys: impl Iterator<Item = X::Key>,
    ) -> Observed<X, HydrationBatch<X::Key, X::Wire>>
    where
        X::Key: Eq + Hash,
    {
        Observed::quiet(HydrationBatch::from_hydrated(
            keys.map(|key| {
                let answer = self.answer::<X>(&key);
                (key, answer)
            })
            .collect(),
        ))
    }

    fn found<X: Exchange<Key = u64>>(&self, keys: &[u64]) -> HashMap<u64, anyhow::Result<X::Wire>> {
        keys.iter()
            .map(|&key| {
                let wire = self.answer::<X>(&key).into_value();
                (
                    key,
                    wire.ok_or_else(|| anyhow!("{}/{key}: no answer", X::NAME)),
                )
            })
            .collect()
    }

    fn asked<X: Exchange>(&self, fields: &[QueryFields]) {
        let Some(name) = X::FIELDS else {
            return;
        };
        let recorded = self.recording.fields.get(name);
        let mut missing: Vec<&QueryFields> = fields
            .iter()
            .filter(|field| !recorded.is_some_and(|recorded| recorded.contains(field)))
            .collect();
        missing.dedup();
        if !missing.is_empty() {
            self.misses
                .lock()
                .unwrap()
                .push(format!("{name} {missing:?}"));
        }
    }

    pub(crate) fn misses(&self) -> Vec<String> {
        let mut misses = self.misses.lock().unwrap().clone();
        misses.sort();
        misses.dedup();
        misses
    }
}

#[tonic::async_trait]
impl Sources for Replay {
    async fn pure_cores(&self, tweet_ids: &[u64]) -> Answered<exchange::TesPureCoreData, PureCore> {
        self.batch::<exchange::TesPureCoreData>(tweet_ids.iter().copied())
            .map(core_batch)
    }

    async fn conversation_roots(
        &self,
        root_ids: &[u64],
    ) -> Answered<exchange::TesConversationRoot, PureCore> {
        self.batch::<exchange::TesConversationRoot>(root_ids.iter().copied())
            .map(core_batch)
    }

    async fn tweets(&self, tweet_ids: &[u64]) -> Answered<exchange::TesTweet, TweetFeatures> {
        let tweets = self.found::<exchange::TesTweet>(tweet_ids);
        Observed::quiet(tweet_batch(tweet_ids, tweets))
    }

    async fn conversation_controls(
        &self,
        tweet_ids: &[u64],
    ) -> Answered<exchange::TesConversationControl, ConversationControl> {
        self.batch::<exchange::TesConversationControl>(tweet_ids.iter().copied())
    }

    async fn safety_labels(
        &self,
        tweet_ids: &[u64],
    ) -> Answered<exchange::SafetyLabels, Arc<vf_pb::SafetyLabelMap>> {
        let labels = self
            .found::<exchange::SafetyLabels>(tweet_ids)
            .into_iter()
            .map(|(id, bytes)| {
                let labels = bytes.and_then(|Bytes(bytes)| {
                    Ok(Arc::new(vf_pb::SafetyLabelMap::decode(bytes.as_slice())?))
                });
                (id, labels)
            })
            .collect();
        Observed::quiet(label_batch(tweet_ids, labels))
    }

    async fn viewer(
        &self,
        viewer_id: u64,
        fields: &[QueryFields],
    ) -> Answered<exchange::GizmoduckViewer, DecodedViewer> {
        self.asked::<exchange::GizmoduckViewer>(fields);
        let viewer = result(self.answer::<exchange::GizmoduckViewer>(&viewer_id));
        Observed::quiet(viewer_batch(viewer_id, viewer, fields))
    }

    async fn users(
        &self,
        user_ids: &[u64],
        fields: &[QueryFields],
    ) -> Answered<exchange::GizmoduckAuthor, DecodedAuthor> {
        self.asked::<exchange::GizmoduckAuthor>(fields);
        self.batch::<exchange::GizmoduckAuthor>(user_ids.iter().copied())
            .map(author_batch)
    }

    async fn select_edges(
        &self,
        _viewer_id: u64,
        queries: &[EdgeQuery],
    ) -> Observed<exchange::Flock, Vec<RawHydrationBatch<bool>>> {
        let batches = queries.iter().map(|query| {
            let answers = query.destination_ids.iter().map(|&destination| {
                let key = EdgeKey::of(query, destination);
                (destination, self.answer::<exchange::Flock>(&key))
            });
            HydrationBatch::from_hydrated(answers.collect())
        });
        Observed::quiet(batches.collect())
    }

    async fn viewer_country(&self, viewer_id: u64) -> Answered<exchange::ViewerCountry, Arc<str>> {
        let country = result(self.answer::<exchange::ViewerCountry>(&viewer_id));
        Observed::quiet(country_batch(viewer_id, country))
    }

    async fn second_degree(
        &self,
        _viewer_id: u64,
        root_author_ids: &[u64],
    ) -> Answered<exchange::Wingman, bool> {
        self.batch::<exchange::Wingman>(root_author_ids.iter().copied())
    }

    async fn community_moderations(
        &self,
        posts: &[CommunityPost],
    ) -> Answered<exchange::CommunityModeration, CommunityModeration> {
        self.batch::<exchange::CommunityModeration>(posts.iter().map(|post| post.tweet_id))
    }

    async fn community_moderators(
        &self,
        _viewer_id: u64,
        community_ids: &[u64],
    ) -> Answered<exchange::CommunityModerator, bool> {
        self.batch::<exchange::CommunityModerator>(community_ids.iter().copied())
    }

    async fn community_viewer_removals(
        &self,
        _viewer_id: u64,
        community_ids: &[u64],
    ) -> Answered<exchange::CommunityViewerRemoved, bool> {
        self.batch::<exchange::CommunityViewerRemoved>(community_ids.iter().copied())
    }

    async fn article_media_entities(
        &self,
        _viewer_id: Option<u64>,
        article_ids: &[u64],
    ) -> Answered<exchange::ArticleMediaEntities, bool> {
        self.batch::<exchange::ArticleMediaEntities>(article_ids.iter().copied())
    }

    async fn article_cover_media(
        &self,
        _viewer_id: Option<u64>,
        article_ids: &[u64],
    ) -> Answered<exchange::ArticleCoverMedia, bool> {
        self.batch::<exchange::ArticleCoverMedia>(article_ids.iter().copied())
    }

    async fn article_lifecycles(
        &self,
        article_ids: &[u64],
    ) -> Answered<exchange::ArticleLifecycle, ArticleLifecycle> {
        self.batch::<exchange::ArticleLifecycle>(article_ids.iter().copied())
            .map(lifecycle_batch)
    }

    async fn trusted_friends(
        &self,
        _viewer_id: u64,
        list_ids: &[u64],
    ) -> Answered<exchange::TrustedFriends, bool> {
        self.batch::<exchange::TrustedFriends>(list_ids.iter().copied())
    }

    async fn outside_places(
        &self,
        _viewer_id: u64,
        place_ids: &[u64],
    ) -> Answered<exchange::UserLocation, bool> {
        self.batch::<exchange::UserLocation>(place_ids.iter().copied())
    }

    async fn post_card(&self, _tweet_id: u64) -> Option<PostCard> {
        None
    }
}

struct ViewerKeyed<'a> {
    recording: &'a Recording,
    names: Vec<&'static str>,
}

impl Visit for ViewerKeyed<'_> {
    fn exchange<X: Exchange>(&mut self) {
        if X::SCOPE == Scope::Viewer {
            let recording = self.recording;
            self.names.extend(owned_names::<X>().filter(|name| {
                recording.answers.contains_key(*name) || recording.fields.contains_key(*name)
            }));
        }
    }
}

impl Recording {
    fn viewer_keyed(&self) -> Vec<&'static str> {
        visit_all(ViewerKeyed {
            recording: self,
            names: vec![],
        })
        .names
    }

    pub(crate) fn into_replay(self, viewer_id: Option<u64>) -> anyhow::Result<Replay> {
        if viewer_id.is_none() {
            let keyed = self.viewer_keyed();
            anyhow::ensure!(
                keyed.is_empty(),
                "a logged-out recording holds viewer-keyed {keyed:?}"
            );
        }
        Ok(Replay {
            recording: self,
            misses: Mutex::default(),
        })
    }

    pub(crate) fn names(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self
            .answers
            .keys()
            .chain(self.fields.keys())
            .map(String::as_str)
            .collect();
        names.sort_unstable();
        names
    }

    pub(crate) fn with_shared(mut self, shared: &Self) -> Self {
        for (name, answers) in &shared.answers {
            let own = self.answers.entry(name.clone()).or_default();
            for (key, answer) in answers {
                own.entry(key.clone()).or_insert_with(|| answer.clone());
            }
        }
        for (name, fields) in &shared.fields {
            let own = self.fields.entry(name.clone()).or_default();
            for field in fields {
                if !own.contains(field) {
                    own.push(*field);
                }
            }
        }
        self
    }
}
