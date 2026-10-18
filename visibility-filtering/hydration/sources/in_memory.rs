use super::*;
use crate::clients::socialgraph_client::{EdgeDirection, Graph};
use crate::hydration::inventory::Source;
use std::iter;
use std::num::NonZeroU64;
use std::sync::Mutex;
use std::time::Duration;
use tokio::time::{sleep, Instant};
use xai_core_entities::entities::{
    ConversationControlArm, GizmoduckUser, GizmoduckUserResult, PureCoreData, Safety,
    UserResponseState,
};

pub(crate) fn found_user(user_id: u64) -> GizmoduckUserResult {
    GizmoduckUserResult {
        user: Some(GizmoduckUser {
            user_id,
            ..Default::default()
        }),
        response_state: Some(UserResponseState::Found),
    }
}

pub(crate) fn suspended() -> GizmoduckUserResult {
    GizmoduckUserResult {
        user: Some(GizmoduckUser {
            safety: Safety {
                suspended: true,
                ..Default::default()
            },
            ..Default::default()
        }),
        response_state: Some(UserResponseState::Found),
    }
}

pub(crate) fn control(
    arm: ConversationControlArm,
    root: u64,
    countries: &[&str],
) -> ConversationControl {
    ConversationControl {
        arm,
        conversation_tweet_author_id: root,
        invited_user_ids: vec![],
        invite_via_mention: None,
        allowed_country_codes: countries.iter().map(|c| (*c).to_owned()).collect(),
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum Fault {
    Fails,
    Hangs,
    Delays(std::time::Duration),
    AnswersPartial,
}

#[derive(Default)]
pub(crate) struct InMemorySources {
    pure_cores: HashMap<u64, PureCoreData>,
    tweets: HashMap<u64, TweetFeatures>,
    controls: HashMap<u64, ConversationControl>,
    labels: HashMap<u64, Arc<vf_pb::SafetyLabelMap>>,
    viewers: HashMap<u64, GizmoduckUser>,
    users: HashMap<u64, GizmoduckUserResult>,
    edges: HashSet<(Graph, u64, u64)>,
    countries: HashMap<u64, Arc<str>>,
    second_degree: HashSet<(u64, u64)>,
    community_moderations: HashMap<u64, CommunityModeration>,
    moderated_communities: HashSet<u64>,
    removed_from_communities: HashSet<u64>,
    community_posts: Mutex<Vec<CommunityPost>>,
    article_media_entities: HashMap<u64, bool>,
    article_cover_media: HashMap<u64, bool>,
    lifecycles: HashMap<u64, ArticleLifecycle>,
    trusted_friends: HashSet<(u64, u64)>,
    viewer_places: HashSet<u64>,
    faults: Mutex<Vec<(Source, Fault)>>,
    failed_keys: HashSet<(Source, u64)>,
    latencies: HashMap<Source, Duration>,
    key_latencies: HashMap<(Source, u64), Duration>,
    failed_graphs: HashSet<Graph>,
    failed_edges: HashSet<(Graph, u64)>,
    hung_graphs: HashSet<Graph>,
    missing_graphs: HashSet<Graph>,
    calls: Mutex<Vec<(Source, Vec<u64>)>>,
    article_viewers: Mutex<Vec<(Source, Option<u64>)>>,
    starts: Mutex<Vec<(Source, Instant)>>,
    selects: Mutex<Vec<Vec<EdgeQuery>>>,
    fields: Mutex<Vec<(Source, Vec<QueryFields>)>>,
}

impl InMemorySources {
    pub(crate) fn tweet(self, tweet_id: u64, author_id: u64) -> Self {
        self.pure_core(
            tweet_id,
            PureCoreData {
                author_id,
                ..Default::default()
            },
        )
    }

    pub(crate) fn pure_core(mut self, tweet_id: u64, core: PureCoreData) -> Self {
        self.pure_cores.insert(tweet_id, core);
        self.tweets.entry(tweet_id).or_default();
        self
    }

    pub(crate) fn retweet(
        self,
        tweet_id: u64,
        author_id: u64,
        source_tweet_id: u64,
        source_user_id: Option<u64>,
    ) -> Self {
        let core = PureCoreData {
            author_id,
            source_tweet_id: Some(source_tweet_id),
            source_user_id,
            ..Default::default()
        };
        self.pure_core(tweet_id, core)
    }

    pub(crate) fn without_tweet_row(mut self, tweet_id: u64) -> Self {
        self.tweets.remove(&tweet_id);
        self
    }

    pub(crate) fn tweet_features(mut self, tweet_id: u64, tweet: TweetFeatures) -> Self {
        self.tweets.insert(tweet_id, tweet);
        self
    }

    pub(crate) fn community_post(mut self, tweet_id: u64, community_id: u64) -> Self {
        self.tweets.entry(tweet_id).or_default().community_id = NonZeroU64::new(community_id);
        self
    }

    pub(crate) fn control(mut self, tweet_id: u64, control: ConversationControl) -> Self {
        self.controls.insert(tweet_id, control);
        self
    }

    pub(crate) fn labels(mut self, tweet_id: u64, labels: vf_pb::SafetyLabelMap) -> Self {
        self.labels.insert(tweet_id, Arc::new(labels));
        self
    }

    pub(crate) fn viewer(mut self, viewer_id: u64, user: GizmoduckUser) -> Self {
        self.viewers.insert(viewer_id, user);
        self
    }

    pub(crate) fn user(mut self, user_id: u64, user: GizmoduckUserResult) -> Self {
        self.users.insert(user_id, user);
        self
    }

    pub(crate) fn authors(self, user_ids: &[u64]) -> Self {
        user_ids.iter().fold(self, |sources, &user_id| {
            sources.user(user_id, found_user(user_id))
        })
    }

    pub(crate) fn edge(mut self, graph: Graph, source: u64, destination: u64) -> Self {
        self.edges.insert((graph, source, destination));
        self
    }

    pub(crate) fn country(mut self, viewer_id: u64, country: &str) -> Self {
        self.countries.insert(viewer_id, Arc::from(country));
        self
    }

    pub(crate) fn second_degree_path(mut self, root_author: u64, viewer_id: u64) -> Self {
        self.second_degree.insert((root_author, viewer_id));
        self
    }

    pub(crate) fn community_moderation(
        mut self,
        tweet_id: u64,
        moderation: CommunityModeration,
    ) -> Self {
        self.community_moderations.insert(tweet_id, moderation);
        self
    }

    pub(crate) fn moderator_of(mut self, community_id: u64) -> Self {
        self.moderated_communities.insert(community_id);
        self
    }

    pub(crate) fn removed_from(mut self, community_id: u64) -> Self {
        self.removed_from_communities.insert(community_id);
        self
    }

    pub(crate) fn community_posts(&self) -> Vec<CommunityPost> {
        self.community_posts.lock().unwrap().clone()
    }

    pub(crate) fn article_media_entities(mut self, article_id: u64, present: bool) -> Self {
        self.article_media_entities.insert(article_id, present);
        self
    }

    pub(crate) fn article_cover_media(mut self, article_id: u64, present: bool) -> Self {
        self.article_cover_media.insert(article_id, present);
        self
    }

    pub(crate) fn lifecycle(mut self, article_id: u64, lifecycle: ArticleLifecycle) -> Self {
        self.lifecycles.insert(article_id, lifecycle);
        self
    }

    pub(crate) fn trusted_friend(mut self, list_id: u64, viewer_id: u64) -> Self {
        self.trusted_friends.insert((list_id, viewer_id));
        self
    }

    pub(crate) fn located_in(mut self, place_id: u64) -> Self {
        self.viewer_places.insert(place_id);
        self
    }

    pub(crate) fn fault(self, source: Source, fault: Fault) -> Self {
        self.break_source(source, fault);
        self
    }

    pub(crate) fn break_source(&self, source: Source, fault: Fault) {
        self.faults.lock().unwrap().push((source, fault));
    }

    pub(crate) fn fail_graph(mut self, graph: Graph) -> Self {
        self.failed_graphs.insert(graph);
        self
    }

    pub(crate) fn fail_edge(mut self, graph: Graph, destination: u64) -> Self {
        self.failed_edges.insert((graph, destination));
        self
    }

    pub(crate) fn hang_graph(mut self, graph: Graph) -> Self {
        self.hung_graphs.insert(graph);
        self
    }

    pub(crate) fn miss_graph(mut self, graph: Graph) -> Self {
        self.missing_graphs.insert(graph);
        self
    }

    pub(crate) fn fail_key(mut self, source: Source, key: u64) -> Self {
        self.failed_keys.insert((source, key));
        self
    }

    pub(crate) fn latency(mut self, source: Source, latency: Duration) -> Self {
        self.latencies.insert(source, latency);
        self
    }

    pub(crate) fn key_latency(mut self, source: Source, key: u64, latency: Duration) -> Self {
        self.key_latencies.insert((source, key), latency);
        self
    }

    pub(crate) fn calls(&self) -> Vec<Source> {
        self.calls
            .lock()
            .unwrap()
            .iter()
            .map(|(source, _)| *source)
            .collect()
    }

    pub(crate) fn starts(&self, source: Source) -> Vec<Instant> {
        self.starts
            .lock()
            .unwrap()
            .iter()
            .filter(|(called, _)| *called == source)
            .map(|(_, started)| *started)
            .collect()
    }

    pub(crate) fn article_viewers(&self, source: Source) -> Vec<Option<u64>> {
        self.article_viewers
            .lock()
            .unwrap()
            .iter()
            .filter(|(called, _)| *called == source)
            .map(|&(_, viewer_id)| viewer_id)
            .collect()
    }

    pub(crate) fn keys(&self, source: Source) -> Vec<Vec<u64>> {
        self.calls
            .lock()
            .unwrap()
            .iter()
            .filter(|(called, _)| *called == source)
            .map(|(_, keys)| keys.clone())
            .collect()
    }

    pub(crate) fn selects(&self) -> Vec<Vec<EdgeQuery>> {
        self.selects.lock().unwrap().clone()
    }

    pub(crate) fn fields(&self, source: Source) -> Vec<Vec<QueryFields>> {
        self.fields
            .lock()
            .unwrap()
            .iter()
            .filter(|(called, _)| *called == source)
            .map(|(_, fields)| fields.clone())
            .collect()
    }

    fn record_fields(&self, source: Source, fields: &[QueryFields]) {
        self.fields.lock().unwrap().push((source, fields.to_vec()));
    }

    async fn enter(&self, source: Source, keys: &[u64]) -> bool {
        let mut sorted = keys.to_vec();
        sorted.sort_unstable();
        self.calls.lock().unwrap().push((source, sorted));
        self.starts.lock().unwrap().push((source, Instant::now()));
        let latency = keys
            .iter()
            .filter_map(|&key| self.key_latencies.get(&(source, key)))
            .chain(self.latencies.get(&source))
            .max();
        if let Some(latency) = latency {
            sleep(*latency).await;
        }
        match self.fault_for(source) {
            Some(Fault::Hangs) => std::future::pending().await,
            Some(Fault::Delays(delay)) => {
                tokio::time::sleep(delay).await;
                false
            }
            Some(Fault::Fails) => true,
            Some(Fault::AnswersPartial) | None => false,
        }
    }

    fn fault_for(&self, source: Source) -> Option<Fault> {
        self.faults
            .lock()
            .unwrap()
            .iter()
            .find(|(faulty, _)| *faulty == source)
            .map(|(_, fault)| *fault)
    }

    async fn keyed<V: Clone>(
        &self,
        source: Source,
        ids: &[u64],
        values: &HashMap<u64, V>,
    ) -> RawHydrationBatch<V> {
        let fails = self.enter(source, ids).await;
        let results: FxHashMap<_, _> = ids
            .iter()
            .map(|&id| {
                let result = if fails || self.failed_keys.contains(&(source, id)) {
                    Err(())
                } else {
                    Ok(values.get(&id).cloned())
                };
                (id, result)
            })
            .collect();
        HydrationBatch::from_results(ids.iter().copied(), results)
    }
}

#[tonic::async_trait]
impl Sources for InMemorySources {
    async fn pure_cores(&self, tweet_ids: &[u64]) -> Answered<exchange::TesPureCoreData, PureCore> {
        let cores = self.keyed(Source::TesPureCoreData, tweet_ids, &self.pure_cores);
        Observed::quiet(cores.await.map(|core| pure_core(&core)))
    }

    async fn conversation_roots(
        &self,
        root_ids: &[u64],
    ) -> Answered<exchange::TesConversationRoot, PureCore> {
        let cores = self.keyed(Source::TesConversationRoot, root_ids, &self.pure_cores);
        Observed::quiet(cores.await.map(|core| pure_core(&core)))
    }

    async fn tweets(&self, tweet_ids: &[u64]) -> Answered<exchange::TesTweet, TweetFeatures> {
        Observed::quiet(self.keyed(Source::TesTweet, tweet_ids, &self.tweets).await)
    }

    async fn conversation_controls(
        &self,
        tweet_ids: &[u64],
    ) -> Answered<exchange::TesConversationControl, ConversationControl> {
        Observed::quiet(
            self.keyed(Source::TesConversationControl, tweet_ids, &self.controls)
                .await,
        )
    }

    async fn safety_labels(
        &self,
        tweet_ids: &[u64],
    ) -> Answered<exchange::SafetyLabels, Arc<vf_pb::SafetyLabelMap>> {
        Observed::quiet(
            self.keyed(Source::SafetyLabels, tweet_ids, &self.labels)
                .await,
        )
    }

    async fn viewer(
        &self,
        viewer_id: u64,
        fields: &[QueryFields],
    ) -> Answered<exchange::GizmoduckViewer, DecodedViewer> {
        self.record_fields(Source::GizmoduckViewer, fields);
        let viewer = HashMap::from([(viewer_id, self.viewers.get(&viewer_id).cloned())]);
        Observed::quiet(
            self.keyed(Source::GizmoduckViewer, &[viewer_id], &viewer)
                .await
                .map(|user| decode_viewer(user.as_ref(), fields)),
        )
    }

    async fn users(
        &self,
        user_ids: &[u64],
        fields: &[QueryFields],
    ) -> Answered<exchange::GizmoduckAuthor, DecodedAuthor> {
        self.record_fields(Source::GizmoduckAuthor, fields);
        let users = self
            .keyed(Source::GizmoduckAuthor, user_ids, &self.users)
            .await;
        let users = match self.fault_for(Source::GizmoduckAuthor) {
            Some(Fault::AnswersPartial) => users.map(|user| GizmoduckUserResult {
                response_state: Some(UserResponseState::Partial),
                ..user
            }),
            _ => users,
        };
        Observed::quiet(author_batch(users))
    }

    async fn select_edges(
        &self,
        viewer_id: u64,
        queries: &[EdgeQuery],
    ) -> Observed<exchange::Flock, Vec<RawHydrationBatch<bool>>> {
        let mut recorded = queries.to_vec();
        for query in &mut recorded {
            query.destination_ids.sort_unstable();
        }
        self.selects.lock().unwrap().push(recorded);
        let failed_graph = queries.iter().any(|query| {
            self.failed_graphs.contains(&query.graph)
                || query
                    .destination_ids
                    .iter()
                    .any(|&id| self.failed_edges.contains(&(query.graph, id)))
        });
        let keys: Vec<u64> = iter::once(viewer_id)
            .chain(
                queries
                    .iter()
                    .flat_map(|query| query.destination_ids.iter().copied()),
            )
            .collect();
        let fails = self.enter(Source::Flock, &keys).await || failed_graph;
        if queries
            .iter()
            .any(|query| self.hung_graphs.contains(&query.graph))
        {
            std::future::pending::<()>().await;
        }
        let answer = |query: &EdgeQuery| {
            let holds = |&id: &u64| {
                let edge = match query.direction {
                    EdgeDirection::Forward => (query.graph, viewer_id, id),
                    EdgeDirection::Reverse => (query.graph, id, viewer_id),
                };
                self.edges.contains(&edge)
            };
            (!self.missing_graphs.contains(&query.graph)).then(|| {
                query
                    .destination_ids
                    .iter()
                    .copied()
                    .filter(holds)
                    .collect()
            })
        };
        let sets: Result<Vec<_>, _> = (!fails)
            .then(|| queries.iter().map(answer).collect())
            .ok_or_else(|| tonic::Status::unavailable("select failed"));
        Observed::quiet(edge_batches(queries, sets.as_deref()))
    }

    async fn viewer_country(&self, viewer_id: u64) -> Answered<exchange::ViewerCountry, Arc<str>> {
        Observed::quiet(
            self.keyed(Source::ViewerCountry, &[viewer_id], &self.countries)
                .await,
        )
    }

    async fn second_degree(
        &self,
        viewer_id: u64,
        root_author_ids: &[u64],
    ) -> Answered<exchange::Wingman, bool> {
        let paths = root_author_ids
            .iter()
            .map(|&root| (root, self.second_degree.contains(&(root, viewer_id))))
            .collect();
        Observed::quiet(self.keyed(Source::Wingman, root_author_ids, &paths).await)
    }

    async fn community_moderations(
        &self,
        posts: &[CommunityPost],
    ) -> Answered<exchange::CommunityModeration, CommunityModeration> {
        self.community_posts
            .lock()
            .unwrap()
            .extend_from_slice(posts);
        let tweet_ids: Vec<u64> = posts.iter().map(|post| post.tweet_id).collect();
        let moderations = tweet_ids
            .iter()
            .map(|&id| {
                let moderation = self.community_moderations.get(&id).copied();
                (id, moderation.unwrap_or_default())
            })
            .collect();
        Observed::quiet(
            self.keyed(Source::CommunityModeration, &tweet_ids, &moderations)
                .await,
        )
    }

    async fn community_moderators(
        &self,
        _viewer_id: u64,
        community_ids: &[u64],
    ) -> Answered<exchange::CommunityModerator, bool> {
        let moderators = community_ids
            .iter()
            .map(|&id| (id, self.moderated_communities.contains(&id)))
            .collect();
        Observed::quiet(
            self.keyed(Source::CommunityModerator, community_ids, &moderators)
                .await,
        )
    }

    async fn community_viewer_removals(
        &self,
        _viewer_id: u64,
        community_ids: &[u64],
    ) -> Answered<exchange::CommunityViewerRemoved, bool> {
        let removals = community_ids
            .iter()
            .map(|&id| (id, self.removed_from_communities.contains(&id)))
            .collect();
        Observed::quiet(
            self.keyed(Source::CommunityViewerRemoved, community_ids, &removals)
                .await,
        )
    }

    async fn article_media_entities(
        &self,
        viewer_id: Option<u64>,
        article_ids: &[u64],
    ) -> Answered<exchange::ArticleMediaEntities, bool> {
        self.article_viewers
            .lock()
            .unwrap()
            .push((Source::ArticleMediaEntities, viewer_id));
        Observed::quiet(
            self.keyed(
                Source::ArticleMediaEntities,
                article_ids,
                &self.article_media_entities,
            )
            .await,
        )
    }

    async fn article_cover_media(
        &self,
        viewer_id: Option<u64>,
        article_ids: &[u64],
    ) -> Answered<exchange::ArticleCoverMedia, bool> {
        self.article_viewers
            .lock()
            .unwrap()
            .push((Source::ArticleCoverMedia, viewer_id));
        Observed::quiet(
            self.keyed(
                Source::ArticleCoverMedia,
                article_ids,
                &self.article_cover_media,
            )
            .await,
        )
    }

    async fn article_lifecycles(
        &self,
        article_ids: &[u64],
    ) -> Answered<exchange::ArticleLifecycle, ArticleLifecycle> {
        Observed::quiet(
            self.keyed(Source::ArticleLifecycle, article_ids, &self.lifecycles)
                .await,
        )
    }

    async fn trusted_friends(
        &self,
        viewer_id: u64,
        list_ids: &[u64],
    ) -> Answered<exchange::TrustedFriends, bool> {
        let lists = list_ids
            .iter()
            .map(|&list| (list, self.trusted_friends.contains(&(list, viewer_id))))
            .collect();
        Observed::quiet(self.keyed(Source::TrustedFriends, list_ids, &lists).await)
    }

    async fn outside_places(
        &self,
        _viewer_id: u64,
        place_ids: &[u64],
    ) -> Answered<exchange::UserLocation, bool> {
        let outside = place_ids
            .iter()
            .map(|&place| (place, !self.viewer_places.contains(&place)))
            .collect();
        Observed::quiet(self.keyed(Source::UserLocation, place_ids, &outside).await)
    }

    async fn post_card(&self, tweet_id: u64) -> Option<PostCard> {
        if matches!(self.fault_for(Source::TesPureCoreData), Some(Fault::Fails)) {
            return None;
        }
        let core = self.pure_cores.get(&tweet_id)?.clone();
        let author = self
            .users
            .get(&core.author_id)
            .and_then(|result| result.user.as_ref());
        Some(PostCard::new(
            tweet_id,
            core,
            author.map(|user| &user.profile),
            &[],
        ))
    }
}
