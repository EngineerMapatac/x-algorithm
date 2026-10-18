use crate::hydration::batch::Hydrated;
use crate::hydration::inventory::Source;
use crate::hydration::sources::{Exchange, Id, Observer, Scope, Visit};
use crate::models::CommunityModeration;
use anyhow::{Context, anyhow};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::{BTreeMap, BTreeSet};
use std::iter;
use std::sync::{Arc, Mutex, PoisonError};
use strum::VariantArray;
use xai_core_entities::gizmoduck_client::QueryFields;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Answer<V> {
    Found(V),
    Partial(V),
    NotFound,
    Failed,
}

#[derive(Serialize, Deserialize)]
#[serde(remote = "CommunityModeration")]
struct CommunityModerationJson {
    is_hidden: bool,
    is_author_removed: bool,
}

impl Serialize for CommunityModeration {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        CommunityModerationJson::serialize(self, serializer)
    }
}

impl<'de> Deserialize<'de> for CommunityModeration {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        CommunityModerationJson::deserialize(deserializer)
    }
}

type Answers = BTreeMap<String, BTreeMap<String, Answer<serde_json::Value>>>;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(from = "BTreeMap<String, Kept>", into = "BTreeMap<String, Kept>")]
pub(crate) struct Recording {
    answers: Answers,
    fields: BTreeMap<String, Vec<QueryFields>>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(untagged)]
enum Kept {
    Answers(BTreeMap<String, Answer<serde_json::Value>>),
    Fields(Vec<QueryFields>),
}

impl From<BTreeMap<String, Kept>> for Recording {
    fn from(kept: BTreeMap<String, Kept>) -> Self {
        let mut recording = Self::default();
        for (name, kept) in kept {
            match kept {
                Kept::Answers(answers) => {
                    recording.answers.insert(name, answers);
                }
                Kept::Fields(fields) => {
                    recording.fields.insert(name, fields);
                }
            }
        }
        recording
    }
}

impl From<Recording> for BTreeMap<String, Kept> {
    fn from(recording: Recording) -> Self {
        let answers = recording
            .answers
            .into_iter()
            .map(|(name, answers)| (name, Kept::Answers(answers)));
        let fields = recording
            .fields
            .into_iter()
            .map(|(name, fields)| (name, Kept::Fields(fields)));
        answers.chain(fields).collect()
    }
}

fn visit_all<V: Visit>(mut visitor: V) -> V {
    for &source in Source::VARIANTS {
        source.visit(&mut visitor);
    }
    visitor
}

fn owned_names<X: Exchange>() -> impl Iterator<Item = &'static str> {
    iter::once(X::NAME).chain(X::FIELDS)
}

fn parse<X: Exchange>(
    key: &str,
    answer: &Answer<serde_json::Value>,
) -> anyhow::Result<(X::Key, Option<X::Wire>)> {
    let parsed = key
        .parse()
        .map_err(|_| anyhow!("{}/{key}: not a key", X::NAME))?;
    let wire = match answer {
        Answer::Found(json) | Answer::Partial(json) => {
            Some(X::Wire::deserialize(json).with_context(|| format!("{}/{key}", X::NAME))?)
        }
        Answer::NotFound | Answer::Failed => None,
    };
    Ok((parsed, wire))
}

struct Named<'a> {
    recording: &'a Recording,
    owned: BTreeSet<&'static str>,
    ids: Vec<anyhow::Result<Vec<Id>>>,
}

impl Visit for Named<'_> {
    fn exchange<X: Exchange>(&mut self) {
        self.owned.extend(owned_names::<X>());
        let answers = self.recording.answers.get(X::NAME).into_iter().flatten();
        self.ids.extend(answers.map(|(key, answer)| {
            let (parsed, wire) = parse::<X>(key, answer)?;
            X::ids(&parsed, wire.as_ref()).with_context(|| format!("{}/{key}", X::NAME))
        }));
    }
}

struct Unsuccessful<'a> {
    answers: &'a Answers,
    keys: Vec<String>,
}

impl Visit for Unsuccessful<'_> {
    fn exchange<X: Exchange>(&mut self) {
        let answers = self.answers.get(X::NAME).into_iter().flatten();
        self.keys.extend(
            answers
                .filter(|(_, answer)| match answer {
                    Answer::Failed => true,
                    Answer::Partial(_) => !X::KEEPS_PARTIAL,
                    Answer::Found(_) | Answer::NotFound => false,
                })
                .map(|(key, _)| format!("{}/{key}", X::NAME)),
        );
    }
}

struct Split {
    case: Recording,
    shared: Recording,
}

impl Visit for Split {
    fn exchange<X: Exchange>(&mut self) {
        if X::SCOPE != Scope::Shared {
            return;
        }
        if let Some(answers) = self.case.answers.remove(X::NAME) {
            self.shared.answers.insert(X::NAME.to_owned(), answers);
        }
        if let Some(name) = X::FIELDS
            && let Some(fields) = self.case.fields.remove(name)
        {
            self.shared.fields.insert(name.to_owned(), fields);
        }
    }
}

impl Recording {
            pub(crate) fn ids(&self) -> anyhow::Result<Vec<Id>> {
        let named = visit_all(Named {
            recording: self,
            owned: BTreeSet::new(),
            ids: vec![],
        });
        let recorded = self.answers.keys().chain(self.fields.keys());
        if let Some(stray) = recorded
            .clone()
            .find(|name| !named.owned.contains(name.as_str()))
        {
            anyhow::bail!("{stray}: no Exchange records by this name");
        }
        Ok(named
            .ids
            .into_iter()
            .collect::<anyhow::Result<Vec<_>>>()?
            .concat())
    }

            pub(crate) fn unsuccessful(&self) -> Vec<String> {
        let mut unsuccessful = visit_all(Unsuccessful {
            answers: &self.answers,
            keys: vec![],
        })
        .keys;
        unsuccessful.sort();
        unsuccessful
    }

            pub(crate) fn split_shared(self) -> (Self, Self) {
        let split = visit_all(Split {
            case: self,
            shared: Self::default(),
        });
        (split.case, split.shared)
    }
}

fn recorded<W: Serialize>(answer: Hydrated<W>) -> Answer<serde_json::Value> {
    match answer {
        Hydrated::Found(wire) => serde_json::to_value(wire).map_or(Answer::Failed, Answer::Found),
        Hydrated::Partial(wire) => {
            serde_json::to_value(wire).map_or(Answer::Failed, Answer::Partial)
        }
        Hydrated::NotFound => Answer::NotFound,
        Hydrated::Failed(_) => Answer::Failed,
    }
}

#[derive(Default)]
pub(crate) struct Recorder(Mutex<Recording>);

impl Recorder {
    pub(crate) fn take(&self) -> Recording {
        std::mem::take(&mut self.lock())
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Recording> {
        self.0.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

impl Observer for Arc<Recorder> {
    fn answered<X: Exchange>(&self, answers: impl Iterator<Item = (X::Key, Hydrated<X::Wire>)>) {
        let answers: Vec<_> = answers
            .map(|(key, answer)| (key.to_string(), recorded(answer)))
            .collect();
        if !answers.is_empty() {
            let mut recording = self.lock();
            let recorded = recording.answers.entry(X::NAME.to_owned()).or_default();
            recorded.extend(answers);
        }
    }

    fn asked<X: Exchange>(&self, fields: &[QueryFields]) {
        let Some(name) = X::FIELDS else {
            return;
        };
        if fields.is_empty() {
            return;
        }
        let mut recording = self.lock();
        let recorded = recording.fields.entry(name.to_owned()).or_default();
        for field in fields {
            if !recorded.contains(field) {
                recorded.push(*field);
            }
        }
    }
}

#[cfg(test)]
pub(crate) use replay::Replay;

#[cfg(test)]
mod replay;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clients::socialgraph_client::{EdgeDirection, EdgeQuery, Graph};
    use crate::hydration::batch::{HydrationError, RawHydrationBatch};
    use crate::hydration::sources::{Bytes, EdgeKey, InMemorySources, Sources, exchange};
    use crate::rules::SafetyLevel;
    use crate::scenario::{Scenario, raw};
    use std::collections::HashMap;
    use std::fmt;
    use xai_core_entities::entities::{
        ConversationControl, GizmoduckUser, GizmoduckUserResult, PureCoreData,
    };
    use xai_visibility_filtering_proto as vf_pb;

    fn failed<V>() -> Hydrated<V> {
        Hydrated::Failed(HydrationError::Error)
    }

    #[tokio::test]
    async fn a_call_the_recording_lacks_is_named() {
        let recorder = Arc::new(Recorder::default());
        recorder.answered::<exchange::TesPureCoreData>(iter::once((
            1,
            Hydrated::Found(PureCoreData {
                author_id: 10,
                ..Default::default()
            }),
        )));
        let replay = recorder.take().into_replay(Some(50)).unwrap();
        let scenario = Scenario::new(replay, SafetyLevel::TimelineHomeHydration).viewer(Some(50));
        scenario.evaluate(&[raw(1, None)]).await;

        assert!(
            scenario
                .sources()
                .misses()
                .contains(&"tweets/1".to_string())
        );
    }

    #[test]
    fn every_answer_a_fallback_cache_could_replace_is_unsuccessful() {
        let recorder = Arc::new(Recorder::default());
        recorder.answered::<exchange::TesPureCoreData>(
            [(1, failed()), (2, Hydrated::NotFound)].into_iter(),
        );
        recorder.answered::<exchange::TesTweet>(iter::once((1, failed())));
        recorder.answered::<exchange::GizmoduckAuthor>(iter::once((10, failed())));
        recorder.answered::<exchange::GizmoduckViewer>(iter::once((
            50,
            Hydrated::Partial(GizmoduckUser::default()),
        )));
        let flock_left_out = EdgeKey {
            graph: Graph::Follows,
            direction: EdgeDirection::Forward,
            destination: 30,
        };
        recorder
            .answered::<exchange::Flock>(iter::once((flock_left_out, Hydrated::Partial(false))));

        assert_eq!(
            recorder.take().unsuccessful(),
            ["pure_cores/1", "tweets/1", "users/10", "viewers/50"]
        );
    }

    mod completeness {
        use super::*;
        use crate::clients::about_this_account_client::AboutThisAccountClient;
        use crate::clients::article_client::ArticleClient;
        use crate::clients::gizmoduck_client::GizmoduckLookup;
        use crate::clients::socialgraph_client::SocialgraphClient;
        use crate::clients::strato_answer::{Thrift, found, no_row};
        use crate::clients::trusted_friends_client::{self, TrustedFriendsClient};
        use crate::clients::user_location_client::{self, UserLocationClient};
        use crate::clients::wingman_client::WingmanClient;
        use crate::hydration::community_source::{CommunityPost, CommunitySource};
        use crate::hydration::sources::{Backends, ProdSources, control};
        use crate::hydration::tweet_source::TweetSource;
        use crate::safety_label_source::SafetyLabelSource;
        use crate::safety_label_source::lookup::{
            LookupError, ManhattanLookup, RemoteSource, TwemcacheLookup,
        };
        use crate::safety_label_source::types::{FailureKind, ManhattanOutcome, TwemcacheOutcome};
        use rustc_hash::FxHashMap;
        use std::any::TypeId;
        use std::collections::HashSet;
        use std::fmt::Write as _;
        use std::hash::Hash;
        use thrift::protocol::TType;
        use tonic::metadata::MetadataMap;
        use tonic::{Request, Response, Status};
        use tower::ServiceExt;
        use tower::util::BoxCloneSyncService;
        use xai_core_entities::entities::{
            ApiCounts, CashtagAttachments, ConversationControlArm, EditControl,
            EscherbirdEntityAnnotation, ExclusiveTweetControl, GizmoduckAccount, MediaEntities,
            PCFLabel, QuotedTweet, ReactionContext, Safety, TakedownReason, TrustedFriendsControl,
            UrlEntities, UserResponseState,
        };
        use xai_core_entities::gizmoduck_client::{
            GizmoduckClient, LookupContext, MockGizmoduckClient, UserFields, ViewerData,
        };
        use xai_core_entities::tweet_entity_service_client::{MockTESClient, TESClient};
        use xai_strato::StratoGrpc;
        use xai_strato::strato_proto::rpc_server::{Rpc, RpcServer};
        use xai_strato::strato_proto::{Call3, Issue3, ResultList, result};
        use xai_x_thrift::entities::ApiMediaEntity;
        use xai_x_thrift::tweets::ApiPerspective;

        const VIEWER: u64 = 50;
        const AUTHOR: u64 = 10;
        const SUSPENDED_AUTHOR: u64 = 11;
        const SOURCE_AUTHOR: u64 = 12;
        const REPLIED_AUTHOR: u64 = 13;
        const INVITED: u64 = 14;
        const LABELED: u64 = 15;
        const TWEET: u64 = 1;
        const SOURCE_TWEET: u64 = 2;
        const REPLIED_TWEET: u64 = 3;
        const CONVERSATION: u64 = 4;
        const COMMUNITY: u64 = 40;
        const ARTICLE: u64 = 60;
        const CIRCLE: u64 = 7;
        const PLACE: u64 = 70;
        const EXCLUSIVE_AUTHOR: u64 = 16;
        const INITIAL_TWEET: u64 = 5;
        const EDIT_TWEET: u64 = 6;
        const ROOT: u64 = 20;
        const FOLLOWER: u64 = 30;
        const VIEWER_AS_ANSWERED: u64 = 51;
        const AUTHOR_AS_ANSWERED: u64 = 18;
        const SUSPENDED_AUTHOR_AS_ANSWERED: u64 = 19;
        const CORE_AUTHOR: u64 = 21;
        const CORE_REPLIED_AUTHOR: u64 = 22;
        const DIRECTED_AT: u64 = 23;
        const CORE_SOURCE_AUTHOR: u64 = 24;
        const TWEET_AS_ANSWERED: u64 = 25;
        const CORE_REPLIED_TWEET: u64 = 26;
        const CORE_SOURCE_TWEET: u64 = 27;
        const PARENT_TWEET: u64 = 28;
        const CORE_CONVERSATION: u64 = 29;
        const SECOND_TWEET: u64 = 33;
        const SECOND_EDIT_TWEET: u64 = 34;
        const SECOND_COMMUNITY: u64 = 41;
                const MISSING: u64 = 8;
                const FAILED: u64 = 9;
                const ABSENT: u64 = 17;
                const EDGES: [(Graph, EdgeDirection, bool); 3] = [
            (Graph::Follows, EdgeDirection::Forward, false),
            (Graph::Follows, EdgeDirection::Reverse, true),
            (Graph::Blocks, EdgeDirection::Forward, true),
        ];
        const FIELDS: [&[QueryFields]; 2] = [&[QueryFields::SAFETY], &[QueryFields::ACCOUNT]];
        const POST: CommunityPost = CommunityPost {
            tweet_id: TWEET,
            author_id: AUTHOR,
            community_id: COMMUNITY,
        };
        const MISSING_POST: CommunityPost = CommunityPost {
            tweet_id: MISSING,
            ..POST
        };
        const FAILED_POST: CommunityPost = CommunityPost {
            tweet_id: FAILED,
            ..POST
        };

                struct Failing<C>(C);

        fn failing<K: Eq + Hash, V>(
            mut answers: HashMap<K, anyhow::Result<V>>,
            key: impl Fn(u64) -> K,
        ) -> HashMap<K, anyhow::Result<V>> {
            answers.remove(&key(ABSENT));
            if let Some(answer) = answers.get_mut(&key(FAILED)) {
                *answer = Err(anyhow!("{FAILED} fails"));
            }
            answers
        }

        #[tonic::async_trait]
        impl TESClient for Failing<MockTESClient> {
            async fn get_tweet_core_datas(
                &self,
                ids: Vec<u64>,
            ) -> HashMap<u64, anyhow::Result<Option<PureCoreData>>> {
                failing(self.0.get_tweet_core_datas(ids).await, |id| id)
            }

            async fn get_tweet_media_entities(
                &self,
                ids: Vec<u64>,
            ) -> HashMap<u64, anyhow::Result<Option<MediaEntities>>> {
                failing(self.0.get_tweet_media_entities(ids).await, |id| id)
            }

            async fn get_subscription_author_ids(
                &self,
                ids: Vec<u64>,
            ) -> HashMap<u64, anyhow::Result<Option<u64>>> {
                failing(self.0.get_subscription_author_ids(ids).await, |id| id)
            }

            async fn get_conversation_controls(
                &self,
                ids: Vec<u64>,
            ) -> HashMap<u64, anyhow::Result<Option<ConversationControl>>> {
                failing(self.0.get_conversation_controls(ids).await, |id| id)
            }

            async fn get_quoted_tweets(
                &self,
                ids: Vec<u64>,
            ) -> HashMap<u64, anyhow::Result<Option<QuotedTweet>>> {
                failing(self.0.get_quoted_tweets(ids).await, |id| id)
            }

            async fn get_reaction_context(
                &self,
                ids: Vec<u64>,
            ) -> HashMap<u64, anyhow::Result<Option<ReactionContext>>> {
                failing(self.0.get_reaction_context(ids).await, |id| id)
            }

            async fn get_min_video_durations(
                &self,
                ids: Vec<u64>,
            ) -> HashMap<u64, anyhow::Result<Option<i64>>> {
                failing(self.0.get_min_video_durations(ids).await, |id| id)
            }

            async fn get_media_count(
                &self,
                ids: Vec<u64>,
            ) -> HashMap<u64, anyhow::Result<Option<i64>>> {
                failing(self.0.get_media_count(ids).await, |id| id)
            }

            async fn get_nullcast(
                &self,
                ids: Vec<u64>,
            ) -> HashMap<u64, anyhow::Result<Option<bool>>> {
                failing(self.0.get_nullcast(ids).await, |id| id)
            }

            async fn get_community(
                &self,
                ids: Vec<u64>,
            ) -> HashMap<u64, anyhow::Result<Option<i64>>> {
                failing(self.0.get_community(ids).await, |id| id)
            }

            async fn get_nsfw_user(
                &self,
                ids: Vec<u64>,
            ) -> HashMap<u64, anyhow::Result<Option<bool>>> {
                failing(self.0.get_nsfw_user(ids).await, |id| id)
            }

            async fn get_nsfw_admin(
                &self,
                ids: Vec<u64>,
            ) -> HashMap<u64, anyhow::Result<Option<bool>>> {
                failing(self.0.get_nsfw_admin(ids).await, |id| id)
            }

            async fn get_has_takedown(
                &self,
                ids: Vec<u64>,
            ) -> HashMap<u64, anyhow::Result<Option<bool>>> {
                failing(self.0.get_has_takedown(ids).await, |id| id)
            }

            async fn get_takedown_country_codes(
                &self,
                ids: Vec<u64>,
            ) -> HashMap<u64, anyhow::Result<Option<Vec<String>>>> {
                failing(self.0.get_takedown_country_codes(ids).await, |id| id)
            }

            async fn get_takedown_reasons(
                &self,
                ids: Vec<u64>,
            ) -> HashMap<u64, anyhow::Result<Option<Vec<TakedownReason>>>> {
                failing(self.0.get_takedown_reasons(ids).await, |id| id)
            }

            async fn get_language_code(
                &self,
                ids: Vec<u64>,
            ) -> HashMap<u64, anyhow::Result<Option<String>>> {
                failing(self.0.get_language_code(ids).await, |id| id)
            }

            async fn get_api_counts(
                &self,
                ids: Vec<u64>,
            ) -> HashMap<u64, anyhow::Result<Option<ApiCounts>>> {
                failing(self.0.get_api_counts(ids).await, |id| id)
            }

            async fn get_is_article(
                &self,
                ids: Vec<u64>,
            ) -> HashMap<u64, anyhow::Result<Option<bool>>> {
                failing(self.0.get_is_article(ids).await, |id| id)
            }

            async fn get_is_premium(
                &self,
                ids: Vec<u64>,
            ) -> HashMap<u64, anyhow::Result<Option<bool>>> {
                failing(self.0.get_is_premium(ids).await, |id| id)
            }

            async fn get_urls(
                &self,
                ids: Vec<u64>,
            ) -> HashMap<u64, anyhow::Result<Option<UrlEntities>>> {
                failing(self.0.get_urls(ids).await, |id| id)
            }

            async fn get_cashtag_attachments(
                &self,
                ids: Vec<u64>,
            ) -> HashMap<u64, anyhow::Result<Option<CashtagAttachments>>> {
                failing(self.0.get_cashtag_attachments(ids).await, |id| id)
            }

            async fn get_exclusive_controls(
                &self,
                ids: Vec<u64>,
            ) -> HashMap<u64, anyhow::Result<Option<ExclusiveTweetControl>>> {
                failing(self.0.get_exclusive_controls(ids).await, |id| id)
            }

            async fn get_trusted_friends_controls(
                &self,
                ids: Vec<u64>,
            ) -> HashMap<u64, anyhow::Result<Option<TrustedFriendsControl>>> {
                failing(self.0.get_trusted_friends_controls(ids).await, |id| id)
            }

            async fn get_grok_post_ids(
                &self,
                ids: Vec<u64>,
            ) -> HashMap<u64, anyhow::Result<Option<String>>> {
                failing(self.0.get_grok_post_ids(ids).await, |id| id)
            }

            async fn get_edit_control(
                &self,
                ids: Vec<u64>,
            ) -> HashMap<u64, anyhow::Result<Option<EditControl>>> {
                failing(self.0.get_edit_control(ids).await, |id| id)
            }

            async fn get_escherbird_entity_annotations(
                &self,
                ids: Vec<u64>,
            ) -> HashMap<u64, anyhow::Result<Option<Vec<EscherbirdEntityAnnotation>>>> {
                failing(self.0.get_escherbird_entity_annotations(ids).await, |id| id)
            }

            async fn get_core_data_and_api_counts(
                &self,
                tweet_id: u64,
            ) -> (
                anyhow::Result<Option<PureCoreData>>,
                anyhow::Result<Option<ApiCounts>>,
            ) {
                self.0.get_core_data_and_api_counts(tweet_id).await
            }

            async fn get_status_perspectives(
                &self,
                ids: Vec<u64>,
                metadata: Option<&MetadataMap>,
            ) -> HashMap<u64, anyhow::Result<Option<ApiPerspective>>> {
                failing(self.0.get_status_perspectives(ids, metadata).await, |id| id)
            }

            async fn get_api_media_entities(
                &self,
                ids: Vec<u64>,
                metadata: Option<&MetadataMap>,
            ) -> HashMap<u64, anyhow::Result<Option<Vec<ApiMediaEntity>>>> {
                failing(self.0.get_api_media_entities(ids, metadata).await, |id| id)
            }
        }

        type Users = HashMap<i64, anyhow::Result<Option<GizmoduckUserResult>>>;

        #[tonic::async_trait]
        impl GizmoduckClient for Failing<MockGizmoduckClient> {
            async fn get_users_with_context(
                &self,
                ids: Vec<i64>,
                context: Option<LookupContext>,
                fields: &[QueryFields],
            ) -> Users {
                let users = self.0.get_users_with_context(ids, context, fields).await;
                failing(users, u64::cast_signed)
            }

            async fn get_users(&self, _: Vec<i64>) -> Users {
                unreachable!()
            }

            async fn get_users_with_perspective(&self, _: i64, _: Vec<i64>) -> Users {
                unreachable!()
            }

            async fn get_viewer_roles(&self, _: u64) -> anyhow::Result<Vec<String>> {
                unreachable!()
            }

            async fn get_viewer_data(&self, _: u64) -> anyhow::Result<ViewerData> {
                unreachable!()
            }

            async fn get_viewer_data_with_fields(
                &self,
                _: u64,
                _: &[QueryFields],
            ) -> anyhow::Result<ViewerData> {
                unreachable!()
            }

            async fn get_pcf_labels(&self, _: Vec<i64>) -> HashMap<i64, anyhow::Result<PCFLabel>> {
                unreachable!()
            }

            async fn get_profile_description_languages(
                &self,
                _: Vec<i64>,
            ) -> HashMap<i64, anyhow::Result<Option<String>>> {
                unreachable!()
            }

            async fn get_user_fields(
                &self,
                _: Vec<i64>,
            ) -> HashMap<i64, anyhow::Result<UserFields>> {
                unreachable!()
            }

            async fn get_by_screen_name(
                &self,
                _: &str,
            ) -> anyhow::Result<Option<GizmoduckUserResult>> {
                unreachable!()
            }
        }

        struct Fake;

        #[tonic::async_trait]
        impl WingmanClient for Fake {
            async fn batch_exists_intersect(
                &self,
                _: u64,
                ids: &[u64],
            ) -> Option<Vec<wingman_client::Exists>> {
                let exists = ids.iter().map(|&id| match id {
                    MISSING => wingman_client::Exists::NotFound,
                    FAILED => wingman_client::Exists::ItemError,
                    _ => wingman_client::Exists::Found,
                });
                Some(exists.collect())
            }
        }

                        #[tonic::async_trait]
        impl SocialgraphClient for Fake {
            async fn select_edges(
                &self,
                _: u64,
                queries: &[EdgeQuery],
            ) -> Result<Vec<Option<HashSet<u64>>>, tonic::Status> {
                let asks = |query: &EdgeQuery, id| query.destination_ids.contains(&id);
                if queries.iter().any(|query| asks(query, FAILED)) {
                    return Err(tonic::Status::unavailable("select failed"));
                }
                let sets = queries.iter().map(|query| {
                    let holds = EDGES.contains(&(query.graph, query.direction, true));
                    let set = query.destination_ids.iter().copied().filter(|_| holds);
                    (!asks(query, MISSING)).then(|| set.collect())
                });
                Ok(sets.collect())
            }
        }

        #[tonic::async_trait]
        impl TwemcacheLookup for Fake {
            async fn get(&self, ids: &[u64]) -> FxHashMap<u64, TwemcacheOutcome> {
                ids.iter()
                    .map(|&id| {
                        let outcome = match id {
                            MISSING => TwemcacheOutcome::NotFound,
                            _ => TwemcacheOutcome::Miss,
                        };
                        (id, outcome)
                    })
                    .collect()
            }
        }

        #[tonic::async_trait]
        impl ManhattanLookup for Fake {
            async fn get(&self, ids: &[u64]) -> FxHashMap<u64, ManhattanOutcome> {
                let label = vf_pb::SafetyLabel {
                    applicable_users: vec![LABELED.cast_signed()],
                    ..Default::default()
                };
                let labels = vf_pb::SafetyLabelMap {
                    labels: [(1, label)].into(),
                };
                ids.iter()
                    .map(|&id| {
                        let outcome = match id {
                            FAILED => ManhattanOutcome::Failure(LookupError::new(
                                FailureKind::ManhattanFetch,
                                "fails",
                            )),
                            _ => ManhattanOutcome::Resolved(labels.clone()),
                        };
                        (id, outcome)
                    })
                    .collect()
            }
        }

        fn id(id: u64) -> Thrift {
            Thrift::I64(id.cast_signed())
        }

        fn tweet() -> Thrift {
            use Thrift::{Bool, List, Str, Struct};
            let ids = |ids: &[u64]| List(TType::I64, ids.iter().copied().map(id).collect());
            let core_data = Struct(vec![
                (1, id(CORE_AUTHOR)),
                (
                    5,
                    Struct(vec![
                        (1, id(CORE_REPLIED_TWEET)),
                        (2, id(CORE_REPLIED_AUTHOR)),
                    ]),
                ),
                (6, Struct(vec![(1, id(DIRECTED_AT))])),
                (
                    7,
                    Struct(vec![
                        (1, id(CORE_SOURCE_TWEET)),
                        (2, id(CORE_SOURCE_AUTHOR)),
                        (3, id(PARENT_TWEET)),
                    ]),
                ),
                (9, Bool(true)),
                (14, id(CORE_CONVERSATION)),
            ]);
            Struct(vec![
                (1, id(TWEET_AS_ANSWERED)),
                (2, core_data),
                (125, Struct(vec![(1, ids(&[COMMUNITY, SECOND_COMMUNITY]))])),
                (155, Struct(vec![(1, id(EXCLUSIVE_AUTHOR))])),
                (156, Struct(vec![(1, id(CIRCLE))])),
                (
                    157,
                    Struct(vec![(
                        2,
                        Struct(vec![
                            (1, id(INITIAL_TWEET)),
                            (2, Struct(vec![(1, ids(&[EDIT_TWEET]))])),
                        ]),
                    )]),
                ),
                (170, Struct(vec![(1, id(ARTICLE))])),
                (172, Struct(vec![(1, Str(format!("{PLACE:x}")))])),
            ])
        }

        fn located() -> Thrift {
            use Thrift::{I32, Map, Set, Struct};
            const CITY: i32 = 3;
            let places = vec![(I32(CITY), Set(TType::I64, vec![id(PLACE)]))];
            let viewer = Struct(vec![
                (1, id(VIEWER)),
                (5, Map(TType::I32, TType::Set, places)),
            ]);
            let found = Map(TType::I64, TType::Struct, vec![(id(VIEWER), viewer)]);
            Struct(vec![(1, found)])
        }

        fn second_tweet() -> Thrift {
            use Thrift::{List, Struct};
            let edits = List(TType::I64, vec![id(SECOND_EDIT_TWEET)]);
            Struct(vec![
                (2, Struct(vec![])),
                (157, Struct(vec![(1, Struct(vec![(1, edits)]))])),
            ])
        }

        struct Strato;

        #[tonic::async_trait]
        impl Rpc for Strato {
            async fn issue3(
                &self,
                request: Request<Issue3>,
            ) -> Result<Response<ResultList>, Status> {
                use Thrift::{Bool, I32, I64, Set, Str, Struct};
                const SOFT_DELETED: i32 = 2;
                                                fn asks(call: &Call3, id: u64) -> bool {
                    let key = match call.path.as_str() {
                        "trusted-friends/visibility-filtering/isMember.TrustedFriendsList"
                        | "trusted-friends/visibility-filtering/isOwner.TrustedFriendsList" => {
                            trusted_friends_client::key(id, VIEWER)
                        }
                        "geo/service/userLocation" => user_location_client::key(id),
                        _ => xai_strato::encode(&(id, ())),
                    };
                    call.args.first().is_some_and(|asked| *asked == key)
                }
                let results = request
                    .get_ref()
                    .calls
                    .iter()
                    .filter(|call| !asks(call, ABSENT))
                    .map(|call| {
                        if asks(call, FAILED) {
                            return Ok(xai_strato::strato_proto::Result {
                                result_type: Some(result::ResultType::Err(
                                    xai_strato::strato_proto::Error {
                                        code: 5,
                                        message: "fails".to_owned(),
                                    },
                                )),
                            });
                        }
                        let value = match call.path.as_str() {
                            _ if asks(call, MISSING) => None,
                            "tweetypie/federated/tweetForVisibility.Tweet"
                                if asks(call, SECOND_TWEET) =>
                            {
                                Some(second_tweet())
                            }
                            "tweetypie/federated/tweetForVisibility.Tweet" => Some(tweet()),
                            "communities/moderationState.TweetCommunityRelationship"
                            | "communities/moderationState.UserCommunityRelationship" => {
                                Some(Struct(vec![(2, Struct(vec![]))]))
                            }
                            "communities/visibility/visibilityFeatures.Community" => {
                                Some(Struct(vec![(1, Struct(vec![(4, Bool(true))]))]))
                            }
                            "communities/isRemoved.Community" => Some(Bool(true)),
                            "about_this_account/tfe_top_country.User" => {
                                Some(Struct(vec![(2, Struct(vec![(1, Str("us".to_owned()))]))]))
                            }
                            "article/fields/mediaEntities.ArticleEntity" => Some(Set(
                                TType::Struct,
                                vec![Struct(vec![(1, Struct(vec![(1, I64(7)), (2, I64(0))]))])],
                            )),
                            "article/fields/coverMedia.ArticleEntity" => {
                                Some(Struct(vec![(1, Struct(vec![(1, I64(7)), (2, I64(0))]))]))
                            }
                            "article/fields/lifecycleState.ArticleEntity" => {
                                Some(Struct(vec![(1, I32(SOFT_DELETED))]))
                            }
                            "trusted-friends/visibility-filtering/isMember.TrustedFriendsList"
                            | "trusted-friends/visibility-filtering/isOwner.TrustedFriendsList" => {
                                Some(Bool(true))
                            }
                            "geo/service/userLocation" => Some(located()),
                            path => return Err(Status::unimplemented(path)),
                        };
                        let answer = value.map_or_else(no_row, found);
                        Ok(xai_strato::strato_proto::Result {
                            result_type: Some(result::ResultType::Ok(answer.into())),
                        })
                    })
                    .collect::<Result<_, _>>()?;
                Ok(Response::new(ResultList { results }))
            }
        }

        fn strato() -> StratoGrpc {
            StratoGrpc::from_boxed_service(BoxCloneSyncService::new(ServiceExt::<
                tonic::codegen::http::Request<tonic::body::Body>,
            >::map_err(
                RpcServer::new(Strato),
                |never| match never {},
            )))
        }

        fn prod_sources() -> ProdSources {
            let found = |user| {
                Some(GizmoduckUserResult {
                    user: Some(user),
                    response_state: Some(UserResponseState::Found),
                })
            };
            let tes = MockTESClient {
                core_data: HashMap::from([(
                    TWEET,
                    Some(PureCoreData {
                        author_id: AUTHOR,
                        source_tweet_id: Some(SOURCE_TWEET),
                        source_user_id: Some(SOURCE_AUTHOR),
                        in_reply_to_tweet_id: Some(REPLIED_TWEET),
                        in_reply_to_user_id: Some(REPLIED_AUTHOR),
                        conversation_id: Some(CONVERSATION),
                        ..Default::default()
                    }),
                )]),
                conversation_controls: HashMap::from([(
                    TWEET,
                    Some(ConversationControl {
                        invited_user_ids: vec![INVITED],
                        ..control(ConversationControlArm::ByInvitation, AUTHOR, &[])
                    }),
                )]),
                ..Default::default()
            };
            let gizmoduck = MockGizmoduckClient {
                users: HashMap::from([
                    (
                        VIEWER.cast_signed(),
                        found(GizmoduckUser {
                            user_id: VIEWER_AS_ANSWERED,
                            account: GizmoduckAccount {
                                nsfw_view: true,
                                country_code: Some("US".to_owned()),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                    ),
                    (
                        AUTHOR.cast_signed(),
                        found(GizmoduckUser {
                            user_id: AUTHOR_AS_ANSWERED,
                            ..Default::default()
                        }),
                    ),
                    (
                        SUSPENDED_AUTHOR.cast_signed(),
                        found(GizmoduckUser {
                            user_id: SUSPENDED_AUTHOR_AS_ANSWERED,
                            safety: Safety {
                                suspended: true,
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                    ),
                ]),
                ..Default::default()
            };
            let fake = Arc::new(Fake);
            ProdSources::new(Backends {
                tes: Arc::new(Failing(tes)),
                tweets: TweetSource {
                    grpc_client: Arc::new(strato()),
                },
                gizmoduck: GizmoduckLookup::new(Arc::new(Failing(gizmoduck))),
                socialgraph: Arc::clone(&fake) as _,
                about_this_account: AboutThisAccountClient::new(strato()),
                wingman: Arc::clone(&fake) as _,
                articles: ArticleClient::new(strato()),
                trusted_friends: TrustedFriendsClient::new(strato()),
                user_location: UserLocationClient::new(strato()),
                safety_labels: Arc::new(SafetyLabelSource::new(
                    Arc::new(RemoteSource::new(Arc::clone(&fake), fake)),
                    None,
                )),
                communities: CommunitySource {
                    grpc_client: strato(),
                },
            })
        }

        fn shown<V: fmt::Debug>(batch: &RawHydrationBatch<V>, keys: &[u64]) -> String {
            let shown: Vec<String> = keys
                .iter()
                .map(|key| format!("{key}: {:?}", batch.hydrated(key)))
                .collect();
            shown.join(", ")
        }

        async fn ask(sources: &dyn Sources, source: Source) -> String {
            match source {
                Source::TesPureCoreData => {
                    let keys = [TWEET, MISSING, FAILED, ABSENT];
                    shown(&sources.pure_cores(&keys).await.into_inner(), &keys)
                }
                Source::TesConversationRoot => {
                    let keys = [TWEET, MISSING, FAILED, ABSENT];
                    shown(&sources.conversation_roots(&keys).await.into_inner(), &keys)
                }
                Source::TesTweet => {
                    let keys = [TWEET, SECOND_TWEET, MISSING, FAILED, ABSENT];
                    shown(&sources.tweets(&keys).await.into_inner(), &keys)
                }
                Source::TesConversationControl => {
                    let keys = [TWEET, MISSING, FAILED, ABSENT];
                    shown(
                        &sources.conversation_controls(&keys).await.into_inner(),
                        &keys,
                    )
                }
                Source::SafetyLabels => {
                    let keys = [TWEET, MISSING, FAILED];
                    shown(&sources.safety_labels(&keys).await.into_inner(), &keys)
                }
                Source::GizmoduckViewer => {
                    let asked = FIELDS
                        .map(|fields| (VIEWER, fields))
                        .into_iter()
                        .chain([MISSING, FAILED].map(|viewer| (viewer, FIELDS[0])));
                    let mut shown_viewers = vec![];
                    for (viewer_id, fields) in asked {
                        let viewer = sources
                            .viewer(viewer_id, fields)
                            .await
                            .into_inner()
                            .map(|viewer| (viewer.profile, viewer.has_age_verified_18_label));
                        shown_viewers.push(shown(&viewer, &[viewer_id]));
                    }
                    shown_viewers.join(", ")
                }
                Source::GizmoduckAuthor => {
                    let asked: [&[u64]; 2] =
                        [&[AUTHOR, MISSING, FAILED, ABSENT], &[SUSPENDED_AUTHOR]];
                    let mut shown_authors = vec![];
                    for (authors, fields) in asked.into_iter().zip(FIELDS) {
                        let users = sources.users(authors, fields).await.into_inner();
                        shown_authors.push(shown(&users, authors));
                    }
                    shown_authors.join(", ")
                }
                Source::Flock => {
                    let query = |graph, direction, destination| EdgeQuery {
                        graph,
                        direction,
                        destination_ids: vec![destination],
                    };
                    let held: Vec<EdgeQuery> = EDGES
                        .map(|(graph, direction, _)| query(graph, direction, FOLLOWER))
                        .into_iter()
                        .chain([query(Graph::Follows, EdgeDirection::Forward, MISSING)])
                        .collect();
                    let failed = vec![query(Graph::Follows, EdgeDirection::Forward, FAILED)];
                    let mut shown_edges = vec![];
                    for queries in [held, failed] {
                        let batches = sources.select_edges(VIEWER, &queries).await.into_inner();
                        for (query, batch) in queries.iter().zip(&batches) {
                            shown_edges.push(shown(batch, &query.destination_ids));
                        }
                    }
                    shown_edges.join(", ")
                }
                Source::ViewerCountry => {
                    let mut shown_countries = vec![];
                    for viewer in [VIEWER, MISSING, FAILED] {
                        let country = sources.viewer_country(viewer).await.into_inner();
                        shown_countries.push(shown(&country, &[viewer]));
                    }
                    shown_countries.join(", ")
                }
                Source::Wingman => {
                    let keys = [ROOT, MISSING, FAILED];
                    shown(
                        &sources.second_degree(VIEWER, &keys).await.into_inner(),
                        &keys,
                    )
                }
                Source::CommunityModeration => {
                    let posts = [POST, MISSING_POST, FAILED_POST];
                    let batch = sources.community_moderations(&posts).await.into_inner();
                    shown(&batch, &posts.map(|post| post.tweet_id))
                }
                Source::CommunityModerator => {
                    let keys = [COMMUNITY, MISSING, FAILED];
                    shown(
                        &sources
                            .community_moderators(VIEWER, &keys)
                            .await
                            .into_inner(),
                        &keys,
                    )
                }
                Source::CommunityViewerRemoved => {
                    let keys = [COMMUNITY, MISSING, FAILED];
                    shown(
                        &sources
                            .community_viewer_removals(VIEWER, &keys)
                            .await
                            .into_inner(),
                        &keys,
                    )
                }
                Source::ArticleMediaEntities => {
                    let keys = [ARTICLE, MISSING, FAILED];
                    shown(
                        &sources
                            .article_media_entities(Some(VIEWER), &keys)
                            .await
                            .into_inner(),
                        &keys,
                    )
                }
                Source::ArticleCoverMedia => {
                    let keys = [ARTICLE, MISSING, FAILED];
                    shown(
                        &sources
                            .article_cover_media(Some(VIEWER), &keys)
                            .await
                            .into_inner(),
                        &keys,
                    )
                }
                Source::ArticleLifecycle => {
                    let keys = [ARTICLE, MISSING, FAILED];
                    shown(&sources.article_lifecycles(&keys).await.into_inner(), &keys)
                }
                Source::TrustedFriends => {
                    let keys = [CIRCLE, MISSING, FAILED];
                    shown(
                        &sources.trusted_friends(VIEWER, &keys).await.into_inner(),
                        &keys,
                    )
                }
                Source::UserLocation => {
                    let keys = [PLACE, MISSING];
                    let located = sources.outside_places(VIEWER, &keys).await;
                    let failed = sources.outside_places(FAILED, &[FAILED]).await;
                    let located = shown(&located.into_inner(), &keys);
                    let failed = shown(&failed.into_inner(), &[FAILED]);
                    format!("{located}, {failed}")
                }
            }
        }

        async fn answered() -> (Vec<(Source, String)>, Recording) {
            let recorder = Arc::new(Recorder::default());
            let prod = prod_sources().observed(Arc::clone(&recorder));
            let mut answers = vec![];
            for &source in Source::VARIANTS {
                answers.push((source, ask(&prod, source).await));
            }
            (answers, recorder.take())
        }

        struct Forget<'a>(&'a mut Recording);

        impl Visit for Forget<'_> {
            fn exchange<X: Exchange>(&mut self) {
                self.0.answers.remove(X::NAME);
            }
        }

        fn without(recording: &Recording, source: Source) -> Recording {
            let mut recording = recording.clone();
            source.visit(&mut Forget(&mut recording));
            recording
        }

        fn defaults() -> InMemorySources {
            let user = || GizmoduckUserResult {
                user: Some(GizmoduckUser::default()),
                response_state: Some(UserResponseState::Found),
            };
            InMemorySources::default()
                .pure_core(TWEET, PureCoreData::default())
                .labels(TWEET, vf_pb::SafetyLabelMap::default())
                .viewer(VIEWER, GizmoduckUser::default())
                .user(AUTHOR, user())
                .user(SUSPENDED_AUTHOR, user())
                .country(VIEWER, "")
        }

        #[tokio::test]
        async fn no_fake_answers_as_defaults_or_a_replay_without_its_source() {
            let (answers, recording) = answered().await;

            let mut indistinct = vec![];
            for (source, answer) in answers {
                let replay = without(&recording, source)
                    .into_replay(Some(VIEWER))
                    .unwrap();
                let replayed = ask(&replay, source).await;
                if replayed == answer || ask(&defaults(), source).await == answer {
                    indistinct.push(source);
                }
            }
            assert_eq!(indistinct, Vec::<Source>::new());
        }

        #[tokio::test]
        async fn a_recording_keeps_the_corpus_shape_and_names_failed_keys() {
            let (_, recording) = answered().await;
            let json = serde_json::to_value(&recording).unwrap();

            assert_eq!(
                json["trusted_friends"],
                serde_json::json!({"7": {"found": true}, "8": {"found": false}, "9": "failed"})
            );
            assert_eq!(
                json["edges"],
                serde_json::json!({
                    "blocks/Forward/30": {"found": true},
                    "follows/Forward/30": {"found": false},
                    "follows/Forward/8": {"partial": false},
                    "follows/Forward/9": "failed",
                    "follows/Reverse/30": {"found": true},
                })
            );
            for fields in ["viewer_fields", "user_fields"] {
                assert_eq!(json[fields], serde_json::json!(["sAFETY", "aCCOUNT"]));
            }
            for bytes in ["tweets", "safety_labels"] {
                let hex = json[bytes][TWEET.to_string()]["found"].as_str();
                assert!(
                    hex.is_some_and(|hex| hex.bytes().all(|byte| byte.is_ascii_hexdigit())),
                    "{bytes}: {hex:?}"
                );
            }
        }

        fn shape(recording: &Recording) -> String {
            let kind = |answer: &Answer<_>| match answer {
                Answer::Found(_) => "found",
                Answer::Partial(_) => "partial",
                Answer::NotFound => "not_found",
                Answer::Failed => "failed",
            };
            let (_, shared) = recording.clone().split_shared();
            let shared = shared.names();
            let unsuccessful = recording.unsuccessful();
            let mut rendered = String::new();
            for name in recording.names() {
                let side = if shared.contains(&name) {
                    "shared"
                } else {
                    "case"
                };
                let kinds: BTreeSet<&str> = match recording.answers.get(name) {
                    Some(answers) => answers.values().map(kind).collect(),
                    None => BTreeSet::from(["fields"]),
                };
                let failed: Vec<&str> = unsuccessful
                    .iter()
                    .filter_map(|key| key.strip_prefix(name)?.strip_prefix('/'))
                    .collect();
                writeln!(rendered, "{name}: {side} {kinds:?} {failed:?}").unwrap();
            }
            let logged_out = recording.clone().into_replay(None).err();
            let logged_out = logged_out.map(|error| error.to_string());
            writeln!(rendered, "logged out: {logged_out:?}").unwrap();
            rendered
        }

        #[tokio::test]
        async fn the_recording_matches_recording_shape_txt() {
            let (_, recording) = answered().await;
            crate::hydration::assert_blessed(
                "recording_shape.txt",
                include_str!("../../tests/fixtures/recording_shape.txt"),
                &shape(&recording),
            );
        }

        #[tokio::test]
        async fn shared_answers_split_out_and_join_back_unchanged() {
            let (_, recording) = answered().await;
            let (case, shared) = recording.clone().split_shared();
            assert_eq!(case.with_shared(&shared), recording);
        }

        #[tokio::test]
        async fn every_source_replays_as_prod_answered() {
            let (answers, recorded) = answered().await;
            let recording: Recording =
                serde_json::from_value(serde_json::to_value(recorded).unwrap()).unwrap();

            let mut unrecorded = vec![];
            let mut diffs = vec![];
            for (source, answer) in answers {
                let replay = recording.clone().into_replay(Some(VIEWER)).unwrap();
                let replayed = ask(&replay, source).await;
                let misses = replay.misses();
                if replayed != answer || !misses.is_empty() {
                    unrecorded.push(source);
                    diffs.push(format!(
                        "{source:?}: prod {answer}, replay {replayed}, misses {misses:?}"
                    ));
                }
            }
            assert_eq!(unrecorded, Vec::<Source>::new(), "{diffs:#?}");
        }

                fn in_bytes(name: &str) -> Option<Vec<Id>> {
            let users = [
                CORE_AUTHOR,
                CORE_REPLIED_AUTHOR,
                DIRECTED_AT,
                CORE_SOURCE_AUTHOR,
                EXCLUSIVE_AUTHOR,
                CIRCLE,
            ];
            let tweets = [
                TWEET,
                SECOND_TWEET,
                TWEET_AS_ANSWERED,
                CORE_REPLIED_TWEET,
                CORE_SOURCE_TWEET,
                PARENT_TWEET,
                CORE_CONVERSATION,
                INITIAL_TWEET,
                EDIT_TWEET,
                SECOND_EDIT_TWEET,
            ];
            match name {
                "tweets" => Some(
                    users
                        .map(Id::User)
                        .into_iter()
                        .chain(tweets.map(Id::Tweet))
                        .chain([COMMUNITY, SECOND_COMMUNITY].map(Id::Community))
                        .chain([Id::Article(ARTICLE), Id::Place(PLACE)])
                        .collect(),
                ),
                "safety_labels" => Some(vec![Id::User(LABELED), Id::Tweet(TWEET)]),
                _ => None,
            }
        }

                const NO_IDS: [&str; 1] = [exchange::ArticleLifecycle::NAME];

        fn numbers(json: &serde_json::Value, into: &mut Vec<u64>) {
            match json {
                serde_json::Value::Number(number) => into.extend(
                    number
                        .as_u64()
                        .or_else(|| number.as_i64().map(i64::cast_unsigned))
                        .filter(|&number| number != 0),
                ),
                serde_json::Value::Array(values) => {
                    values.iter().for_each(|value| numbers(value, into));
                }
                serde_json::Value::Object(fields) => {
                    fields.values().for_each(|value| numbers(value, into));
                }
                serde_json::Value::Null
                | serde_json::Value::Bool(_)
                | serde_json::Value::String(_) => {}
            }
        }

                        struct Scan<'a> {
            recording: &'a Recording,
            walked: &'a [Id],
            unwalked: Vec<String>,
        }

        impl Visit for Scan<'_> {
            fn exchange<X: Exchange>(&mut self) {
                let name = X::NAME;
                let Some(answers) = self.recording.answers.get(name) else {
                    self.unwalked.push(format!("{name}: answered nothing"));
                    return;
                };
                let bytes = TypeId::of::<X::Wire>() == TypeId::of::<Bytes>();
                if bytes {
                    let planted = in_bytes(name)
                        .unwrap_or_else(|| panic!("{name}: in_bytes names no id its bytes hold"));
                    self.unwalked.extend(
                        planted
                            .into_iter()
                            .filter(|id| !self.walked.contains(id))
                            .map(|id| format!("{name}: {id:?}")),
                    );
                }
                for (key, answer) in answers {
                    let mut named: Vec<u64> = key
                        .split(|c: char| !c.is_ascii_digit())
                        .filter_map(|digits| digits.parse().ok())
                        .collect();
                    if let Answer::Found(json) | Answer::Partial(json) = answer
                        && !bytes
                        && !NO_IDS.contains(&name)
                    {
                        numbers(json, &mut named);
                    }
                    let walked = |number: &u64| {
                        self.walked.iter().any(|&id| {
                            let (Id::User(id)
                            | Id::Tweet(id)
                            | Id::Community(id)
                            | Id::Article(id)
                            | Id::Place(id)) = id;
                            id == *number
                        })
                    };
                    self.unwalked.extend(
                        named
                            .iter()
                            .filter(|number| !walked(number))
                            .map(|number| format!("{name}/{key}: {number}")),
                    );
                }
            }
        }

        #[tokio::test]
        async fn every_id_a_source_records_is_walked() {
            let mut unwalked = vec![];
            for &source in Source::VARIANTS {
                let recorder = Arc::new(Recorder::default());
                ask(&prod_sources().observed(Arc::clone(&recorder)), source).await;
                let recording = recorder.take();
                let walked = recording.ids().unwrap();
                let mut scan = Scan {
                    recording: &recording,
                    walked: &walked,
                    unwalked: vec![],
                };
                source.visit(&mut scan);
                unwalked.extend(
                    scan.unwalked
                        .into_iter()
                        .map(|unwalked| format!("{source:?}: {unwalked}")),
                );
            }
            assert_eq!(unwalked, Vec::<String>::new());
        }
    }
}
