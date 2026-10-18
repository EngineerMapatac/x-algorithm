use rustc_hash::FxHashMap;
use std::hash::Hash;
use std::time::Duration;

use tracing::debug;
use xai_stats_receiver::{global_stats_receiver, HistogramBuckets};

use crate::hydration::batch::{Hydrated, HydrationBatch, HydrationError};
use crate::params::CountryList;
use crate::rules::SafetyLevel;

const HYDRATOR_REQUESTS: &str = "vf_hydrator_requests";
const HYDRATOR_LATENCY_MS: &str = "vf_hydrator_latency_ms";
const HYDRATOR_KEYS: &str = "vf_hydrator_keys";
const HYDRATOR_TWEET_IDS: &str = "vf_hydrator_tweet_ids";
const HYDRATOR_BATCH_SIZE: &str = "vf_hydrator_batch_size";
const HYDRATOR_EXPANDED_BATCHES: &str = "vf_hydrator_expanded_batches";
const HYDRATOR_EXPANDED_KEYS: &str = "vf_hydrator_expanded_keys";
const HYDRATOR_UNASKED_KEYS: &str = "vf_hydrator_unasked_keys";
const FALLBACK_CACHE_KEYS: &str = "vf_fallback_cache_keys";
const FALLBACK_CACHE_ENTRIES: &str = "vf_fallback_cache_entries";
const FALLBACK_CACHE_RESIDENT_KEYS: &str = "vf_fallback_cache_resident_keys";
const VIEWER_COUNTRY: &str = "vf_viewer_country";
const VIEWER_STATE: &str = "vf_viewer_state";
const REQUEST_COUNTRY: &str = "vf_request_country";
const VIEWER_NOT_AGE_VERIFIED: &str = "vf_viewer_not_age_verified";
const FLOCK_MISSING_KEYS: &str = "vf_flock_missing_keys";

#[derive(Clone, Copy, Debug, PartialEq, Eq, strum::IntoStaticStr, strum::VariantArray)]
#[strum(serialize_all = "snake_case")]
pub(crate) enum HydratorOutcome {
    Success,
    Partial,
    Timeout,
    Error,
}

#[derive(Clone, Copy, strum::IntoStaticStr)]
#[strum(serialize_all = "snake_case")]
enum FallbackCacheResult {
    Fresh,
    Stale,
    NotFound,
    Partial,
    PartialStale,
    Unavailable,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct KeyedResultCounts {
    success_keys: usize,
    partial_keys: usize,
    timeout_keys: usize,
    error_keys: usize,
    success_candidates: usize,
    partial_candidates: usize,
    timeout_candidates: usize,
    error_candidates: usize,
}

impl KeyedResultCounts {
    fn from_batch<K, V>(
        candidate_count_by_claimed_key: &FxHashMap<K, usize>,
        batches: &impl AsRef<[HydrationBatch<K, V>]>,
    ) -> Self
    where
        K: Eq + Hash,
    {
        let mut counts = Self::default();
        for (key, &candidate_count) in candidate_count_by_claimed_key {
            let hydrated = batches
                .as_ref()
                .iter()
                .find_map(|batch| batch.hydrated(key));
            match hydrated {
                Some(Hydrated::Found(_) | Hydrated::NotFound) => {
                    counts.success_keys += 1;
                    counts.success_candidates += candidate_count;
                }
                Some(Hydrated::Partial(_)) => {
                    counts.partial_keys += 1;
                    counts.partial_candidates += candidate_count;
                }
                Some(Hydrated::Failed(HydrationError::Timeout)) => {
                    counts.timeout_keys += 1;
                    counts.timeout_candidates += candidate_count;
                }
                _ => {
                    counts.error_keys += 1;
                    counts.error_candidates += candidate_count;
                }
            }
        }
        counts
    }

    fn outcome(self) -> HydratorOutcome {
        let answered = self.success_keys + self.partial_keys;
        let failures = self.timeout_keys + self.error_keys;
        match (answered, failures) {
            (_, 0) if self.partial_keys == 0 => HydratorOutcome::Success,
            (0, _) if self.error_keys == 0 => HydratorOutcome::Timeout,
            (0, _) => HydratorOutcome::Error,
            _ => HydratorOutcome::Partial,
        }
    }
}

pub(crate) fn record_keyed_hydrator_request<K, V>(
    client: &str,
    method: &str,
    safety_level: SafetyLevel,
    candidate_count_by_claimed_key: &FxHashMap<K, usize>,
    answer: &impl AsRef<[HydrationBatch<K, V>]>,
    latency_ms: f64,
) where
    K: Eq + Hash,
{
    let counts = KeyedResultCounts::from_batch(candidate_count_by_claimed_key, answer);
    let outcome = counts.outcome();
    if outcome != HydratorOutcome::Success {
        debug!(
            client,
            method,
            outcome = <&str>::from(outcome),
            success_keys = counts.success_keys,
            partial_keys = counts.partial_keys,
            timeout_keys = counts.timeout_keys,
            error_keys = counts.error_keys,
            "Hydrator fail-open"
        );
    }
    incr(
        HYDRATOR_REQUESTS,
        &[
            ("client", client),
            ("method", method),
            ("outcome", outcome.into()),
            ("safety_level", safety_level.into()),
        ],
        1,
    );
    for (result, keys, candidates) in [
        (
            HydratorOutcome::Success,
            counts.success_keys,
            counts.success_candidates,
        ),
        (
            HydratorOutcome::Partial,
            counts.partial_keys,
            counts.partial_candidates,
        ),
        (
            HydratorOutcome::Timeout,
            counts.timeout_keys,
            counts.timeout_candidates,
        ),
        (
            HydratorOutcome::Error,
            counts.error_keys,
            counts.error_candidates,
        ),
    ] {
        let labels = [
            ("client", client),
            ("method", method),
            ("result", result.into()),
            ("safety_level", safety_level.into()),
        ];
        incr_nonzero(HYDRATOR_KEYS, &labels, keys as u64);
        incr_nonzero(HYDRATOR_TWEET_IDS, &labels, candidates as u64);
    }
    observe(
        HYDRATOR_LATENCY_MS,
        &[
            ("client", client),
            ("method", method),
            ("safety_level", safety_level.into()),
        ],
        latency_ms,
        HistogramBuckets::Bucket50To500,
    );
}

pub(crate) fn record_tes_join_latency(safety_level: SafetyLevel, elapsed: Duration) {
    observe(
        HYDRATOR_LATENCY_MS,
        &[
            ("client", "tes"),
            ("method", "join"),
            ("hydrator", "tes"),
            ("safety_level", safety_level.into()),
        ],
        elapsed.as_secs_f64() * 1000.0,
        HistogramBuckets::Bucket50To500,
    );
}

pub(crate) fn record_viewer_country(result: &'static str, safety_level: SafetyLevel) {
    incr(
        VIEWER_COUNTRY,
        &[("result", result), ("safety_level", safety_level.into())],
        1,
    );
}

pub(crate) fn record_viewer_state(state: &'static str, safety_level: SafetyLevel) {
    incr(
        VIEWER_STATE,
        &[("state", state), ("safety_level", safety_level.into())],
        1,
    );
}

const REQUEST_COUNTRY_EXTRA: [&str; 28] = [
    "ae", "ch", "cl", "co", "ec", "eg", "gh", "hk", "il", "iq", "jp", "ke", "kw", "my", "ng", "om",
    "pe", "pk", "sa", "sg", "tr", "tw", "ua", "us", "ve", "vn", "ye", "za",
];

fn request_country_label(request_country: Option<&str>) -> &'static str {
    let Some(code) = request_country else {
        return "none";
    };
    CountryList::listed_by_default(code)
        .or_else(|| {
            REQUEST_COUNTRY_EXTRA
                .into_iter()
                .find(|&extra| extra == code)
        })
        .unwrap_or("other")
}

pub(crate) fn record_request_country(request_country: Option<&str>) {
    incr(
        REQUEST_COUNTRY,
        &[("country", request_country_label(request_country))],
        1,
    );
}

pub(crate) fn record_viewer_not_age_verified(country: &'static str, age: &'static str) {
    incr(
        VIEWER_NOT_AGE_VERIFIED,
        &[("country", country), ("age", age)],
        1,
    );
}

pub(crate) fn record_flock_missing_keys(
    client: &str,
    method: &str,
    safety_level: SafetyLevel,
    keys: usize,
) {
    incr_nonzero(
        FLOCK_MISSING_KEYS,
        &[
            ("client", client),
            ("method", method),
            ("safety_level", safety_level.into()),
        ],
        keys as u64,
    );
}

pub(crate) fn record_batch_size(client: &str, candidate_count: usize) {
    observe(
        HYDRATOR_BATCH_SIZE,
        &[("client", client)],
        candidate_count as f64,
        HistogramBuckets::Bucket50To500,
    );
}

pub(crate) fn record_expanded_batch(client: &str, method: &str, keys: usize) {
    let labels = [("client", client), ("method", method)];
    incr(HYDRATOR_EXPANDED_BATCHES, &labels, 1);
    incr_nonzero(HYDRATOR_EXPANDED_KEYS, &labels, keys as u64);
}

pub(crate) fn record_unasked_keys(client: &str, method: &str, keys: usize) {
    incr_nonzero(
        HYDRATOR_UNASKED_KEYS,
        &[("client", client), ("method", method)],
        keys as u64,
    );
}

#[derive(Default)]
pub(crate) struct FallbackCacheCounts {
    pub(crate) fresh: usize,
    pub(crate) stale: usize,
    pub(crate) not_found: usize,
    pub(crate) partial: usize,
    pub(crate) partial_stale: usize,
    pub(crate) unavailable: usize,
    pub(crate) resident: usize,
}

impl FallbackCacheCounts {
    pub(crate) fn record(&self, cache: &'static str, column: &'static str) {
        let live = self.fresh + self.not_found + self.partial + self.partial_stale;
        for (metric, result, count) in [
            (
                FALLBACK_CACHE_KEYS,
                FallbackCacheResult::Fresh.into(),
                self.fresh,
            ),
            (
                FALLBACK_CACHE_KEYS,
                FallbackCacheResult::Stale.into(),
                self.stale,
            ),
            (
                FALLBACK_CACHE_KEYS,
                FallbackCacheResult::NotFound.into(),
                self.not_found,
            ),
            (
                FALLBACK_CACHE_KEYS,
                FallbackCacheResult::Partial.into(),
                self.partial,
            ),
            (
                FALLBACK_CACHE_KEYS,
                FallbackCacheResult::PartialStale.into(),
                self.partial_stale,
            ),
            (
                FALLBACK_CACHE_KEYS,
                FallbackCacheResult::Unavailable.into(),
                self.unavailable,
            ),
            (FALLBACK_CACHE_RESIDENT_KEYS, "resident", self.resident),
            (FALLBACK_CACHE_RESIDENT_KEYS, "absent", live - self.resident),
        ] {
            incr_nonzero(
                metric,
                &[("cache", cache), ("column", column), ("result", result)],
                count as u64,
            );
        }
    }
}

fn incr(metric: &str, labels: &[(&str, &str)], count: u64) {
    if let Some(sr) = global_stats_receiver() {
        sr.incr(metric, labels, count);
    }
}

fn incr_nonzero(metric: &str, labels: &[(&str, &str)], count: u64) -> bool {
    if count > 0 {
        incr(metric, labels, count);
        true
    } else {
        false
    }
}

pub(crate) fn record_fallback_cache_entries(cache: &'static str, entries: usize) {
    if let Some(sr) = global_stats_receiver() {
        sr.gauge(FALLBACK_CACHE_ENTRIES, &[("cache", cache)], entries as f64);
    }
}

fn observe(metric: &str, labels: &[(&str, &str)], value: f64, buckets: HistogramBuckets) {
    if let Some(sr) = global_stats_receiver() {
        sr.observe(metric, labels, value, buckets);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hydration::execute::Reply;
    use crate::hydration::inventory::Source;
    use crate::hydration::store::Store;
    use crate::models::{ClientCapability, RawCandidate, TweetFeatures, TweetId};
    use crate::rules::RuleEngine;
    use std::collections::HashMap;

    #[test]
    fn a_request_country_reads_its_listed_code_other_or_none() {
        for (country, label) in [
            (Some("de"), "de"),
            (Some("us"), "us"),
            (Some("zz"), "other"),
            (None, "none"),
        ] {
            assert_eq!(request_country_label(country), label, "{country:?}");
        }
        for extra in REQUEST_COUNTRY_EXTRA {
            assert_eq!(
                CountryList::listed_by_default(extra),
                None,
                "{extra} is listed"
            );
        }
    }

    #[test]
    fn dashboard_generator_pins_the_key_results_fallback_cache_results_and_viewer_metrics() {
        let cargo = concat!(env!("CARGO_MANIFEST_DIR"), "/scripts/dashboard.py");
        let ws = "crates/x-product/xai-visibility-filtering-service/scripts/dashboard.py";
        let path = if std::path::Path::new(cargo).exists() {
            cargo
        } else {
            ws
        };
        let dashboard =
            std::fs::read_to_string(path).unwrap_or_else(|e| panic!("read {path}: {e}"));
        assert!(dashboard.contains(&format!(
            "FALLBACK_CACHE_KEYS_METRIC = \"{FALLBACK_CACHE_KEYS}\""
        )));
        assert!(dashboard.contains(&format!("VIEWER_STATE_METRIC = \"{VIEWER_STATE}\"")));
        assert!(dashboard.contains(&format!("REQUEST_COUNTRY_METRIC = \"{REQUEST_COUNTRY}\"")));
        assert!(dashboard.contains(&format!(
            "VIEWER_NOT_AGE_VERIFIED_METRIC = \"{VIEWER_NOT_AGE_VERIFIED}\""
        )));
        for result in [
            FallbackCacheResult::Stale,
            FallbackCacheResult::Unavailable,
            FallbackCacheResult::PartialStale,
            FallbackCacheResult::Partial,
        ] {
            let result: &str = result.into();
            assert!(
                dashboard.contains(&format!("    \"{result}\": ")),
                "{result}"
            );
        }
        let key_results: Vec<&str> = <HydratorOutcome as strum::VariantArray>::VARIANTS
            .iter()
            .map(|&result| result.into())
            .collect();
        assert!(dashboard.contains(&format!(
            "HYDRATOR_KEY_RESULTS = \"{}\"",
            key_results.join("|")
        )));
    }

    #[test]
    fn partial_keys_make_a_partial_call_and_a_failed_one_key_call_is_an_error() {
        use crate::hydration::decode::author::author_batch;
        use xai_core_entities::entities::{GizmoduckUserResult, UserResponseState};
        let user = |state| {
            Ok::<_, ()>(Some(GizmoduckUserResult {
                user: Some(Default::default()),
                response_state: Some(state),
            }))
        };
        let authors = |states: &[(u64, UserResponseState)]| {
            let results = states
                .iter()
                .map(|&(author, state)| (author, user(state)))
                .collect::<HashMap<_, _>>();
            let expected: Vec<u64> = results.keys().copied().collect();
            author_batch(HydrationBatch::from_results(expected, results))
        };
        let counts = FxHashMap::from_iter([(10, 1), (20, 2)]);

        let one_partial = KeyedResultCounts::from_batch(
            &counts,
            &authors(&[
                (10, UserResponseState::Found),
                (20, UserResponseState::Partial),
            ]),
        );
        assert_eq!(
            (
                one_partial.success_keys,
                one_partial.partial_keys,
                one_partial.partial_candidates
            ),
            (1, 1, 2)
        );
        assert_eq!(one_partial.outcome(), HydratorOutcome::Partial);
        let all_partial = KeyedResultCounts::from_batch(
            &counts,
            &authors(&[
                (10, UserResponseState::Failed),
                (20, UserResponseState::Partial),
            ]),
        );
        assert_eq!(all_partial.outcome(), HydratorOutcome::Partial);

        let viewer: HydrationBatch<u64, u8> = HydrationBatch::from_results(
            [50],
            HashMap::from([(50, Err::<Option<u8>, _>("gizmoduck unavailable"))]),
        );
        let viewer = KeyedResultCounts::from_batch(&FxHashMap::from_iter([(50, 1)]), &viewer);
        assert_eq!((viewer.error_keys, viewer.error_candidates), (1, 1));
        assert_eq!(viewer.outcome(), HydratorOutcome::Error);
    }

    #[test]
    fn from_batch_counts_not_found_as_success_and_missing_as_error() {
        let expected = FxHashMap::from_iter([(1, 1), (2, 2), (3, 3), (4, 4)]);
        let batch: HydrationBatch<u64, u8> = HydrationBatch::from_results(
            [1, 2, 3, 4],
            HashMap::from([(1, Ok(Some(7))), (2, Ok(None)), (3, Err("boom"))]),
        );

        let counts = KeyedResultCounts::from_batch(&expected, &batch);

        assert_eq!(counts.success_keys, 2);
        assert_eq!(counts.success_candidates, 3);
        assert_eq!(counts.error_keys, 2);
        assert_eq!(counts.error_candidates, 7);
        assert_eq!(counts.outcome(), HydratorOutcome::Partial);
    }

    #[test]
    fn a_tweet_keyed_call_counts_its_keys_before_pure_core_resolves_authors() {
        let engine = RuleEngine::for_tests();
        let plan = engine.plan(SafetyLevel::TimelineHomeHydration);
        let raw = [RawCandidate {
            tweet_id: TweetId(1),
            request_author_id: None,
        }];
        let mut store = Store::new(plan, Some(50), ClientCapability::default(), &raw, false);
        let group = |source| plan.groups().find(|group| group.source == source).unwrap();
        let tweets = store.offer(group(Source::TesTweet)).unwrap();
        let circle = TweetFeatures {
            trusted_friends_list_id: Some(7),
            ..Default::default()
        };
        let landed =
            HydrationBatch::from_results([1], HashMap::from([(1, Ok::<_, ()>(Some(circle)))]));
        store.land(&tweets, Reply::Tweets(landed), Duration::ZERO);

        let call = store.offer(group(Source::TrustedFriends)).unwrap();
        let failed: HydrationBatch<u64, bool> =
            HydrationBatch::from_results(call.keys.clone(), HashMap::<_, Result<_, ()>>::new());
        let counts = KeyedResultCounts::from_batch(&call.candidate_count_by_claimed_key, &failed);
        assert_eq!((counts.error_keys, counts.error_candidates), (1, 1));
        assert_eq!(counts.outcome(), HydratorOutcome::Error);
    }

    #[test]
    fn a_claimed_key_no_candidate_counted_still_counts_its_timeout() {
        let batch: HydrationBatch<u64, u8> = HydrationBatch::timed_out([40, 41]);

        let counts =
            KeyedResultCounts::from_batch(&FxHashMap::from_iter([(40, 0), (41, 3)]), &batch);

        assert_eq!((counts.timeout_keys, counts.timeout_candidates), (2, 3));
        assert_eq!(counts.outcome(), HydratorOutcome::Timeout);
    }
}
