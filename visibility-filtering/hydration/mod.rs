pub mod batch;
pub(crate) mod community_source;
mod decode;
mod execute;
pub(crate) mod fallback_cache;
mod fetcher;
mod hydrator;
pub(crate) mod inventory;
pub mod metrics;
pub(crate) mod plan;
pub(crate) mod sources;
mod store;
pub mod tweet_source;

use crate::models::{
    ClientCapability, PostCandidate, RawCandidate, TweetFeatures, TweetId, ViewerFeatures,
};
pub(crate) use decode::author::fallback_cache as author_fallback_cache;
pub(crate) use decode::tweet::tweet_fallback_cache;
use decode::viewer::ViewerState;
use fallback_cache::FallbackCaches;
pub use hydrator::{Hydrator, Hydrators};
pub(crate) use plan::HydrationPlan;
use rustc_hash::FxHashMap;
use std::borrow::Cow;
use std::hash::Hash;
use std::sync::Arc;
use std::time::Duration;
use xai_visibility_filtering_proto as vf_pb;

pub(crate) const HYDRATION_TIMEOUT: Duration = Duration::from_secs(1);
pub(crate) const BEST_EFFORT_HYDRATION_TIMEOUT: Duration = Duration::from_millis(300);
pub(crate) const INBOUND_ALLOWANCE: Duration = Duration::from_millis(10);

pub(crate) fn request_context(
    entered: tokio::time::Instant,
    grpc_timeout: Option<Duration>,
) -> xai_x_rpc::CallContext {
    xai_x_rpc::CallContext {
        deadline: Some(
            entered
                + grpc_timeout
                    .unwrap_or(HYDRATION_TIMEOUT)
                    .saturating_sub(INBOUND_ALLOWANCE),
        ),
    }
}

pub(crate) struct HydrationRequest<'a> {
    viewer_id: Option<u64>,
    country_code: Option<String>,
    client_capability: ClientCapability,
    raw_candidates: &'a [RawCandidate],
    is_expanding_retweet_sources: bool,
    fallback_caches: &'a FallbackCaches,
}

impl<'a> HydrationRequest<'a> {
    pub(crate) fn new(
        viewer_id: Option<u64>,
        country_code: Option<String>,
        client_capability: ClientCapability,
        raw_candidates: &'a [RawCandidate],
        fallback_caches: &'a FallbackCaches,
    ) -> Self {
        Self {
            viewer_id,
            country_code,
            client_capability,
            raw_candidates,
            is_expanding_retweet_sources: false,
            fallback_caches,
        }
    }

    pub(crate) fn with_retweet_sources(mut self, is_expanding_retweet_sources: bool) -> Self {
        self.is_expanding_retweet_sources = is_expanding_retweet_sources;
        self
    }
}

pub(crate) fn candidate_count_by_key<K: Eq + Hash>(
    keys: impl Iterator<Item = K>,
) -> FxHashMap<K, usize> {
    let mut candidate_count_by_key =
        FxHashMap::with_capacity_and_hasher(keys.size_hint().0, Default::default());
    for key in keys {
        *candidate_count_by_key.entry(key).or_default() += 1;
    }
    candidate_count_by_key
}

pub(crate) struct Hydration {
    viewer: ViewerFeatures,
    viewer_state: Option<ViewerState>,
    planned: Hydrators,
    incomplete: Hydrators,
    tweets: FxHashMap<TweetId, HydratedTweet>,
    has_fetched_sources: bool,
}

impl Hydration {
    pub(crate) fn viewer(&self) -> &ViewerFeatures {
        &self.viewer
    }

            pub(crate) fn viewer_found(&self) -> bool {
        self.viewer_state.is_some()
    }

    pub(crate) fn planned(&self) -> Hydrators {
        self.planned
    }

        pub(crate) fn incomplete(&self) -> Hydrators {
        self.incomplete
    }

    pub(crate) fn tweet(&self, id: TweetId) -> Option<&HydratedTweet> {
        self.tweets.get(&id)
    }

            pub(crate) fn candidate(&self, id: TweetId, copied: bool) -> Option<Cow<'_, PostCandidate>> {
        let tweet = self.tweet(id)?;
        let candidate = tweet.candidate()?;
        let copy = copied
            .then(|| self.tweet(tweet.source_tweet_id()?)?.candidate())
            .flatten()
            .map(|source| CopiedFromSource::of(&source.post.tweet_features))
            .filter(|copy| !copy.is_on(&candidate.post.tweet_features));
        Some(match copy {
            None => Cow::Borrowed(candidate),
            Some(copy) => {
                let mut candidate = candidate.clone();
                copy.copy_onto(&mut candidate.post.tweet_features);
                Cow::Owned(candidate)
            }
        })
    }

    pub(crate) fn failed_nodes(&self, post: TweetId) -> Hydrators {
        let Some(HydratedTweet::Resolved {
            candidate,
            source_tweet_id,
            ..
        }) = self.tweet(post)
        else {
            return Hydrators::empty();
        };
        let source = source_tweet_id
            .and_then(|source| self.tweet(source)?.candidate())
            .map_or(Hydrators::empty(), |source| source.failed);
        candidate.failed.union(source)
    }

    pub(crate) fn has_fetched_sources(&self) -> bool {
        self.has_fetched_sources
    }
}

#[derive(Clone, Copy)]
struct CopiedFromSource {
    has_media: bool,
}

impl CopiedFromSource {
    fn of(source: &TweetFeatures) -> Self {
        Self {
            has_media: source.media.has_uploaded_media,
        }
    }

    fn is_on(self, retweet: &TweetFeatures) -> bool {
        retweet.media.has_media == self.has_media
    }

    fn copy_onto(self, retweet: &mut TweetFeatures) {
        retweet.media.has_media = self.has_media;
    }
}

#[expect(
    clippy::large_enum_variant,
    reason = "nearly every tweet resolves, so a box would cost an allocation per tweet"
)]
pub(crate) enum HydratedTweet {
    Resolved {
        candidate: PostCandidate,
        source_tweet_id: Option<TweetId>,
        safety_labels: Option<Arc<vf_pb::SafetyLabelMap>>,
    },
    Unresolved {
        reason: Unresolved,
        safety_labels: Option<Arc<vf_pb::SafetyLabelMap>>,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct Unresolved {
    pub(crate) lookup: Lookup,
    pub(crate) cause: Cause,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, strum::IntoStaticStr)]
#[strum(serialize_all = "snake_case")]
pub enum Lookup {
    Tweet,
    Author,
    SharedTweet,
    SharedAuthor,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, strum::IntoStaticStr)]
#[strum(serialize_all = "snake_case")]
pub(crate) enum Cause {
    NotFound,
    Failed,
}

impl HydratedTweet {
    pub(crate) fn candidate(&self) -> Option<&PostCandidate> {
        match self {
            Self::Resolved { candidate, .. } => Some(candidate),
            Self::Unresolved { .. } => None,
        }
    }

    pub(crate) fn source_tweet_id(&self) -> Option<TweetId> {
        match self {
            Self::Resolved {
                source_tweet_id, ..
            } => *source_tweet_id,
            Self::Unresolved { .. } => None,
        }
    }

    pub(crate) fn source_to_merge(&self) -> Option<TweetId> {
        match self {
            Self::Resolved {
                candidate,
                source_tweet_id,
                ..
            } if !candidate.failed.has_must_have() => *source_tweet_id,
            Self::Resolved { .. } | Self::Unresolved { .. } => None,
        }
    }

    pub(crate) fn safety_labels(&self) -> Option<&Arc<vf_pb::SafetyLabelMap>> {
        match self {
            Self::Resolved { safety_labels, .. } | Self::Unresolved { safety_labels, .. } => {
                safety_labels.as_ref()
            }
        }
    }
}

#[cfg(test)]
pub(crate) fn assert_blessed(fixture: &str, blessed: &str, rendered: &str) {
    if std::env::var_os("VF_BLESS").is_some() {
        let outputs = std::env::var("TEST_UNDECLARED_OUTPUTS_DIR").expect("bless under bazel test");
        std::fs::write(std::path::Path::new(&outputs).join(fixture), rendered).unwrap();
        return;
    }
    assert!(
        blessed == rendered,
        "{fixture} changed. Bless it with `bazel test --test_env=VF_BLESS=1 \
         --nozip_undeclared_test_outputs --test_arg=<test> <this target>`, then copy \
         `bazel-testlogs/<this target>/test.outputs/{fixture}` to tests/fixtures.\n\
         --- blessed\n{blessed}--- rendered\n{rendered}"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_context_budget_is_header_or_hang_guard_minus_allowance() {
        let entered = tokio::time::Instant::now();

        let header = Duration::from_millis(400);
        assert_eq!(
            request_context(entered, Some(header)).deadline,
            Some(entered + header - INBOUND_ALLOWANCE)
        );
        assert_eq!(
            request_context(entered, None).deadline,
            Some(entered + HYDRATION_TIMEOUT - INBOUND_ALLOWANCE)
        );
    }
}
