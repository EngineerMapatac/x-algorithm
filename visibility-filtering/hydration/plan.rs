use crate::clients::socialgraph_client::{EdgeDirection, Graph};
use crate::hydration::hydrator::{Edge, Part};
use crate::hydration::inventory::Source;
use crate::hydration::{Hydrator, Hydrators, BEST_EFFORT_HYDRATION_TIMEOUT, HYDRATION_TIMEOUT};
use crate::rules::SafetyLevel;
use std::fmt;
use std::time::Duration;
use strum::VariantArray;
use xai_core_entities::gizmoduck_client::QueryFields;

pub(crate) struct HydrationPlan {
    level: SafetyLevel,
    groups: Vec<Group>,
    nodes: Hydrators,
    logged_out_nodes: Hydrators,
}

pub(super) struct Group {
    pub(super) position: usize,
    pub(super) source: Source,
    pub(super) input: Option<Hydrator>,
    pub(super) nodes: Hydrators,
    clients: Vec<&'static str>,
    methods: Vec<&'static str>,
    edges: Vec<(Edge, Graph, EdgeDirection, Hydrators)>,
    fields: Vec<QueryFields>,
    readers: Vec<usize>,
}

impl HydrationPlan {
    pub(crate) fn new(level: SafetyLevel, hydrators: Hydrators) -> Self {
        let nodes = hydrators.closed();
        let mut groups: Vec<Group> = Vec::new();
        for node in nodes.iter() {
            let spec = node.spec();
            let (client, method) = spec.label;
            let input = node.input();
            match groups
                .iter_mut()
                .find(|group| group.source == spec.source && group.input == input)
            {
                Some(group) => {
                    group.nodes = group.nodes.with(node);
                    if !group.clients.contains(&client) {
                        group.clients.push(client);
                    }
                    if !group.methods.contains(&method) {
                        group.methods.push(method);
                    }
                }
                None => groups.push(Group {
                    position: groups.len(),
                    source: spec.source,
                    input,
                    nodes: Hydrators::of(node),
                    clients: vec![client],
                    methods: vec![method],
                    edges: Vec::new(),
                    fields: Vec::new(),
                    readers: Vec::new(),
                }),
            }
        }
        let reads: Vec<Hydrators> = groups
            .iter()
            .map(|group| {
                group.nodes.iter().fold(Hydrators::empty(), |reads, node| {
                    reads.union(node.spec().key.reads())
                })
            })
            .collect();
        let readers: Vec<Vec<usize>> = groups
            .iter()
            .map(|landed| {
                reads
                    .iter()
                    .enumerate()
                    .filter(|(_, reads)| !reads.intersection(landed.nodes).is_empty())
                    .map(|(position, _)| position)
                    .collect()
            })
            .collect();
        for (group, readers) in groups.iter_mut().zip(readers) {
            group.edges = edges(group.nodes);
            group.fields = fields(group.source, group.nodes);
            group.readers = readers;
        }
        Self {
            level,
            groups,
            nodes,
            logged_out_nodes: nodes
                .iter()
                .filter(|node| !node.needs_viewer())
                .fold(Hydrators::empty(), Hydrators::with),
        }
    }

    pub(crate) fn level(&self) -> SafetyLevel {
        self.level
    }

    pub(super) fn reads_viewer_profile(&self) -> bool {
        self.nodes.contains(Hydrator::ViewerProfile)
    }

    pub(super) fn reads_viewer_labels(&self) -> bool {
        self.nodes.contains(Hydrator::ViewerLabels)
    }

    pub(super) fn callable(&self, viewer_id: Option<u64>) -> Hydrators {
        match viewer_id {
            Some(_) => self.nodes,
            None => self.logged_out_nodes,
        }
    }

    pub(super) fn groups(&self) -> impl Iterator<Item = &Group> {
        self.groups.iter()
    }

    pub(super) fn readers<'a>(&'a self, landed: &'a Group) -> impl Iterator<Item = &'a Group> {
        landed
            .readers
            .iter()
            .filter_map(|&position| self.groups.get(position))
    }
}

impl Group {
    pub(super) fn ceiling(&self) -> Duration {
        if self.nodes.has_must_have() {
            HYDRATION_TIMEOUT
        } else {
            BEST_EFFORT_HYDRATION_TIMEOUT
        }
    }

    pub(super) fn label(&self) -> (String, String) {
        (self.clients.join("+"), self.methods.join("+"))
    }

    pub(super) fn fields(&self) -> &[QueryFields] {
        &self.fields
    }

    pub(super) fn edges(&self) -> &[(Edge, Graph, EdgeDirection, Hydrators)] {
        &self.edges
    }
}

fn fields(source: Source, nodes: Hydrators) -> Vec<QueryFields> {
    let nodes = match source {
        Source::GizmoduckAuthor => Hydrator::VARIANTS
            .iter()
            .copied()
            .filter(|node| node.spec().source == source)
            .fold(Hydrators::empty(), Hydrators::with),
        Source::TesPureCoreData
        | Source::TesConversationRoot
        | Source::TesTweet
        | Source::TesConversationControl
        | Source::SafetyLabels
        | Source::GizmoduckViewer
        | Source::Flock
        | Source::ViewerCountry
        | Source::Wingman
        | Source::CommunityModeration
        | Source::CommunityModerator
        | Source::CommunityViewerRemoved
        | Source::ArticleLifecycle
        | Source::ArticleMediaEntities
        | Source::ArticleCoverMedia
        | Source::TrustedFriends
        | Source::UserLocation => nodes,
    };
    let mut fields = Vec::new();
    for node in nodes.iter() {
        if let Part::Fields(node_fields) = node.spec().part {
            for field in node_fields {
                if !fields.contains(field) {
                    fields.push(*field);
                }
            }
        }
    }
    fields
}

fn edges(nodes: Hydrators) -> Vec<(Edge, Graph, EdgeDirection, Hydrators)> {
    let mut edges: Vec<(Edge, Graph, EdgeDirection, Hydrators)> = Vec::new();
    for node in nodes.iter() {
        let Some(edge) = node.edge() else {
            continue;
        };
        let Some((graph, direction)) = edge.flock() else {
            continue;
        };
        match edges.iter_mut().find(|(queried, ..)| *queried == edge) {
            Some((.., nodes)) => *nodes = nodes.with(node),
            None => edges.push((edge, graph, direction, Hydrators::of(node))),
        }
    }
    edges
}

impl fmt::Display for HydrationPlan {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "{}: {} calls",
            <&str>::from(self.level),
            self.groups.len()
        )?;
        for group in &self.groups {
            let (client, method) = group.label();
            let after = group.input.map_or("-", <&str>::from);
            let nodes: Vec<&str> = group.nodes.iter().map(<&str>::from).collect();
            write!(
                f,
                "{client}/{method} after: {after} nodes: {}",
                nodes.join(",")
            )?;
            let fields = group.fields();
            if !fields.is_empty() {
                let fields: Vec<String> = fields.iter().map(|field| format!("{field:?}")).collect();
                write!(f, " fields: {}", fields.join("|"))?;
            }
            for &(_, graph, direction, nodes) in group.edges() {
                let direction = match direction {
                    EdgeDirection::Forward => "fwd",
                    EdgeDirection::Reverse => "rev",
                };
                let keys: Vec<String> = nodes
                    .iter()
                    .map(|node| node.spec().key.to_string())
                    .collect();
                write!(
                    f,
                    " {}-{direction}[{}]",
                    <&str>::from(graph),
                    keys.join(",")
                )?;
            }
            if group.nodes.iter().all(Hydrator::needs_viewer) {
                f.write_str(" (skipped logged out)")?;
            }
            if group.ceiling() != HYDRATION_TIMEOUT {
                write!(f, " timeout: {:?}", group.ceiling())?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::rules::{RuleEngine, SafetyLevel};
    use strum::VariantArray;

    const PLANS: &str = "\
filter_all: 1 calls
tes/get_tweet_core_datas after: - nodes: pure_core
timeline_home: 10 calls
tes/get_tweet_core_datas after: - nodes: pure_core
tes/get_tweets after: - nodes: tweet
safety_labels/get after: - nodes: tweet_safety_labels
gizmoduck/get_viewer_data after: - nodes: viewer_profile fields: ACCOUNT|EXTENDED_PROFILE|SAFETY (skipped logged out)
gizmoduck/get_users after: pure_core nodes: author_safety fields: SAFETY|LABELS
socialgraph/batch_check_relationships after: pure_core nodes: follows,blocks,mutes,mute_retweets follows-fwd[author] blocks-fwd[author] mutes-fwd[author] mute_retweets-fwd[retweeter] (skipped logged out)
exclusive_content/batch_check_super_follows after: tweet nodes: super_follows_exclusive super_follows-fwd[exclusive_author] (skipped logged out)
article/get_media_entities after: tweet nodes: article_media_entities
article/get_cover_media after: tweet nodes: article_cover_media
trusted_friends/is_member_or_owner after: tweet nodes: trusted_friends (skipped logged out)
timeline_home_recommendations: 10 calls
tes/get_tweet_core_datas after: - nodes: pure_core
tes/get_tweets after: - nodes: tweet
safety_labels/get after: - nodes: tweet_safety_labels
gizmoduck/get_viewer_data after: - nodes: viewer_profile fields: ACCOUNT|EXTENDED_PROFILE|SAFETY (skipped logged out)
gizmoduck/get_users after: pure_core nodes: author_safety,author_labels fields: SAFETY|LABELS
socialgraph/batch_check_relationships after: pure_core nodes: follows,blocks,mutes,mute_retweets follows-fwd[author] blocks-fwd[author] mutes-fwd[author] mute_retweets-fwd[retweeter] (skipped logged out)
exclusive_content/batch_check_super_follows after: tweet nodes: super_follows_exclusive super_follows-fwd[exclusive_author] (skipped logged out)
article/get_media_entities after: tweet nodes: article_media_entities
article/get_cover_media after: tweet nodes: article_cover_media
trusted_friends/is_member_or_owner after: tweet nodes: trusted_friends (skipped logged out)
timeline_home_hydration: 21 calls
tes/get_tweet_core_datas after: - nodes: pure_core
tes/get_tweets after: - nodes: tweet
conversation_control/get_conversation_controls after: - nodes: conversation_control timeout: 300ms
safety_labels/get after: - nodes: tweet_safety_labels
gizmoduck/get_viewer_data after: - nodes: viewer_profile,viewer_labels fields: ACCOUNT|EXTENDED_PROFILE|SAFETY|LABELS (skipped logged out)
gizmoduck/get_users after: pure_core nodes: author_safety fields: SAFETY|LABELS
socialgraph+blocked_by/batch_check_relationships+batch_check_blocked_by after: pure_core nodes: follows,blocked_by_author,blocked_by_reply_root follows-fwd[author] blocks-rev[author,reply_root] (skipped logged out) timeout: 300ms
exclusive_content/batch_check_super_follows after: tweet nodes: super_follows_exclusive super_follows-fwd[exclusive_author] (skipped logged out)
conversation_control/batch_check_followed_by+batch_check_super_follows+batch_check_blocked_by after: conversation_control nodes: root_follows_viewer,super_follows_root,blocked_by_conversation_root follows-rev[root:Community|MyNetwork] super_follows-fwd[root:Subscribers] blocks-rev[control_root_author] (skipped logged out) timeout: 300ms
conversation_control/exists_intersect after: root_follows_viewer nodes: root_follows_viewer_second_degree (skipped logged out) timeout: 300ms
conversation_root/get_tweet_core_datas after: pure_core nodes: conversation_root_post (skipped logged out) timeout: 300ms
conversation_root/batch_check_blocked_by after: conversation_root_post nodes: blocked_by_conversation_root_post blocks-rev[conversation_root_post_author] (skipped logged out) timeout: 300ms
conversation_control/tfe_top_country after: conversation_control nodes: viewer_country (skipped logged out) timeout: 300ms
communities/moderation_state after: tweet nodes: community_moderation
communities/visibility_features after: community_moderation nodes: community_moderator (skipped logged out)
communities/is_removed after: tweet nodes: community_viewer_removed (skipped logged out) timeout: 300ms
article/get_lifecycles after: tweet nodes: article_lifecycle
article/get_media_entities after: tweet nodes: article_media_entities
article/get_cover_media after: tweet nodes: article_cover_media
trusted_friends/is_member_or_owner after: tweet nodes: trusted_friends (skipped logged out)
geoduck/user_location after: tweet nodes: outside_narrowcast_place (skipped logged out) timeout: 300ms
immersive_expanded_recommendations: 10 calls
tes/get_tweet_core_datas after: - nodes: pure_core
tes/get_tweets after: - nodes: tweet
safety_labels/get after: - nodes: tweet_safety_labels
gizmoduck/get_viewer_data after: - nodes: viewer_profile fields: ACCOUNT|EXTENDED_PROFILE|SAFETY (skipped logged out)
gizmoduck/get_users after: pure_core nodes: author_safety,author_labels fields: SAFETY|LABELS
socialgraph/batch_check_relationships after: pure_core nodes: follows,blocks,mutes,mute_retweets follows-fwd[author] blocks-fwd[author] mutes-fwd[author] mute_retweets-fwd[retweeter] (skipped logged out)
exclusive_content/batch_check_super_follows after: tweet nodes: super_follows_exclusive super_follows-fwd[exclusive_author] (skipped logged out)
article/get_media_entities after: tweet nodes: article_media_entities
article/get_cover_media after: tweet nodes: article_cover_media
trusted_friends/is_member_or_owner after: tweet nodes: trusted_friends (skipped logged out)
";

    #[test]
    fn each_level_plans_these_calls() {
        let engine = RuleEngine::for_tests();
        let plans: String = SafetyLevel::VARIANTS
            .iter()
            .map(|&level| engine.plan(level).to_string())
            .collect();
        assert_eq!(plans, PLANS);
    }
}
