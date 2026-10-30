use crate::config::{ENV_APP_ENV, ENV_REFERENCE};
use crate::evaluate_tweets::EntryComparator;
use crate::filter_tweets::{Comparator, FinishComparison};
use crate::models::RequestViewer;
use crate::params::ClientSwitches;
use crate::post_evaluator::{PostEvaluation, PostEvaluator};
use crate::quote::Passes;
use crate::rules::SafetyLevel;
use crate::server_deps::Comparators;
use std::env;
use std::sync::Arc;
use strum::VariantNames;
use tokio::time::Instant;
use tonic::metadata::MetadataMap;
use tweetypie::TweetypieReference;

pub(crate) mod tweetypie;

pub const ENV_IMAGE: &str = "VF_IMAGE";
pub const ENV_TWEETYPIE_XDS_LISTENER: &str = "VF_TWEETYPIE_XDS_LISTENER";
pub const ENV_TWEETYPIE_TLS_DOMAIN: &str = "VF_TWEETYPIE_TLS_DOMAIN";
pub const ENV_TWEETYPIE_CLIENT_ID: &str = "VF_TWEETYPIE_CLIENT_ID";

#[derive(Clone, Copy, Debug, PartialEq, Eq, strum::EnumString, strum::VariantNames)]
#[strum(serialize_all = "snake_case")]
pub enum Reference {
    None,
        Tweetypie,
}

impl Reference {
    pub fn resolve(configured: Option<&str>, app_env: Option<&str>) -> Result<Self, String> {
        let reference = configured.map_or(Ok(Self::None), |value| {
            value.parse().map_err(|_| {
                format!(
                    "{ENV_REFERENCE}={value:?} is not one of {}",
                    Self::VARIANTS.join(", ")
                )
            })
        })?;
        match (reference, app_env) {
            (Self::None, _) => Ok(Self::None),
            (_, Some("prod")) => Err(format!(
                "{ENV_REFERENCE} selects {reference:?} but {ENV_APP_ENV}=prod; reference comparisons are staging-only"
            )),
            (reference, _) => Ok(reference),
        }
    }
}

#[expect(clippy::panic, reason = "startup fail-fast on misconfiguration")]
fn reference() -> Reference {
    Reference::resolve(
        env::var(ENV_REFERENCE).ok().as_deref(),
        env::var(ENV_APP_ENV).ok().as_deref(),
    )
    .unwrap_or_else(|misconfiguration| panic!("{misconfiguration}"))
}

pub(crate) async fn build(
    init_deadline: Instant,
    post_evaluator: &Arc<PostEvaluator>,
    client_switches: &ClientSwitches,
    metadata: Option<&MetadataMap>,
) -> Comparators {
    match reference() {
        Reference::None => Comparators::default(),
        Reference::Tweetypie => {
            let reference = tweetypie::build(
                init_deadline,
                post_evaluator,
                client_switches.clone(),
                metadata,
            )
            .await;
            Comparators {
                filter_tweets: Some(Box::new(Arc::clone(&reference))),
                evaluate_tweets: Some(Box::new(reference)),
            }
        }
    }
}

impl Comparator for Arc<TweetypieReference> {
    fn begin(&self, viewer: &RequestViewer, safety_level: SafetyLevel) -> Option<FinishComparison> {
        let reference = Arc::clone(self);
        let viewer_id = viewer.id();
        let country_code = viewer.country_code().map(str::to_owned);
        Some(Box::new(move |outcomes| {
            reference.spawn(viewer_id, country_code, safety_level, outcomes);
        }))
    }
}

impl EntryComparator for Arc<TweetypieReference> {
    fn compare(
        &self,
        viewer: &RequestViewer,
        safety_level: SafetyLevel,
        outcomes: &[PostEvaluation],
        quote_passes: &Passes,
    ) {
        self.spawn_quote_comparison(viewer, safety_level, outcomes, quote_passes.ran());
    }
}

#[cfg(test)]
mod tests {
    use super::Reference;
    use serde::Deserialize;

    #[test]
    fn reference_defaults_to_none_and_is_refused_in_prod() {
        assert_eq!(
            Reference::resolve(None, Some("staging")),
            Ok(Reference::None)
        );
        assert_eq!(
            Reference::resolve(Some("none"), Some("staging")),
            Ok(Reference::None)
        );
        assert_eq!(
            Reference::resolve(Some("tweetypie"), Some("staging")),
            Ok(Reference::Tweetypie)
        );
        assert!(Reference::resolve(Some("Tweetypie"), Some("staging")).is_err());
        assert!(Reference::resolve(Some("vf_service"), Some("staging")).is_err());
        assert!(Reference::resolve(Some("tweetypie"), Some("prod")).is_err());
        assert_eq!(
            Reference::resolve(Some("none"), Some("prod")),
            Ok(Reference::None)
        );
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "snake_case")]
    enum Kind {
        Deliberate,
        Gap,
    }

        #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Divergence {
        divergence: String,
        kind: Kind,
        ticket: Option<String>,
        behavior: String,
    }

    #[test]
    fn each_divergence_states_its_behavior_and_a_gap_names_its_ticket() {
        let path = crate::pinned::crate_path("tests/divergences.json");
        let registry: Vec<Divergence> =
            serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
        for entry in registry {
            let name = &entry.divergence;
            assert!(!entry.behavior.is_empty(), "{name}: no behavior");
            assert!(
                matches!(entry.kind, Kind::Deliberate) || entry.ticket.is_some(),
                "{name}: a gap names its ticket"
            );
        }
    }
}
