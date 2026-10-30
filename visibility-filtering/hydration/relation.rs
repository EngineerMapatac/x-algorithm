use crate::hydration::inventory::Source;
use crate::hydration::store::KeyOrigin;
use crate::hydration::{Hydrator, Hydrators};
use crate::models::{AuthorId, PureCore, RetweetMerge, TweetFeatures, TweetId};
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, strum::IntoStaticStr, strum::VariantArray)]
#[strum(serialize_all = "snake_case")]
pub(crate) enum Relation {
    RetweetSource,
    ConversationRoot,
    ReactedTo,
    QuoteOuter,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Related {
    pub(super) tweet_id: TweetId,
    pub(super) author: Option<AuthorId>,
}

pub(super) struct RelationSpec {
    pub(super) trigger: Trigger,
    pub(super) role: Role,
    pub(super) fetch: Fetch,
}

#[derive(Clone, Copy)]
pub(super) enum Trigger {
    PureCore(fn(&PureCore) -> Option<Related>),
    Tweet(fn(&TweetFeatures) -> Option<Related>),
    Key(KeyOrigin),
    QuoteEntry,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Role {
    Judged,
    Read,
}

#[derive(Clone, Copy)]
pub(super) enum Fetch {
    Own(&'static [Source]),
    Shared,
}

impl Relation {
    pub(super) const fn spec(self) -> RelationSpec {
        match self {
            Relation::RetweetSource => RelationSpec {
                trigger: Trigger::PureCore(retweet_source),
                role: Role::Judged,
                fetch: Fetch::Shared,
            },
            Relation::ConversationRoot => RelationSpec {
                trigger: Trigger::Key(KeyOrigin::ConversationRootToFetch),
                role: Role::Read,
                fetch: Fetch::Own(&[Source::TesPureCoreData]),
            },
            Relation::ReactedTo => RelationSpec {
                trigger: Trigger::Tweet(reacted_to),
                role: Role::Read,
                fetch: Fetch::Own(&[Source::SafetyLabels, Source::GizmoduckAuthor]),
            },
            Relation::QuoteOuter => RelationSpec {
                trigger: Trigger::QuoteEntry,
                role: Role::Read,
                fetch: Fetch::Own(&[Source::Flock]),
            },
        }
    }
}

impl Relation {
    pub(super) const fn owns(self, source: Source) -> bool {
        let Fetch::Own(mut rest) = self.spec().fetch else {
            return false;
        };
        while let [owned, tail @ ..] = rest {
            if *owned as u8 == source as u8 {
                return true;
            }
            rest = tail;
        }
        false
    }

    pub(super) fn is_on(self, retweet_merge: RetweetMerge, planned: Hydrators) -> bool {
        match self.spec().role {
            Role::Judged => retweet_merge == RetweetMerge::Merged,
            Role::Read => planned
                .iter()
                .any(|node| node.spec().relation == Some(self)),
        }
    }
}

impl Trigger {
    pub(super) const fn input(self) -> Option<Hydrator> {
        match self {
            Trigger::PureCore(_) | Trigger::QuoteEntry => Some(Hydrator::PureCore),
            Trigger::Tweet(_) => Some(Hydrator::Tweet),
            Trigger::Key(origin) => origin.input(),
        }
    }

    pub(super) const fn reads(self) -> Hydrators {
        match self {
            Trigger::PureCore(_) | Trigger::QuoteEntry => Hydrators::of(Hydrator::PureCore),
            Trigger::Tweet(_) => Hydrators::of(Hydrator::Tweet),
            Trigger::Key(origin) => origin.reads(),
        }
    }

    pub(super) const fn needs_viewer(self) -> bool {
        match self {
            Trigger::PureCore(_) | Trigger::Tweet(_) | Trigger::QuoteEntry => false,
            Trigger::Key(origin) => origin.needs_viewer(),
        }
    }
}

fn retweet_source(core: &PureCore) -> Option<Related> {
    Some(Related {
        tweet_id: core.source_tweet_id?,
        author: core.source_author_id,
    })
}

pub(super) fn quoting_author(core: &PureCore) -> AuthorId {
    core.source_author_id.unwrap_or(core.author_id)
}

fn reacted_to(tweet: &TweetFeatures) -> Option<Related> {
    let context = tweet.reaction_context?;
    Some(Related {
        tweet_id: TweetId(context.reacted_to_post_id),
        author: Some(AuthorId(context.reacted_to_user_id)),
    })
}

impl fmt::Display for Relation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let spec = self.spec();
        let reads: Vec<&str> = spec.trigger.reads().iter().map(<&str>::from).collect();
        write!(
            f,
            "expand {} after: {}",
            <&str>::from(self),
            reads.join(",")
        )?;
        match spec.role {
            Role::Judged => f.write_str(" role: judged (retweet merge)")?,
            Role::Read => f.write_str(" role: read")?,
        }
        if spec.trigger.needs_viewer() {
            f.write_str(" (skipped logged out)")?;
        }
        Ok(())
    }
}
