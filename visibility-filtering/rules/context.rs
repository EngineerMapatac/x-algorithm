use crate::hydration::{Hydrator, Hydrators};
use crate::models::{
    tweet_timestamp_ms, ClientCapability, PostCandidate, PostEntity, UserEntity, ViewerFeatures,
};
use crate::params::{CountryList, CountryLists};
use crate::rules::rule_spec::features::Feature;

pub(crate) struct RuleContext<'a> {
    facts: CoreFacts<'a>,
    viewer: &'a ViewerFeatures,
    candidate: &'a PostCandidate,
    #[cfg(test)]
    hydrated: Hydrators,
}

#[derive(Clone, Copy)]
pub(super) struct CoreFacts<'a> {
    viewer_id: Option<u64>,
    request_country: Option<&'a str>,
    client_capability: ClientCapability,
    country_lists: &'a CountryLists,
}

#[derive(Clone, Copy)]
pub(super) struct PostCoreFacts<'a>(&'a PostCandidate);

impl<'a> RuleContext<'a> {
    pub(super) fn new(
        viewer: &'a ViewerFeatures,
        candidate: &'a PostCandidate,
        country_lists: &'a CountryLists,
    ) -> Self {
        Self {
            facts: CoreFacts {
                viewer_id: viewer.viewer.user_id(),
                request_country: viewer.country_code.as_deref(),
                client_capability: viewer.client_capability,
                country_lists,
            },
            viewer,
            candidate,
            #[cfg(test)]
            hydrated: Hydrators::all(),
        }
    }

    #[cfg(test)]
    pub(super) fn hydrated_by(self, hydrated: Hydrators) -> Self {
        Self { hydrated, ..self }
    }

    #[inline]
    fn reads(&self, required: Hydrators) -> &Self {
        #[cfg(test)]
        for hydrator in required.iter() {
            assert!(
                self.hydrated.contains(hydrator),
                "a rule reads {hydrator:?}, which its policy does not derive"
            );
        }
        #[cfg(not(test))]
        let _ = required;
        self
    }

    #[inline]
    pub(super) fn facts(&self) -> CoreFacts<'a> {
        self.facts
    }

    #[inline]
    pub(super) fn failed(&self) -> Hydrators {
        self.candidate.failed
    }

    #[inline]
    pub(super) fn acquire<F: Feature>(&self) -> F::Value<'a> {
        F::read(At(self.reads(F::REQUIRED)))
    }

    #[inline]
    pub(super) fn acquire_edge(&self, node: Hydrator) -> bool {
        At(self.reads(Hydrators::of(node))).edge(node)
    }
}

#[derive(Clone, Copy)]
pub(super) struct At<'c, 'a>(&'c RuleContext<'a>);

impl<'a> At<'_, 'a> {
    #[inline]
    pub(super) fn post(self) -> &'a PostEntity {
        &self.0.candidate.post
    }

    #[inline]
    pub(super) fn post_facts(self) -> PostCoreFacts<'a> {
        PostCoreFacts(self.0.candidate)
    }

    #[inline]
    pub(super) fn author(self) -> &'a UserEntity {
        &self.0.candidate.author
    }

    #[inline]
    pub(super) fn viewer(self) -> &'a ViewerFeatures {
        self.0.viewer
    }

    #[inline]
    pub(super) fn edge(self, node: Hydrator) -> bool {
        debug_assert!(node.is_edge(), "{node:?} is not an edge node");
        let candidate = self.0.candidate;
        debug_assert!(candidate
            .post
            .edges
            .iter()
            .all(|edge| candidate.viewer_relationship.get(edge).is_none()));
        candidate
            .viewer_relationship
            .get(node)
            .unwrap_or_else(|| candidate.post.edges.contains(node))
    }
}

impl<'a> CoreFacts<'a> {
    #[inline]
    pub(super) fn viewer_id(self) -> Option<u64> {
        self.viewer_id
    }

    #[inline]
    pub(super) fn request_country(self) -> Option<&'a str> {
        self.request_country
    }

    #[inline]
    pub(super) fn client_capability(self) -> ClientCapability {
        self.client_capability
    }

    #[inline]
    pub(super) fn in_country_list(self, list: CountryList, country_code: &str) -> bool {
        self.country_lists.contains(list, country_code)
    }
}

impl PostCoreFacts<'_> {
    #[inline]
    pub(super) fn tweet_id(self) -> u64 {
        self.0.post.tweet_id
    }

    #[inline]
    pub(super) fn author_id(self) -> u64 {
        self.0.author.id
    }

    #[inline]
    pub(super) fn is_reply(self) -> bool {
        self.0.post.is_reply
    }

    #[inline]
    pub(super) fn created_after(self, unix_ms: u64) -> bool {
        tweet_timestamp_ms(self.0.post.tweet_id) > unix_ms
    }
}
