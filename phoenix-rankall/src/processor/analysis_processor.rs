use std::collections::HashMap;

use log::warn;

use super::record::IndexRecord;
use super::thrift_types::{deserialize_binary, deserialize_compact, ScoredCandidate};
use super::{ProcessorStats, RecordProcessor};

pub struct AnalysisProcessor {
    stats: ProcessorStats,
    score_prefix: String,
    compact_protocol: bool,
    warn_limit: u64,
}

impl AnalysisProcessor {
    pub fn new(score_prefix: String, compact_protocol: bool) -> Self {
        Self {
            stats: ProcessorStats::default(),
            score_prefix,
            compact_protocol,
            warn_limit: 5,
        }
    }

    fn is_excluded_suggest_type(suggest_type: &Option<String>) -> bool {
        match suggest_type {
            None => false,
            Some(st) => st.trim().eq_ignore_ascii_case("rankedfollowing"),
        }
    }
}

impl RecordProcessor for AnalysisProcessor {
    fn process_batch(&mut self, raw: &[Vec<u8>]) -> Vec<IndexRecord> {
        let mut results = Vec::with_capacity(raw.len());

        for payload in raw {
            self.stats.total_processed += 1;

            if payload.is_empty() {
                self.stats.total_invalid += 1;
                continue;
            }

            let candidate: ScoredCandidate = if self.compact_protocol {
                match deserialize_compact(payload) {
                    Ok(c) => c,
                    Err(e) => {
                        self.stats.total_deser_error += 1;
                        if self.stats.total_deser_error <= self.warn_limit {
                            warn!("failed to deserialize ScoredCandidate (compact): {e}");
                        }
                        continue;
                    }
                }
            } else {
                match deserialize_binary(payload) {
                    Ok(c) => c,
                    Err(e) => {
                        self.stats.total_deser_error += 1;
                        if self.stats.total_deser_error <= self.warn_limit {
                            warn!("failed to deserialize ScoredCandidate (binary): {e}");
                        }
                        continue;
                    }
                }
            };

            let tweet_id = candidate.tweet_id;
            let author_id = match candidate.author_id {
                Some(id) if id != 0 => id,
                _ => {
                    self.stats.total_invalid += 1;
                    continue;
                }
            };
            let viewer_id = match candidate.viewer_id {
                Some(id) if id != 0 => id,
                _ => {
                    self.stats.total_invalid += 1;
                    continue;
                }
            };

            if tweet_id == 0 {
                self.stats.total_invalid += 1;
                continue;
            }

            if Self::is_excluded_suggest_type(&candidate.suggest_type) {
                self.stats.total_filtered += 1;
                continue;
            }

            let prediction_scores = candidate.prediction_scores.unwrap_or_default();
            let mut scores: HashMap<String, f64> = HashMap::new();

            for ps in &prediction_scores {
                let name = match &ps.name {
                    Some(n) => n,
                    None => continue,
                };
                let score: f64 = match ps.score {
                    Some(s) => s.into(),
                    None => continue,
                };
                let name_str = name.as_str();

                if !self.score_prefix.is_empty() && !name_str.starts_with(&self.score_prefix) {
                    continue;
                }

                let short_name =
                    if !self.score_prefix.is_empty() && name_str.starts_with(&self.score_prefix) {
                        &name_str[self.score_prefix.len()..]
                    } else {
                        name_str
                    };

                scores.insert(short_name.to_string(), score);
            }

            if scores.is_empty() {
                self.stats.total_filtered += 1;
                continue;
            }

            self.stats.total_success += 1;
            results.push(IndexRecord::Analysis {
                post_id: tweet_id,
                author_id,
                viewer_id,
                served_type: candidate.suggest_type,
                scores,
            });
        }

        results
    }

    fn stats(&self) -> &ProcessorStats {
        &self.stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::processor::thrift_types::{serialize_binary, PredictionScore, ScoredCandidate};

    fn make_scored_candidate(
        tweet_id: i64,
        viewer_id: i64,
        author_id: i64,
        suggest_type: Option<&str>,
        scores: Vec<(&str, f64)>,
    ) -> Vec<u8> {
        use std::collections::BTreeSet;
        use thrift::OrderedFloat;

        let prediction_scores = if scores.is_empty() {
            None
        } else {
            let set: BTreeSet<Box<PredictionScore>> = scores
                .into_iter()
                .map(|(name, score)| {
                    Box::new(PredictionScore::new(
                        Some(name.to_string()),
                        Some(OrderedFloat::from(score)),
                    ))
                })
                .collect();
            Some(set)
        };

        serialize_binary(&ScoredCandidate::new(
            tweet_id,
            Some(viewer_id),
            Some(author_id),
            None::<OrderedFloat<f64>>,
            suggest_type.map(|s| s.to_string()),
            None::<i64>,
            None::<i64>,
            prediction_scores,
        ))
        .unwrap()
    }

    const PREFIX: &str = "phoenix.HomeExperiment1Lap7.";

    #[test]
    fn process_valid_scored_candidate() {
        let mut proc = AnalysisProcessor::new(PREFIX.to_string(), false);
        let raw = vec![make_scored_candidate(
            100,
            200,
            300,
            Some("RecTweet"),
            vec![
                ("phoenix.HomeExperiment1Lap7.engagement", 0.95),
                ("phoenix.HomeExperiment1Lap7.quality", 0.82),
                ("other.prefix.score", 0.5),
            ],
        )];

        let results = proc.process_batch(&raw);
        assert_eq!(results.len(), 1);

        match &results[0] {
            IndexRecord::Analysis {
                post_id,
                author_id,
                viewer_id,
                served_type,
                scores,
            } => {
                assert_eq!(*post_id, 100);
                assert_eq!(*author_id, 300);
                assert_eq!(*viewer_id, 200);
                assert_eq!(served_type.as_deref(), Some("RecTweet"));
                assert_eq!(scores.len(), 2, "only prefix-matched scores");
                assert!((scores["engagement"] - 0.95).abs() < 1e-10);
                assert!((scores["quality"] - 0.82).abs() < 1e-10);
            }
            _ => panic!("expected Analysis variant"),
        }
    }

    #[test]
    fn filter_excluded_suggest_type() {
        let mut proc = AnalysisProcessor::new(PREFIX.to_string(), false);
        let raw = vec![
            make_scored_candidate(
                100,
                200,
                300,
                Some("RankedFollowing"),
                vec![("phoenix.HomeExperiment1Lap7.x", 0.5)],
            ),
            make_scored_candidate(
                101,
                201,
                301,
                Some("RecTweet"),
                vec![("phoenix.HomeExperiment1Lap7.x", 0.5)],
            ),
        ];

        let results = proc.process_batch(&raw);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].post_id(), 101);
        assert_eq!(proc.stats().total_filtered, 1);
    }

    #[test]
    fn filter_no_matching_scores() {
        let mut proc = AnalysisProcessor::new(PREFIX.to_string(), false);
        let raw = vec![make_scored_candidate(
            100,
            200,
            300,
            Some("RecTweet"),
            vec![("some.other.prefix.score", 0.5)],
        )];

        let results = proc.process_batch(&raw);
        assert_eq!(results.len(), 0, "no matching scores → filtered");
        assert_eq!(proc.stats().total_filtered, 1);
    }

    #[test]
    fn skip_missing_required_ids() {
        let mut proc = AnalysisProcessor::new(PREFIX.to_string(), false);
        let raw = vec![
            make_scored_candidate(
                0,
                200,
                300,
                None,
                vec![("phoenix.HomeExperiment1Lap7.x", 0.5)],
            ),
            make_scored_candidate(
                100,
                0,
                300,
                None,
                vec![("phoenix.HomeExperiment1Lap7.x", 0.5)],
            ),
            make_scored_candidate(
                100,
                200,
                0,
                None,
                vec![("phoenix.HomeExperiment1Lap7.x", 0.5)],
            ),
        ];

        let results = proc.process_batch(&raw);
        assert_eq!(results.len(), 0);
        assert_eq!(proc.stats().total_invalid, 3);
    }

    #[test]
    fn handle_corrupt_payload() {
        let mut proc = AnalysisProcessor::new(PREFIX.to_string(), false);
        let raw = vec![vec![0xFF, 0xFE]];

        let results = proc.process_batch(&raw);
        assert!(results.is_empty());
        assert_eq!(proc.stats().total_deser_error, 1);
    }

    #[test]
    fn empty_prefix_keeps_all_scores() {
        let mut proc = AnalysisProcessor::new(String::new(), false);
        let raw = vec![make_scored_candidate(
            100,
            200,
            300,
            None,
            vec![("scoreA", 0.1), ("scoreB", 0.2)],
        )];

        let results = proc.process_batch(&raw);
        assert_eq!(results.len(), 1);

        match &results[0] {
            IndexRecord::Analysis { scores, .. } => {
                assert_eq!(scores.len(), 2);
                assert!(scores.contains_key("scoreA"));
                assert!(scores.contains_key("scoreB"));
            }
            _ => panic!("expected Analysis"),
        }
    }
}
