// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 X.AI Corp.
use std::collections::HashSet;
use std::io::Cursor;
use std::sync::Arc;

use arrow::array::{Array, AsArray, BooleanArray, Int64Array};
use arrow::ipc::reader::StreamReader;
use half::f16;
use numpy::ndarray::Array2;
use numpy::{PyArray2, ToPyArray};
use pyo3::prelude::*;
use rand::seq::SliceRandom;
use xai_recsys_mm_server::mm_embedding_client::MmEmbeddingsClient;
use xai_recsys_proto as pb;

use crate::util::RetrieveRequestItem;

const SNOWFLAKE_EPOCH_MS: i64 = 1288834974657;

#[derive(Clone)]
pub struct PrefetchMmQueryConfig {
    pub mm_embedding_dim: usize,
    pub query_sample_size: usize,
    pub max_age_hours: u64,
    pub positive_action_types: Arc<HashSet<i32>>,
}

fn age_cutoff_ms(max_age_hours: u64) -> i64 {
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0);
    now_ms - (max_age_hours as i64 * 3600 * 1000)
}

fn passes_age(tweet_id: i64, cutoff_ms: i64) -> bool {
    tweet_id != 0 && (tweet_id >> 22) + SNOWFLAKE_EPOCH_MS >= cutoff_ms
}

fn finalize_sample(mut pids: Vec<u64>, sample_size: usize) -> Vec<u64> {
    if pids.len() > sample_size {
        pids.shuffle(&mut rand::rng());
        pids.truncate(sample_size);
    }
    pids
}

fn sample_from_proto(
    sequence: Option<&pb::UserActionSequence>,
    config: &PrefetchMmQueryConfig,
) -> Vec<u64> {
    use pb::user_action_sequence_data_container::Data;

    let cutoff_ms = age_cutoff_ms(config.max_age_hours);
    let agg_actions = match sequence {
        Some(seq) => match &seq.user_actions_data {
            Some(container) => match &container.data {
                Some(Data::OrderedAggregatedUserActionsList(list)) => {
                    &list.aggregated_user_actions[..]
                }
                _ => &[],
            },
            None => &[],
        },
        None => &[],
    };

    let pids: Vec<u64> = agg_actions
        .iter()
        .filter_map(|a| {
            let tid = a.tweet_info.as_ref()?.tweet_id;
            if !passes_age(tid as i64, cutoff_ms) {
                return None;
            }
            let has_positive = a
                .actions
                .iter()
                .any(|act| config.positive_action_types.contains(&act.action_name));
            has_positive.then_some(tid)
        })
        .collect();

    finalize_sample(pids, config.query_sample_size)
}

fn sample_from_columnar(columnar_bytes: &[u8], config: &PrefetchMmQueryConfig) -> Vec<u64> {
    let cutoff_ms = age_cutoff_ms(config.max_age_hours);

    let cursor = Cursor::new(columnar_bytes);
    let batch = match StreamReader::try_new(cursor, None).and_then(|mut r| {
        r.next().unwrap_or_else(|| {
            Err(arrow::error::ArrowError::ComputeError(
                "empty stream".into(),
            ))
        })
    }) {
        Ok(batch) if batch.num_rows() > 0 => batch,
        _ => return Vec::new(),
    };

    let Some(tweet_ids) = batch
        .column_by_name("tweetId")
        .and_then(|c| c.as_any().downcast_ref::<Int64Array>())
    else {
        return Vec::new();
    };
    let action_multi_hot = batch.column_by_name("actionNameMultiHot");

    let pids: Vec<u64> = (0..batch.num_rows())
        .filter_map(|row_idx| {
            let tid = tweet_ids.value(row_idx);
            if !passes_age(tid, cutoff_ms) {
                return None;
            }
            let bools = action_multi_hot?
                .as_fixed_size_list()
                .value(row_idx)
                .as_any()
                .downcast_ref::<BooleanArray>()
                .cloned()?;
            let has_positive = config
                .positive_action_types
                .iter()
                .any(|&a| a >= 0 && (a as usize) < bools.len() && bools.value(a as usize));
            has_positive.then_some(tid as u64)
        })
        .collect();

    finalize_sample(pids, config.query_sample_size)
}

pub fn sample_positive_history_pids(
    columnar: Option<&[u8]>,
    sequence: Option<&pb::UserActionSequence>,
    config: &PrefetchMmQueryConfig,
) -> Vec<u64> {
    match columnar {
        Some(bytes) => sample_from_columnar(bytes, config),
        None => sample_from_proto(sequence, config),
    }
}

pub async fn prefetch_seed_embeddings(
    config: &PrefetchMmQueryConfig,
    client: &MmEmbeddingsClient,
    columnar: Option<&[u8]>,
    sequence: Option<&pb::UserActionSequence>,
) -> (Option<Vec<f16>>, Option<Vec<u64>>) {
    let pids = sample_positive_history_pids(columnar, sequence, config);
    if pids.is_empty() {
        return (None, None);
    }
    let n = pids.len();
    match client
        .fetch_mm_embeddings(pids.clone(), config.mm_embedding_dim, n)
        .await
    {
        Ok(embs) => (Some(embs), Some(pids)),
        Err(e) => {
            log::warn!("MM query pre-fetch failed: {}", e);
            (None, None)
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn read_pre_fetched_seed_embeddings<'py>(
    items: &[RetrieveRequestItem],
    py: Python<'py>,
    emb_dim: usize,
) -> PyResult<Vec<(Vec<u64>, Bound<'py, PyArray2<f32>>)>> {
    let mut out: Vec<(Vec<u64>, Bound<'py, PyArray2<f32>>)> = Vec::with_capacity(items.len());
    for item in items {
        match (&item.mm_query_seed_post_ids, &item.mm_query_embeddings) {
            (Some(pids), Some(embs_f16)) if !pids.is_empty() => {
                let n = pids.len();
                let expected = n * emb_dim;
                if embs_f16.len() < expected {
                    log::warn!(
                        "Pre-fetched MM embeddings shorter than expected: {} < {}",
                        embs_f16.len(),
                        expected
                    );
                }
                let take = embs_f16.len().min(expected);
                let mut as_f32: Vec<f32> = Vec::with_capacity(take);
                for v in &embs_f16[..take] {
                    as_f32.push(v.to_f32());
                }
                if take < expected {
                    as_f32.resize(expected, 0.0_f32);
                }
                let arr = Array2::from_shape_vec((n, emb_dim), as_f32).map_err(|e| {
                    pyo3::exceptions::PyRuntimeError::new_err(format!(
                        "shape mismatch building seed embedding array: {}",
                        e
                    ))
                })?;
                out.push((pids.clone(), arr.to_pyarray(py)));
            }
            _ => {
                let arr = Array2::<f32>::zeros((0, emb_dim));
                out.push((Vec::new(), arr.to_pyarray(py)));
            }
        }
    }
    Ok(out)
}
