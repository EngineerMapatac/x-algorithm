use crate::clients::about_this_account_client::AboutThisAccountClient;
use crate::clients::article_client::ArticleClient;
use crate::clients::gizmoduck_client::GizmoduckLookup;
use crate::clients::socialgraph_client::{EdgeQuery, SocialgraphClient};
use crate::clients::trusted_friends_client::TrustedFriendsClient;
use crate::clients::user_location_client::UserLocationClient;
use crate::clients::wingman_client::WingmanClient;
use crate::hydration::batch::{Hydrated, HydrationBatch, HydrationError, RawHydrationBatch};
use crate::hydration::community_source::{CommunityPost, CommunitySource};
use crate::hydration::decode::article::decode_lifecycle;
pub(crate) use crate::hydration::decode::author::{author_batch, DecodedAuthor};
use crate::hydration::decode::tweet::pure_core;
use crate::hydration::decode::viewer::decode_viewer;
pub(crate) use crate::hydration::decode::viewer::DecodedViewer;
use crate::hydration::tweet_source::{decode_tweet, TweetSource};
use crate::models::{ArticleLifecycle, CommunityModeration, PureCore, TweetFeatures};
use crate::post_card::PostCard;
use crate::safety_label_source::SafetyLabelSource;
pub(crate) use exchange::{Bytes, EdgeKey, Exchange, Id, Scope, Visit};
use futures::future::join;
use prost::Message;
use rustc_hash::FxHashMap;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::iter;
use std::sync::Arc;
use tonic::metadata::{MetadataMap, MetadataValue};
use tracing::warn;
use wingman_client::Exists;
use xai_core_entities::entities::{ConversationControl, GizmoduckUser, PureCoreData};
use xai_core_entities::gizmoduck_client::QueryFields;
use xai_core_entities::tweet_entity_service_client::TESClient;
use xai_visibility_filtering_proto as vf_pb;

pub(crate) mod exchange;

pub(crate) use observed::Observed;

mod observed {
    use super::{Exchange, Hydrated, Observer};
    use std::marker::PhantomData;

    pub(crate) struct Observed<X, T> {
        answer: T,
        exchange: PhantomData<fn() -> X>,
    }

    impl<X: Exchange> Observed<X, ()> {
        pub(super) fn new(
            observer: &impl Observer,
            answers: impl Iterator<Item = (X::Key, Hydrated<X::Wire>)>,
        ) -> Self {
            observer.answered::<X>(answers);
            Self {
                answer: (),
                exchange: PhantomData,
            }
        }
    }

    impl<X, T> Observed<X, T> {
        #[cfg(test)]
        pub(crate) fn quiet(answer: T) -> Self {
            Self {
                answer,
                exchange: PhantomData,
            }
        }

        pub(crate) fn map<U>(self, decode: impl FnOnce(T) -> U) -> Observed<X, U> {
            Observed {
                answer: decode(self.answer),
                exchange: PhantomData,
            }
        }

        pub(crate) fn into_inner(self) -> T {
            self.answer
        }
    }
}

pub(crate) type Answered<X, V> = Observed<X, RawHydrationBatch<V>>;

#[tonic::async_trait]
pub(crate) trait Sources: Send + Sync {
    async fn pure_cores(&self, tweet_ids: &[u64]) -> Answered<exchange::TesPureCoreData, PureCore>;

    async fn conversation_roots(
        &self,
        root_ids: &[u64],
    ) -> Answered<exchange::TesConversationRoot, PureCore>;

    async fn tweets(&self, tweet_ids: &[u64]) -> Answered<exchange::TesTweet, TweetFeatures>;

    async fn conversation_controls(
        &self,
        tweet_ids: &[u64],
    ) -> Answered<exchange::TesConversationControl, ConversationControl>;

    async fn safety_labels(
        &self,
        tweet_ids: &[u64],
    ) -> Answered<exchange::SafetyLabels, Arc<vf_pb::SafetyLabelMap>>;

    async fn viewer(
        &self,
        viewer_id: u64,
        fields: &[QueryFields],
    ) -> Answered<exchange::GizmoduckViewer, DecodedViewer>;

    async fn users(
        &self,
        user_ids: &[u64],
        fields: &[QueryFields],
    ) -> Answered<exchange::GizmoduckAuthor, DecodedAuthor>;

    async fn select_edges(
        &self,
        viewer_id: u64,
        queries: &[EdgeQuery],
    ) -> Observed<exchange::Flock, Vec<RawHydrationBatch<bool>>>;

    async fn viewer_country(&self, viewer_id: u64) -> Answered<exchange::ViewerCountry, Arc<str>>;

    async fn second_degree(
        &self,
        viewer_id: u64,
        root_author_ids: &[u64],
    ) -> Answered<exchange::Wingman, bool>;

    async fn community_moderations(
        &self,
        posts: &[CommunityPost],
    ) -> Answered<exchange::CommunityModeration, CommunityModeration>;

    async fn community_moderators(
        &self,
        viewer_id: u64,
        community_ids: &[u64],
    ) -> Answered<exchange::CommunityModerator, bool>;

    async fn community_viewer_removals(
        &self,
        viewer_id: u64,
        community_ids: &[u64],
    ) -> Answered<exchange::CommunityViewerRemoved, bool>;

    async fn article_lifecycles(
        &self,
        article_ids: &[u64],
    ) -> Answered<exchange::ArticleLifecycle, ArticleLifecycle>;

    async fn article_media_entities(
        &self,
        viewer_id: Option<u64>,
        article_ids: &[u64],
    ) -> Answered<exchange::ArticleMediaEntities, bool>;

    async fn article_cover_media(
        &self,
        viewer_id: Option<u64>,
        article_ids: &[u64],
    ) -> Answered<exchange::ArticleCoverMedia, bool>;

    async fn trusted_friends(
        &self,
        viewer_id: u64,
        list_ids: &[u64],
    ) -> Answered<exchange::TrustedFriends, bool>;

    async fn outside_places(
        &self,
        viewer_id: u64,
        place_ids: &[u64],
    ) -> Answered<exchange::UserLocation, bool>;

    async fn post_card(&self, tweet_id: u64) -> Option<PostCard>;
}

fn hydration_error(status: &tonic::Status) -> HydrationError {
    let timer_expired = iter::successors(std::error::Error::source(status), |err| err.source())
        .any(|err| err.is::<tonic::TimeoutExpired>());
    if timer_expired || status.code() == tonic::Code::DeadlineExceeded {
        HydrationError::Timeout
    } else {
        HydrationError::Error
    }
}

fn edge_batches(
    queries: &[EdgeQuery],
    sets: Result<&[Option<HashSet<u64>>], &tonic::Status>,
) -> Vec<RawHydrationBatch<bool>> {
    let sets = sets.map_err(hydration_error);
    queries
        .iter()
        .enumerate()
        .map(|(position, query)| {
            let set = sets
                .as_ref()
                .map(|sets| sets.get(position).and_then(Option::as_ref));
            let answers = query.destination_ids.iter().map(|&destination| {
                let answer = match set {
                    Err(error) => Hydrated::Failed(error.clone()),
                    Ok(None) => Hydrated::Partial(false),
                    Ok(Some(set)) => Hydrated::Found(set.contains(&destination)),
                };
                (destination, answer)
            });
            HydrationBatch::from_hydrated(answers.collect())
        })
        .collect()
}

pub(crate) fn core_batch(cores: RawHydrationBatch<PureCoreData>) -> RawHydrationBatch<PureCore> {
    cores.map(|core| pure_core(&core))
}

pub(crate) fn tweet_batch<B: AsRef<[u8]>>(
    tweet_ids: &[u64],
    values: HashMap<u64, anyhow::Result<B>>,
) -> RawHydrationBatch<TweetFeatures> {
    let tweets: FxHashMap<_, _> = values
        .into_iter()
        .map(|(id, value)| (id, value.and_then(|bytes| decode_tweet(bytes.as_ref()))))
        .collect();
    HydrationBatch::from_results(tweet_ids.iter().copied(), tweets)
}

pub(crate) fn label_batch<E>(
    tweet_ids: &[u64],
    labels: FxHashMap<u64, Result<Arc<vf_pb::SafetyLabelMap>, E>>,
) -> RawHydrationBatch<Arc<vf_pb::SafetyLabelMap>> {
    let labels: FxHashMap<_, _> = labels
        .into_iter()
        .map(|(id, labels)| (id, labels.map(Some)))
        .collect();
    HydrationBatch::from_results(tweet_ids.iter().copied(), labels)
}

pub(crate) fn viewer_batch<E>(
    viewer_id: u64,
    viewer: Result<Option<GizmoduckUser>, E>,
    fields: &[QueryFields],
) -> RawHydrationBatch<DecodedViewer> {
    let viewer = viewer.map(|user| Some(decode_viewer(user.as_ref(), fields)));
    one_viewer_batch(viewer_id, viewer)
}

pub(crate) fn country_batch<E>(
    viewer_id: u64,
    country: Result<Option<String>, E>,
) -> RawHydrationBatch<Arc<str>> {
    one_viewer_batch(viewer_id, country.map(|country| country.map(Arc::from)))
}

fn one_viewer_batch<V, E>(viewer_id: u64, answer: Result<Option<V>, E>) -> RawHydrationBatch<V> {
    HydrationBatch::from_results([viewer_id], HashMap::from([(viewer_id, answer)]))
}

pub(crate) fn lifecycle_batch(rows: RawHydrationBatch<i32>) -> RawHydrationBatch<ArticleLifecycle> {
    let decoded = rows.into_hydrated().into_iter().map(|(id, row)| {
        let lifecycle = match row {
            Hydrated::Found(value) => decode_lifecycle(value)
                .map_or(Hydrated::Failed(HydrationError::Error), Hydrated::Found),
            Hydrated::Partial(value) => decode_lifecycle(value)
                .map_or(Hydrated::Failed(HydrationError::Error), Hydrated::Partial),
            Hydrated::NotFound => Hydrated::NotFound,
            Hydrated::Failed(error) => Hydrated::Failed(error),
        };
        (id, lifecycle)
    });
    HydrationBatch::from_hydrated(decoded.collect())
}

pub(crate) trait Observer: Send + Sync {
    fn answered<X: Exchange>(&self, answers: impl Iterator<Item = (X::Key, Hydrated<X::Wire>)>);

    fn asked<X: Exchange>(&self, fields: &[QueryFields]);
}

impl Observer for () {
    fn answered<X: Exchange>(&self, _: impl Iterator<Item = (X::Key, Hydrated<X::Wire>)>) {}

    fn asked<X: Exchange>(&self, _: &[QueryFields]) {}
}

fn seen<V, W, E>(
    result: Option<&Result<V, E>>,
    wire: impl FnOnce(&V) -> Hydrated<W>,
) -> Hydrated<W> {
    match result {
        Some(Ok(value)) => wire(value),
        Some(Err(_)) | None => Hydrated::Failed(HydrationError::Error),
    }
}

fn found<W: Clone>(value: Option<&W>) -> Hydrated<W> {
    value.cloned().map_or(Hydrated::NotFound, Hydrated::Found)
}

pub(crate) struct Backends {
    pub(crate) tes: Arc<dyn TESClient + Send + Sync>,
    pub(crate) tweets: TweetSource,
    pub(crate) gizmoduck: GizmoduckLookup,
    pub(crate) socialgraph: Arc<dyn SocialgraphClient + Send + Sync>,
    pub(crate) about_this_account: AboutThisAccountClient,
    pub(crate) wingman: Arc<dyn WingmanClient>,
    pub(crate) articles: ArticleClient,
    pub(crate) trusted_friends: TrustedFriendsClient,
    pub(crate) user_location: UserLocationClient,
    pub(crate) safety_labels: Arc<SafetyLabelSource>,
    pub(crate) communities: CommunitySource,
}

pub(crate) struct ProdSources<O = ()> {
    backends: Backends,
    observer: O,
}

impl ProdSources {
    pub(crate) fn new(backends: Backends) -> Self {
        Self {
            backends,
            observer: (),
        }
    }

    pub(crate) fn observed<O: Observer>(self, observer: O) -> ProdSources<O> {
        ProdSources {
            backends: self.backends,
            observer,
        }
    }
}

const TEST_USERS_DTAB: &str = "/s/gizmoduck/test-users-permanent=>/s/gizmoduck/gizmoduck";

pub fn test_users_metadata() -> MetadataMap {
    let mut metadata = MetadataMap::new();
    metadata.insert("dtab-local", MetadataValue::from_static(TEST_USERS_DTAB));
    metadata
}

impl<O: Observer> ProdSources<O> {
    fn observe<X: Exchange>(
        &self,
        answers: impl Iterator<Item = (X::Key, Hydrated<X::Wire>)>,
    ) -> Observed<X, ()> {
        Observed::new(&self.observer, answers)
    }

    fn relay<X: Exchange, E>(
        &self,
        keys: impl Iterator<Item = X::Key> + Clone,
        answers: HashMap<X::Key, Result<Option<X::Wire>, E>>,
    ) -> Observed<X, HydrationBatch<X::Key, X::Wire>>
    where
        X::Key: Eq + Hash,
        X::Wire: Clone,
    {
        self.observe::<X>(keys.clone().map(|key| {
            let answer = seen(answers.get(&key), |wire| found(wire.as_ref()));
            (key, answer)
        }))
        .map(|()| HydrationBatch::from_results(keys, answers))
    }
}

#[tonic::async_trait]
impl<O: Observer> Sources for ProdSources<O> {
    async fn pure_cores(&self, tweet_ids: &[u64]) -> Answered<exchange::TesPureCoreData, PureCore> {
        let cores = self
            .backends
            .tes
            .get_tweet_core_datas(tweet_ids.to_vec())
            .await;
        self.relay::<exchange::TesPureCoreData, _>(tweet_ids.iter().copied(), cores)
            .map(core_batch)
    }

    async fn conversation_roots(
        &self,
        root_ids: &[u64],
    ) -> Answered<exchange::TesConversationRoot, PureCore> {
        let cores = self
            .backends
            .tes
            .get_tweet_core_datas(root_ids.to_vec())
            .await;
        self.relay::<exchange::TesConversationRoot, _>(root_ids.iter().copied(), cores)
            .map(core_batch)
    }

    async fn tweets(&self, tweet_ids: &[u64]) -> Answered<exchange::TesTweet, TweetFeatures> {
        let values = self.backends.tweets.get_tweet_values(tweet_ids).await;
        self.observe::<exchange::TesTweet>(tweet_ids.iter().map(|&id| {
            let answer = seen(values.get(&id), |bytes| {
                Hydrated::Found(Bytes(bytes.to_vec()))
            });
            (id, answer)
        }))
        .map(|()| tweet_batch(tweet_ids, values))
    }

    async fn conversation_controls(
        &self,
        tweet_ids: &[u64],
    ) -> Answered<exchange::TesConversationControl, ConversationControl> {
        let controls = self
            .backends
            .tes
            .get_conversation_controls(tweet_ids.to_vec())
            .await;
        self.relay::<exchange::TesConversationControl, _>(tweet_ids.iter().copied(), controls)
    }

    async fn safety_labels(
        &self,
        tweet_ids: &[u64],
    ) -> Answered<exchange::SafetyLabels, Arc<vf_pb::SafetyLabelMap>> {
        let labels = self.backends.safety_labels.get(tweet_ids).await;
        self.observe::<exchange::SafetyLabels>(tweet_ids.iter().map(|&id| {
            let answer = seen(labels.get(&id), |labels| {
                Hydrated::Found(Bytes(labels.encode_to_vec()))
            });
            (id, answer)
        }))
        .map(|()| label_batch(tweet_ids, labels))
    }

    async fn viewer(
        &self,
        viewer_id: u64,
        fields: &[QueryFields],
    ) -> Answered<exchange::GizmoduckViewer, DecodedViewer> {
        let viewer = self
            .backends
            .gizmoduck
            .get_viewer(viewer_id, fields)
            .await
            .inspect_err(|error| warn!(%error, "Gizmoduck viewer lookup failed; failing open"));
        self.observer.asked::<exchange::GizmoduckViewer>(fields);
        self.observe::<exchange::GizmoduckViewer>(iter::once_with(|| {
            (viewer_id, seen(Some(&viewer), |user| found(user.as_ref())))
        }))
        .map(|()| viewer_batch(viewer_id, viewer, fields))
    }

    async fn users(
        &self,
        user_ids: &[u64],
        fields: &[QueryFields],
    ) -> Answered<exchange::GizmoduckAuthor, DecodedAuthor> {
        let users = self
            .backends
            .gizmoduck
            .get_users(user_ids.to_vec(), fields)
            .await;
        self.observer.asked::<exchange::GizmoduckAuthor>(fields);
        self.relay::<exchange::GizmoduckAuthor, _>(user_ids.iter().copied(), users)
            .map(author_batch)
    }

    async fn select_edges(
        &self,
        viewer_id: u64,
        queries: &[EdgeQuery],
    ) -> Observed<exchange::Flock, Vec<RawHydrationBatch<bool>>> {
        let sets = self
            .backends
            .socialgraph
            .select_edges(viewer_id, queries)
            .await;
        let edges = edge_batches(queries, sets.as_deref());
        self.observe::<exchange::Flock>(queries.iter().zip(&edges).flat_map(|(query, batch)| {
            query.destination_ids.iter().map(move |&destination| {
                let answer = batch
                    .hydrated(&destination)
                    .cloned()
                    .unwrap_or(Hydrated::Failed(HydrationError::Error));
                (EdgeKey::of(query, destination), answer)
            })
        }))
        .map(|()| edges)
    }

    async fn viewer_country(&self, viewer_id: u64) -> Answered<exchange::ViewerCountry, Arc<str>> {
        let country = self
            .backends
            .about_this_account
            .tfe_top_country(viewer_id)
            .await
            .inspect_err(|error| warn!(%error, "tfe_top_country lookup failed"));
        self.observe::<exchange::ViewerCountry>(iter::once_with(|| {
            (
                viewer_id,
                seen(Some(&country), |country| found(country.as_ref())),
            )
        }))
        .map(|()| country_batch(viewer_id, country))
    }

    async fn second_degree(
        &self,
        viewer_id: u64,
        root_author_ids: &[u64],
    ) -> Answered<exchange::Wingman, bool> {
        let answers = self
            .backends
            .wingman
            .batch_exists_intersect(viewer_id, root_author_ids)
            .await;
        let answers = root_author_ids
            .iter()
            .copied()
            .zip(answers.into_iter().flatten())
            .map(|(root, answer)| {
                let answer = match answer {
                    Exists::Found => Ok(Some(true)),
                    Exists::NotFound => Ok(Some(false)),
                    Exists::Incomplete | Exists::ItemError => Err(answer),
                };
                (root, answer)
            })
            .collect();
        self.relay::<exchange::Wingman, _>(root_author_ids.iter().copied(), answers)
    }

    async fn community_moderations(
        &self,
        posts: &[CommunityPost],
    ) -> Answered<exchange::CommunityModeration, CommunityModeration> {
        let moderations = self.backends.communities.moderations(posts).await;
        self.relay::<exchange::CommunityModeration, _>(
            posts.iter().map(|post| post.tweet_id),
            moderations,
        )
    }

    async fn community_moderators(
        &self,
        viewer_id: u64,
        community_ids: &[u64],
    ) -> Answered<exchange::CommunityModerator, bool> {
        let moderators = self
            .backends
            .communities
            .moderators(viewer_id, community_ids)
            .await;
        self.relay::<exchange::CommunityModerator, _>(community_ids.iter().copied(), moderators)
    }

    async fn community_viewer_removals(
        &self,
        viewer_id: u64,
        community_ids: &[u64],
    ) -> Answered<exchange::CommunityViewerRemoved, bool> {
        let removals = self
            .backends
            .communities
            .viewer_removals(viewer_id, community_ids)
            .await;
        self.relay::<exchange::CommunityViewerRemoved, _>(community_ids.iter().copied(), removals)
    }

    async fn article_media_entities(
        &self,
        viewer_id: Option<u64>,
        article_ids: &[u64],
    ) -> Answered<exchange::ArticleMediaEntities, bool> {
        let rows = self
            .backends
            .articles
            .media_entities(viewer_id, article_ids)
            .await;
        self.relay::<exchange::ArticleMediaEntities, _>(article_ids.iter().copied(), rows)
    }

    async fn article_cover_media(
        &self,
        viewer_id: Option<u64>,
        article_ids: &[u64],
    ) -> Answered<exchange::ArticleCoverMedia, bool> {
        let rows = self
            .backends
            .articles
            .cover_media(viewer_id, article_ids)
            .await;
        self.relay::<exchange::ArticleCoverMedia, _>(article_ids.iter().copied(), rows)
    }

    async fn article_lifecycles(
        &self,
        article_ids: &[u64],
    ) -> Answered<exchange::ArticleLifecycle, ArticleLifecycle> {
        let rows = self.backends.articles.lifecycles(article_ids).await;
        self.relay::<exchange::ArticleLifecycle, _>(article_ids.iter().copied(), rows)
            .map(lifecycle_batch)
    }

    async fn trusted_friends(
        &self,
        viewer_id: u64,
        list_ids: &[u64],
    ) -> Answered<exchange::TrustedFriends, bool> {
        let answers = self
            .backends
            .trusted_friends
            .batch_is_member_or_owner(viewer_id, list_ids)
            .await;
        let answers = list_ids
            .iter()
            .copied()
            .zip(answers)
            .map(|(list_id, answer)| (list_id, answer.map(Some)))
            .collect();
        self.relay::<exchange::TrustedFriends, _>(list_ids.iter().copied(), answers)
    }

    async fn outside_places(
        &self,
        viewer_id: u64,
        place_ids: &[u64],
    ) -> Answered<exchange::UserLocation, bool> {
        let places = self
            .backends
            .user_location
            .places(viewer_id)
            .await
            .inspect_err(|error| warn!(%error, "Geoduck userLocation lookup failed"));
        let answers = place_ids
            .iter()
            .map(|&place| {
                let outside = places.as_ref().map(|places| Some(!places.contains(&place)));
                (place, outside)
            })
            .collect();
        self.relay::<exchange::UserLocation, _>(place_ids.iter().copied(), answers)
    }

    async fn post_card(&self, tweet_id: u64) -> Option<PostCard> {
        let (core, media) = join(
            self.backends.tes.get_tweet_core_datas(vec![tweet_id]),
            self.backends.tes.get_tweet_media_entities(vec![tweet_id]),
        )
        .await;
        let core = only(core)?;
        let author = only(
            self.backends
                .gizmoduck
                .get_users(vec![core.author_id], &[QueryFields::PROFILE])
                .await,
        )
        .and_then(|result| result.user);
        Some(PostCard::new(
            tweet_id,
            core,
            author.as_ref().map(|user| &user.profile),
            &only(media).unwrap_or_default(),
        ))
    }
}

fn only<V, E>(answers: HashMap<u64, Result<Option<V>, E>>) -> Option<V> {
    answers.into_values().next()?.ok().flatten()
}

#[cfg(test)]
pub(crate) use in_memory::{control, found_user, suspended, Fault, InMemorySources};

#[cfg(test)]
mod in_memory;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clients::socialgraph_client::Graph;
    use tonic::Status;

    #[test]
    fn a_select_that_ran_out_of_its_forwarded_deadline_times_out_every_key() {
        let queries = [EdgeQuery::forward(Graph::Follows, vec![1, 2])];
        let client_timer = Status::from_error(Box::new(tonic::TimeoutExpired(())));
        for (status, error) in [
            (client_timer, HydrationError::Timeout),
            (Status::deadline_exceeded(""), HydrationError::Timeout),
            (Status::unavailable(""), HydrationError::Error),
        ] {
            let batches = edge_batches(&queries, Err(&status));
            let failed = Some(Hydrated::Failed(error));
            assert_eq!(
                [1, 2].map(|key| batches[0].hydrated(&key).cloned()),
                [failed.clone(), failed],
                "{status:?}"
            );
        }
    }
}
